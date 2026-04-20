use std::path::Path;
use hf_hub::{api::sync::ApiBuilder, Repo, RepoType};

use crate::error::{Error, Result};
use super::{ModelHub, ProgressCallback};

pub struct HfHub {
    api: hf_hub::api::sync::Api,
}

impl HfHub {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut builder = ApiBuilder::new();
        if let Some(t) = token {
            builder = builder.with_token(Some(t));
        }
        let api = builder.build().map_err(|e| Error::Hub(e.to_string()))?;
        Ok(Self { api })
    }
}

impl ModelHub for HfHub {
    fn download(
        &self,
        repo_id: &str,
        files: &[String],
        dest_dir: &Path,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<()> {
        std::fs::create_dir_all(dest_dir)?;
        let repo = self.api.repo(Repo::new(repo_id.to_string(), RepoType::Model));

        let mut downloaded: i64 = 0;
        let total: i64 = files.len() as i64; // rough progress unit: files

        for file_name in files {
            // hf-hub downloads to its own cache and returns the local path.
            let cached = repo.get(file_name).map_err(|e| Error::Hub(e.to_string()))?;
            let dest = dest_dir.join(file_name);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&cached, &dest)?;
            downloaded += 1;

            if let Some(cb) = on_progress {
                // Report file-level progress (byte-level requires hf-hub callbacks).
                if !cb(downloaded, total) {
                    return Err(Error::Hub("download cancelled by caller".to_string()));
                }
            }
        }
        Ok(())
    }
}
