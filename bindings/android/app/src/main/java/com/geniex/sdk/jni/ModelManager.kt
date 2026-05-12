package com.geniex.sdk.jni

import com.geniex.sdk.bean.ModelPaths
import com.geniex.sdk.bean.ModelPullInput
import com.geniex.sdk.callback.DownloadProgressCallback

/**
 * Thin Kotlin shim over the `geniex_model_*` C FFI. Functions are
 * instance methods but operate on a process-global store (matches the
 * FFI's singleton). Prefer `ModelManagerWrapper` from app code.
 */
internal class ModelManager {
    /**
     * @return 0 on success, the FFI error code otherwise.
     *   `GENIEX_ERROR_COMMON_ALREADY_INITIALIZED` (-100008) if the
     *   store has already been initialized.
     */
    external fun init(dataDir: String): Int

    external fun deinit(): Int

    /**
     * Blocking. Invokes [callback] periodically (~100 ms) from a tokio
     * worker thread that the bridge attaches to the JVM. Returning
     * `false` from the callback cancels; the FFI returns
     * `GENIEX_ERROR_COMMON_CANCELLED` (-100006).
     */
    external fun pull(input: ModelPullInput, callback: DownloadProgressCallback?): Int

    external fun list(): Array<String>

    /** @return `null` if the model is not cached or paths cannot be resolved. */
    external fun getPaths(modelName: String): ModelPaths?

    external fun remove(modelName: String): Int

    external fun clean(): Int

    /**
     * @return `geniex_ModelType` as an ordinal (0 = LLM, 1 = VLM), or a
     *   negative FFI error code.
     */
    external fun getType(modelName: String): Int

    external fun resolveAlias(alias: String): String?
}
