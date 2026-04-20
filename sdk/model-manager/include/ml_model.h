#pragma once

/**
 * @file ml_model.h
 * @brief C API for model management: download, local storage, and path resolution.
 *
 * Implemented in Rust (sdk/model-manager), compiled to libgeniex_model.a and
 * linked into libgeniex.so via the GENIEX_MODEL_MANAGER CMake option.
 *
 * Memory convention (mirrors ml.h):
 *   - All output char* fields are heap-allocated; free them with ml_free().
 *   - Dedicated _free helpers (ml_model_paths_free, ml_model_list_free) call
 *     ml_free internally and zero out the struct.
 *   - Input pointers are caller-owned and never freed by this library.
 *
 * Error codes reuse the ml_ErrorCode range defined in ml.h (negative = error).
 */

#include <stdbool.h>
#include <stdint.h>
#include "ml.h" /* ML_API, ml_ErrorCode, ml_Path, ml_free() */

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================
 *  Initialization
 * ============================================================ */

/**
 * @brief Initialize the model manager.
 *
 * Must be called before any other ml_model_* function.
 *
 * @param data_dir  Local cache directory. NULL → defaults to ~/.cache/geniex
 *                  (or GENIEX_DATADIR env var).
 * @param hf_token  HuggingFace bearer token. NULL → anonymous access.
 * @return ML_SUCCESS on success, negative ml_ErrorCode on failure.
 */
ML_API int32_t ml_model_init(ml_Path data_dir, const char* hf_token);

/**
 * @brief Deinitialize the model manager and release resources.
 * @return ML_SUCCESS.
 */
ML_API int32_t ml_model_deinit(void);

/* ============================================================
 *  Model type
 * ============================================================ */

typedef enum {
    ML_MODEL_TYPE_LLM       = 0,
    ML_MODEL_TYPE_VLM       = 1,
    ML_MODEL_TYPE_EMBEDDER  = 2,
    ML_MODEL_TYPE_RERANKER  = 3,
    ML_MODEL_TYPE_TTS       = 4,
    ML_MODEL_TYPE_ASR       = 5,
    ML_MODEL_TYPE_DIARIZE   = 6,
    ML_MODEL_TYPE_CV        = 7,
    ML_MODEL_TYPE_IMAGE_GEN = 8,
} ml_ModelType;

/* ============================================================
 *  Path resolution
 * ============================================================ */

/**
 * @brief Resolved absolute file paths for a loaded model.
 *
 * All non-NULL char* fields are heap-allocated.
 * Free the entire struct with ml_model_paths_free().
 */
typedef struct {
    char* model_path;     /**< Main model file. For image_gen: the model directory. */
    char* mmproj_path;    /**< Multimodal projection file. NULL if unused.          */
    char* tokenizer_path; /**< Tokenizer file. NULL if unused.                       */
    char* model_dir;      /**< Model directory (always set).                         */
    char* model_name;     /**< Architecture name, e.g. "qwen3-4b".                  */
    char* plugin_id;      /**< Plugin ID, e.g. "llama_cpp".                          */
    char* device_id;      /**< Device ID. NULL means default device.                 */
} ml_ModelPaths;

/**
 * @brief Get resolved file paths for a model.
 *
 * @param model_name  "org/repo" or "org/repo:quant".
 *                    If quant is omitted the first downloaded quantization is used.
 * @param out_paths   Populated on success. Call ml_model_paths_free() when done.
 * @return ML_SUCCESS, or a negative ml_ErrorCode.
 */
ML_API int32_t ml_model_get_paths(const char* model_name, ml_ModelPaths* out_paths);

/** Free all heap strings inside ml_ModelPaths and zero the struct. */
ML_API void ml_model_paths_free(ml_ModelPaths* paths);

/* ============================================================
 *  Local cache management
 * ============================================================ */

typedef struct {
    char**  names; /**< Heap-allocated array of "org/repo" strings. */
    int32_t count;
} ml_ModelListOutput;

