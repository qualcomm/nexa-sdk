use crate::error::Result;
use crate::hub::{HubSource, ModelHub, ProgressCallback};
use crate::hub::hf::HfHub;
use crate::hub::localfs::LocalFsHub;
use crate::store::Store;

pub struct PullRequest {
    /// "org/repo" (may be a short alias already resolved by the caller).
    pub model_name: String,
    pub hub: HubSource,
    pub on_progress: Option<ProgressCallback>,
}

/// Download a model and write its manifest to the store.
///
/// The remote hub must expose a `geniex.json` manifest that the store will parse
/// to know which files to download.  After fetching all files the manifest is
/// written locally so subsequent calls to `store.get_manifest()` succeed.
pub fn pull(store: &Store, req: PullRequest) -> Result<()> {
    let hub: Box<dyn ModelHub> = match req.hub {
        HubSource::HuggingFace => {
            // Token is read from StoreConfig; retrieve via the store's cfg.
            // We pass None here; the HfHub reads HUGGING_FACE_HUB_TOKEN / HF_TOKEN env vars too.
            Box::new(HfHub::new(None)?)
        }
        HubSource::LocalFs(path) => Box::new(LocalFsHub::new(path)),
    };

    let dest_dir = store.model_file_path(&req.model_name, "");
    std::fs::create_dir_all(&dest_dir)?;

    // Step 1: Fetch geniex.json from the hub to learn which files to download.
    hub.download(
        &req.model_name,
        &["geniex.json".to_string()],
        &dest_dir,
        req.on_progress.as_ref(),
    )?;

    // Step 2: Parse the manifest to get the file list.
    let manifest_path = dest_dir.join("geniex.json");
    let data = std::fs::read_to_string(&manifest_path)?;
    let manifest: crate::manifest::ModelManifest = serde_json::from_str(&data)?;

    // Step 3: Build the list of model files to download.
    let mut files: Vec<String> = Vec::new();
    for fi in manifest.model_file.values() {
        if fi.downloaded && !fi.name.is_empty() {
            files.push(fi.name.clone());
        }
    }
    if manifest.mmproj_file.downloaded && !manifest.mmproj_file.name.is_empty() {
        files.push(manifest.mmproj_file.name.clone());
    }
    if manifest.tokenizer_file.downloaded && !manifest.tokenizer_file.name.is_empty() {
        files.push(manifest.tokenizer_file.name.clone());
    }
    for fi in &manifest.extra_files {
        if fi.downloaded && !fi.name.is_empty() {
            files.push(fi.name.clone());
        }
    }

    // Step 4: Download model files.
    hub.download(&req.model_name, &files, &dest_dir, req.on_progress.as_ref())?;

    // Step 5: Persist the manifest to the store.
    store.write_manifest(&manifest)?;

    Ok(())
}
