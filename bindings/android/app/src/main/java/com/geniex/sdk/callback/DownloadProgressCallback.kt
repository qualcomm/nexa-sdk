package com.geniex.sdk.callback

import com.geniex.sdk.bean.FileProgress

/**
 * Progress callback invoked periodically during a model pull. Return
 * `false` to request cancellation — the Rust downloader unwinds the
 * current chunk and leaves partial files on disk for a later resume.
 *
 * Called from a JNI-attached worker thread; callers must not assume
 * the Android main looper.
 */
interface DownloadProgressCallback {
    fun onProgress(files: Array<FileProgress>): Boolean
}
