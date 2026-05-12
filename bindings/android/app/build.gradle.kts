plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.geniex.sdk"
    compileSdk = 35
    ndkVersion = "29.0.14206865"

    defaultConfig {
        minSdk = 27

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        externalNativeBuild {
            cmake {
                cppFlags += "-std=c++17"
            }
        }
        ndk {
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    packaging {
        jniLibs.useLegacyPackaging = true
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    kotlinOptions {
        jvmTarget = "11"
    }
    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            version = "3.22.1"
        }
    }
}

// Copy the prebuilt geniex libraries from sdk/pkg-geniex so they get packaged
// into jniLibs (and therefore into the final APK of any consumer).
val pkgGeniexDir = file("$projectDir/../../../sdk/pkg-geniex")
val jniOutDir = file("$projectDir/src/main/jniLibs/arm64-v8a")

val copyBridgeLibs = tasks.register<Copy>("copyBridgeLibs") {
    require(pkgGeniexDir.exists()) {
        "SDK package not found at ${pkgGeniexDir.absolutePath}"
    }
    val libDir = File(pkgGeniexDir, "lib")
    from(libDir) { include("libgeniex.so") }
    from(File(libDir, "llama_cpp")) {
        include("*.so")
        exclude("*.a")
        // The SDK package ships multiple libgeniex_plugin.so variants, one per backend.
        // They must be renamed to coexist in the flat APK jniLibs directory.
        rename("libgeniex_plugin\\.so", "libgeniex_plugin_llama_cpp.so")
    }
    from(File(libDir, "qairt")) {
        include("libgeniex_core.so", "libgeniex_plugin.so")
        rename("libgeniex_plugin\\.so", "libgeniex_plugin_qairt.so")
    }
    // Do NOT copy from qairt/htp-files/: the Windows CLI package ships
    // Hexagon DSP (32-bit) binaries there for FastRPC, which Android's
    // linker then rejects out of arm64-v8a/. Instead, pull the ARM64
    // CPU-side QAIRT client libs from the qairt submodule's Android
    // third-party dir (these are what the qairt plugin dlopens).
    from(File(projectDir, "../../../third-party/geniex-qairt/third-party/android")) {
        include("*.so")
        // Skip DSP skels and Hexagon-only binaries (32-bit); leave those
        // to GeniexSdk.extractHtpAssets which unpacks them from assets/.
        exclude("libCalculator_skel.so")
        exclude("libQnnHtpV??Skel.so")
        exclude("libQnnHtpV??.so")
        exclude("libQnnHtpV??QemuDriver.so")
        exclude("libQnnNetRunDirectV??Skel.so")
        exclude("libSnpeHtpV??Skel.so")
        exclude("libqnnhtpv??.cat")
    }
    from(File(projectDir, "extLibs/arm64-v8a")) { include("*.so") }
    into(jniOutDir)
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
}

tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn(copyBridgeLibs)
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.kotlinx.coroutines.android)
}