/**
 * @brief List all locally cached models.
 * @param output  Populated on success. Call ml_model_list_free() when done.
 * @return ML_SUCCESS, or a negative ml_ErrorCode.
 */
ML_API int32_t ml_model_list(ml_ModelListOutput* output);

/** Free the names array and zero the struct. */
ML_API void ml_model_list_free(ml_ModelListOutput* output);

/**
 * @brief Delete a cached model from disk.
 * @param model_name  "org/repo" format.
 * @return ML_SUCCESS, or ML_ERROR_COMMON_FILE_NOT_FOUND if not cached.
 */
ML_API int32_t ml_model_remove(const char* model_name);

/**
 * @brief Delete all cached models.
 * @param removed_count  Set to the number of deleted models. May be NULL.
 * @return ML_SUCCESS, or a negative ml_ErrorCode.
 */
ML_API int32_t ml_model_clean(int32_t* removed_count);

/**
 * @brief Get the model type of a cached model.
 * @param model_name  "org/repo" format.
 * @param out_type    Set on success.
 * @return ML_SUCCESS, or a negative ml_ErrorCode.
 */
ML_API int32_t ml_model_get_type(const char* model_name, ml_ModelType* out_type);

/* ============================================================
 *  Download
 * ============================================================ */

typedef enum {
    ML_HUB_AUTO        = 0, /**< Automatic hub selection              */
    ML_HUB_HUGGINGFACE = 1, /**< HuggingFace Hub                      */
    ML_HUB_MODELSCOPE  = 2, /**< ModelScope (mainland China preferred) */
    ML_HUB_S3          = 3, /**< AWS S3 (nexa-model-hub-bucket)        */
    ML_HUB_VOLCES      = 4, /**< Volces TOS (mainland China preferred) */
    ML_HUB_LOCALFS     = 5, /**< Local filesystem                      */
} ml_HubSource;

/**
 * @brief Progress callback invoked periodically during download.
 * @param downloaded_bytes  Bytes downloaded so far.
 * @param total_bytes       Total bytes to download (-1 if unknown).
 * @param user_data         Caller-provided pointer.
 * @return false to cancel the download, true to continue.
 */
typedef bool (*ml_download_progress_cb)(
    int64_t downloaded_bytes,
    int64_t total_bytes,
    void*   user_data
);

typedef struct {
    const char*             model_name;  /**< "org/repo" or short alias              */
    const char*             quant;       /**< Quantization hint. NULL for auto-select */
    ml_HubSource            hub;         /**< Use ML_HUB_AUTO for automatic selection */
    ml_Path                 local_path;  /**< Required only when hub == ML_HUB_LOCALFS */
    ml_download_progress_cb on_progress; /**< NULL to suppress progress reporting     */
    void*                   user_data;   /**< Forwarded to on_progress               */
} ml_ModelPullInput;

/**
 * @brief Download a model (blocking).
 *
 * Supports resume: partially downloaded files are continued from where they
 * left off.  On success the model is immediately available via ml_model_get_paths().
 *
 * @param input  Pull parameters. Must not be NULL.
 * @return ML_SUCCESS, or a negative ml_ErrorCode.
 */
ML_API int32_t ml_model_pull(const ml_ModelPullInput* input);

/* ============================================================
 *  Platform alias resolution
 * ============================================================ */

/**
 * @brief Resolve a short alias to the canonical "org/repo" name for the
 *        current OS and CPU architecture.
 *
 * Example: "qwen3"   → "NexaAI/Qwen3-4B-GGUF"   (x86-64)
 *          "qwen3vl" → "NexaAI/Qwen3-VL-4B-NPU"  (Windows arm64)
 *
 * @param alias         Short model name.
 * @param out_full_name Set to a heap-allocated string on success.
 *                      Free with ml_free().
 * @return ML_SUCCESS if resolved, ML_ERROR_COMMON_INVALID_INPUT if unknown alias.
 */
ML_API int32_t ml_model_resolve_alias(const char* alias, char** out_full_name);

#ifdef __cplusplus
} /* extern "C" */
#endif
