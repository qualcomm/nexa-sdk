use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub data_dir: PathBuf,
    pub hf_token: Option<String>,
}

impl StoreConfig {
    /// Read from environment variables: GENIEX_DATADIR, GENIEX_HFTOKEN.
    /// Falls back to ~/.cache/geniex if GENIEX_DATADIR is unset.
    pub fn from_env() -> Self {
        let data_dir = if let Ok(d) = std::env::var("GENIEX_DATADIR") {
            PathBuf::from(d)
        } else {
            default_data_dir()
        };
        let hf_token = std::env::var("GENIEX_HFTOKEN").ok();
        Self { data_dir, hf_token }
    }

    pub fn new(data_dir: PathBuf, hf_token: Option<String>) -> Self {
        Self { data_dir, hf_token }
    }

    pub fn models_dir(&self) -> PathBuf {
        self.data_dir.join("models")
    }

    pub fn model_dir(&self, name: &str) -> PathBuf {
        self.models_dir().join(name)
    }

    pub fn model_file_path(&self, name: &str, file: &str) -> PathBuf {
        self.model_dir(name).join(file)
    }
}

fn default_data_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache").join("geniex")
}
