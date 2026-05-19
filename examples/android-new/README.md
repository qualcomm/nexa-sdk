[![Qualcomm® AI Hub Apps](https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/quic-logo.jpg)](https://aihub.qualcomm.com)

# Geniex Demo

On-device AI chat application for Android on Snapdragon® powered by the
[Geniex SDK](https://github.com/qualcomm/geniex). Runs Large Language Models
(LLMs), Vision-Language Models (VLMs), embeddings, speech recognition, and
reranking models on the Snapdragon® Neural Processing Unit (NPU), GPU, or CPU
through a single pluggable runtime.

## Current limitations

> [!IMPORTANT]
> This app requires a **mobile** device with Snapdragon® 8 Gen 3 or newer and
> Android API level 31 (Android 12) or newer. NPU acceleration on older
> Snapdragon® chipsets falls back to CPU/GPU execution.

We recommend using a device from [QDC](https://qdc.qualcomm.com/) for this
demo. QDC devices have newer meta-builds and are validated for on-device LLM
inference.

## Demo

<!-- TODO: replace with mp4/gif hosted on qaihub-public-assets S3 -->

## Requirements

### Platform

- Snapdragon® 8 Gen 3, Snapdragon® 8 Elite, or newer
- Or access to a Snapdragon® Android device on [QDC](https://qdc.qualcomm.com/)
- The host computer can run Windows, Linux, or macOS.

### Tools and SDK

1. Clone this repository.
2. Download [Android Studio](https://developer.android.com/studio). **Version
   2024.3.1 or newer** is required.
3. The Geniex Android binding is consumed from Maven Central as
   `com.qualcomm.qti:geniex-android:0.1.6` — no separate SDK download is
   required. Gradle resolves it automatically on first sync.

## Build App

1. Go to the Geniex Demo directory.

   ```bash
   cd <ai-hub-apps-repo-root>/apps/geniex_chat_android/
   ```

2. Download a model and place it under `src/main/assets/models/`. The demo
   ships with `src/main/assets/model_list.json` describing the supported model
   catalog (LLMs from Hugging Face in GGUF format, plus optional QAIRT `.bin`
   bundles). For example, to use Qwen3-0.6B:

   ```bash
   mkdir -p src/main/assets/models/qwen3-0.6b
   # Download Qwen3-0.6B-GGUF Q4_0 from Hugging Face into this directory.
   ```

3. Build the APK.

   - Open this folder in Android Studio.
   - Run Gradle sync.
   - Build the `app` target: `Build` → `Build Bundle(s) / APK(s)` → `Build
     APK(s)`.
   - The APK is written to:

     ```text
     <ai-hub-apps-repo-root>/apps/geniex_chat_android/build/outputs/apk/{build_type}/
     ```

4. Run on an Android device.

   We recommend using [QDC](https://qdc.qualcomm.com/) to run this app.

   **Steps for running Geniex Demo on QDC**

   1. Copy the APK to your QDC device (browser upload, file browser, or
      `adb push` over SSH tunneling — see [QDC docs](https://qdc.qualcomm.com/)
      for details).

   2. Install the APK via `adb shell`:

      ```bash
      pm install -t /data/local/tmp/app-debug.apk
      ```

   3. Open the app from the QDC browser UI and start chatting.

## License

This app is released under the [BSD-3 License](../../../LICENSE) found at the
root of this repository.

The Geniex SDK dependency is released under its own license (Apache 2.0).
Refer to the [Geniex repository](https://github.com/qualcomm/geniex) for
details.

All third-party libraries used by this app are BSD-3-compatible (Apache 2.0
or MIT). Refer to each library's source repository for license details.
