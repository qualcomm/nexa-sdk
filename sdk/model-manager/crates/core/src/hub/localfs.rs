use std::path::{Path, PathBuf};
use crate::error::{Error, Result};
use super::{ModelHub, ProgressCallback};

pub struct LocalFsHub {
    source_dir: PathBuf,
}

impl LocalFsHub {
    pub fn new(source_dir: PathBuf) -> Self {
        Self { source_dir }
    }
}

impl ModelHub for LocalFsHub {
    fn download(
        &self,
        _repo_id: &str,
        files: &[String],
        dest_dir: &Path,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<()> {
        std::fs::create_dir_all(dest_dir)?;
        let total = files.len() as i64;

        for (i, file_name) in files.iter().enumerate() {
            let src = self.source_dir.join(file_name);
            if !src.exists() {
                return Err(Error::Hub(format!("local file not found: {}", src.display())));
            }
            let dest = dest_dir.join(file_name);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&src, &dest)?;

            if let Some(cb) = on_progress {
                if !cb(i as i64 + 1, total) {
                    return Err(Error::Hub("download cancelled by caller".to_string()));
                }
            }
        }
        Ok(())
    }
}
