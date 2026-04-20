/**
 * End-to-end test for the model manager C API.
 *
 * Build (after cmake -DGENIEX_MODEL_MANAGER=ON):
 *
 *   cmake --build <build-dir> --target test_model_manager
 *
 * Run:
 *   LD_LIBRARY_PATH=<build-dir>/src GENIEX_DATADIR=/tmp/geniex-test \
 *       ./<build-dir>/src/test_model_manager
 *
 * Two test modes:
 *   1. LocalFS (always runs): creates a dummy geniex.json and model file in /tmp,
 *      then exercises the full CRUD + path resolution flow.
 *   2. HuggingFace (optional, set GENIEX_TEST_HF=1): downloads a real model from HF.
 *      Requires network access and a geniex.json in the target HF repo.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#include "ml.h"
#include "ml_model.h"

/* ---- helpers ---- */

#define CHECK(call)                                                        \
    do {                                                                   \
        int32_t _rc = (call);                                              \
        if (_rc != ML_SUCCESS) {                                           \
            fprintf(stderr, "FAIL  %s  (rc=%d)\n", #call, _rc);           \
            return 1;                                                      \
        }                                                                  \
        printf("OK    %s\n", #call);                                       \
    } while (0)

#define EXPECT_FAIL(call, expected_rc)                                     \
    do {                                                                   \
        int32_t _rc = (call);                                              \
        if (_rc != (expected_rc)) {                                        \
            fprintf(stderr, "FAIL  %s  expected rc=%d got rc=%d\n",       \
                    #call, (expected_rc), _rc);                            \
            return 1;                                                      \
        }                                                                  \
        printf("OK    %s  (expected failure rc=%d)\n", #call, _rc);       \
    } while (0)

static void mkdir_p(const char* path) {
    char tmp[512];
    snprintf(tmp, sizeof(tmp), "%s", path);
    for (char* p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0755);
            *p = '/';
        }
    }
    mkdir(tmp, 0755);
}

static void write_file(const char* path, const char* content) {
    FILE* f = fopen(path, "w");
    if (!f) { perror(path); exit(1); }
    fputs(content, f);
    fclose(f);
}

/* ---- LocalFS test ---- */

static int test_localfs(const char* data_dir) {
    printf("\n=== LocalFS test ===\n");

    /* 1. Create a fake model source directory */
    const char* src_dir = "/tmp/geniex-localfs-src/NexaAI/TestModel-GGUF";
    mkdir_p(src_dir);

    /* Fake GGUF file */
    char gguf_path[512];
    snprintf(gguf_path, sizeof(gguf_path), "%s/model-Q4_K_M.gguf", src_dir);
    write_file(gguf_path, "fake gguf content");

    /* geniex.json manifest */
    char manifest_path[512];
    snprintf(manifest_path, sizeof(manifest_path), "%s/geniex.json", src_dir);
    write_file(manifest_path,
        "{"
        "\"Name\":\"NexaAI/TestModel-GGUF\","
        "\"ModelName\":\"test-1b\","
        "\"ModelType\":\"llm\","
        "\"PluginId\":\"llama_cpp\","
        "\"DeviceId\":\"\","
        "\"MinSDKVersion\":\"\","
        "\"ModelFile\":{\"Q4_K_M\":{\"Name\":\"model-Q4_K_M.gguf\",\"Downloaded\":true,\"Size\":17}},"
        "\"MMProjFile\":{\"Name\":\"\",\"Downloaded\":false,\"Size\":0},"
        "\"TokenizerFile\":{\"Name\":\"\",\"Downloaded\":false,\"Size\":0},"
        "\"ExtraFiles\":[]"
        "}"
    );

    /* 2. Pull from LocalFS */
    ml_ModelPullInput pull_input = {
        .model_name = "NexaAI/TestModel-GGUF",
        .quant      = NULL,
        .hub        = ML_HUB_LOCALFS,
        .local_path = "/tmp/geniex-localfs-src/NexaAI/TestModel-GGUF",
        .on_progress = NULL,
        .user_data  = NULL,
    };
    CHECK(ml_model_pull(&pull_input));

    /* 3. List */
    ml_ModelListOutput list = {0};
    CHECK(ml_model_list(&list));
    printf("      cached models: %d\n", list.count);
    if (list.count < 1) {
        fprintf(stderr, "FAIL  expected at least 1 cached model\n");
        ml_model_list_free(&list);
        return 1;
    }
    printf("      [0] %s\n", list.names[0]);
    ml_model_list_free(&list);

    /* 4. Get type */
    ml_ModelType mtype;
    CHECK(ml_model_get_type("NexaAI/TestModel-GGUF", &mtype));
    if (mtype != ML_MODEL_TYPE_LLM) {
        fprintf(stderr, "FAIL  expected ML_MODEL_TYPE_LLM (%d), got %d\n",
                ML_MODEL_TYPE_LLM, mtype);
        return 1;
    }
    printf("      model type: LLM ✓\n");

    /* 5. Get paths */
    ml_ModelPaths paths = {0};
    CHECK(ml_model_get_paths("NexaAI/TestModel-GGUF", &paths));
    printf("      model_path: %s\n", paths.model_path ? paths.model_path : "(null)");
    printf("      model_dir:  %s\n", paths.model_dir  ? paths.model_dir  : "(null)");
    printf("      model_name: %s\n", paths.model_name ? paths.model_name : "(null)");
    printf("      plugin_id:  %s\n", paths.plugin_id  ? paths.plugin_id  : "(null)");
    if (!paths.model_path || strstr(paths.model_path, "model-Q4_K_M.gguf") == NULL) {
        fprintf(stderr, "FAIL  model_path does not contain expected filename\n");
        ml_model_paths_free(&paths);
        return 1;
    }
    printf("      model_path contains 'model-Q4_K_M.gguf' ✓\n");
    ml_model_paths_free(&paths);

    /* 6. Get paths with explicit quant */
    CHECK(ml_model_get_paths("NexaAI/TestModel-GGUF:Q4_K_M", &paths));
    ml_model_paths_free(&paths);

    /* 7. Error case: unknown quant */
    EXPECT_FAIL(
        ml_model_get_paths("NexaAI/TestModel-GGUF:Q8_0", &paths),
        ML_ERROR_COMMON_INVALID_INPUT
    );

    /* 8. Remove */
    CHECK(ml_model_remove("NexaAI/TestModel-GGUF"));

    /* 9. Verify gone */
    ml_ModelListOutput list2 = {0};
    CHECK(ml_model_list(&list2));
    if (list2.count != 0) {
        fprintf(stderr, "FAIL  expected 0 models after remove, got %d\n", list2.count);
        ml_model_list_free(&list2);
        return 1;
    }
    printf("      list after remove: 0 ✓\n");
    ml_model_list_free(&list2);

    printf("=== LocalFS test PASSED ===\n");
    return 0;
}

/* ---- alias test ---- */

static int test_alias(void) {
    printf("\n=== Alias test ===\n");
    char* full = NULL;
    CHECK(ml_model_resolve_alias("qwen3", &full));
    printf("      qwen3 -> %s\n", full);
    ml_free(full);

    /* unknown alias should fail */
    EXPECT_FAIL(
        ml_model_resolve_alias("nonexistent_model_xyz_abc", &full),
        ML_ERROR_COMMON_INVALID_INPUT
    );
    printf("=== Alias test PASSED ===\n");
    return 0;
}

/* ---- main ---- */

int main(void) {
    const char* data_dir = getenv("GENIEX_DATADIR");
    const char* hf_token = getenv("HF_TOKEN");

    printf("=== ml_model_init ===\n");
    CHECK(ml_model_init(data_dir, hf_token));

    if (test_alias())   return 1;
    if (test_localfs(data_dir ? data_dir : "/tmp/geniex-test")) return 1;

    CHECK(ml_model_deinit());

    printf("\n=== ALL TESTS PASSED ===\n");
    return 0;
}
