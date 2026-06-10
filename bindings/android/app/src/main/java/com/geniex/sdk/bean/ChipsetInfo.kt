package com.geniex.sdk.bean

/** Mirrors `geniex_ChipsetInfo`. One chipset Qualcomm AI Hub supports. */
data class ChipsetInfo(
    val name: String,
    val aliases: Array<String>,
)
