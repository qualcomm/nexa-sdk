pub mod hf;
pub mod localfs;

use std::path::Path;
use crate::error::Result;

/// Progress callback: (downloaded_bytes, total_bytes). Return false to cancel.
pub type ProgressCallback = Box<dyn Fn(i64, i64) -> bool>;

pub enum HubSource {
    HuggingFace,
    LocalFs(std::path::PathBuf),
}

pub trait ModelHub {
    /// Download all required files to `dest_dir`.
    /// `files` is a list of filenames (relative to the model root) to fetch.
    /// `on_progress` is called periodically; returning false cancels the download.
    fn download(
        &self,
        repo_id: &str,
        files: &[String],
        dest_dir: &Path,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<()>;
}
