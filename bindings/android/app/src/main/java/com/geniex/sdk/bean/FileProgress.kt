package com.geniex.sdk.bean

/** Per-file download progress. [total_bytes] is -1 when the total size is not yet known. */
data class FileProgress(
    val file_name: String,
    val downloaded_bytes: Long,
    val total_bytes: Long,
)
