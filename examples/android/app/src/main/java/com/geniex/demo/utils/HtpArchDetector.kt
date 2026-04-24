package com.geniex.demo.utils

import android.os.Build
import android.util.Log

/**
 * Maps the device's Qualcomm SoC model to the Hexagon HTP arch version used
 * to select the right QAIRT model shards. Source of truth:
 * third-party/geniex-qairt/README.md (SoC → HTP arch table).
 */
object HtpArchDetector {
    private const val TAG = "HtpArchDetector"

    private val SOC_TO_ARCH = mapOf(
        "SM8550" to "v73", // Snapdragon 8 Gen 2
        "SM8650" to "v75", // Snapdragon 8 Gen 3 (not shipped in the public Qwen3 repo today)
        "SM8750" to "v79", // Snapdragon 8 Elite
        "SM8850" to "v81"  // Snapdragon 8 Elite Gen 5
    )

    /**
     * Returns the HTP arch string (e.g. "v79") for this device, or null if the SoC
     * is unknown. Callers should treat null as "skip arch-specific filtering".
     */
    fun detect(): String? {
        // Build.SOC_MODEL is API 31+. On older devices the constant reference
        // still compiles but reading it throws NoSuchFieldError at runtime.
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) {
            Log.w(TAG, "API ${Build.VERSION.SDK_INT} < 31, Build.SOC_MODEL unavailable; skipping arch detect")
            return null
        }
        val soc = Build.SOC_MODEL?.uppercase()
        if (soc.isNullOrBlank()) {
            Log.w(TAG, "Build.SOC_MODEL is blank, cannot detect HTP arch")
            return null
        }
        val arch = SOC_TO_ARCH[soc]
        if (arch == null) {
            Log.w(TAG, "Unknown SoC '$soc' - no HTP arch mapping; model downloads will not be filtered by arch")
        } else {
            Log.i(TAG, "Detected SoC=$soc -> HTP arch=$arch")
        }
        return arch
    }

    // All HTP arch directory names that may appear in a model repo, e.g. "v79",
    // "v81". Used by the file-listing filter to recognise the arch prefix on
    // repo entries like v79/part_1.bin.
    val KNOWN_ARCH_DIRS: Set<String> = SOC_TO_ARCH.values.toSet() + setOf("v68", "v69")
}
