package com.geniex.sdk.bean

/** Mirrors `geniex_ModelType` from geniex_model.h. */
enum class ModelType(val value: Int) {
    LLM(0),
    VLM(1);

    companion object {
        fun fromValue(v: Int): ModelType? = values().firstOrNull { it.value == v }
    }
}
