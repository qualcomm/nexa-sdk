use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use crate::config::StoreConfig;
use crate::error::{Error, Result};
use crate::manifest::{ModelManifest, ModelType};
use crate::paths::{resolve_model_paths, ModelPaths};

const MANIFEST_FILE: &str = "geniex.json";

pub struct Store {
    cfg: StoreConfig,
    // per-model mutex map; key is "org/repo"
    locks: Mutex<HashMap<String, Mutex<()>>>,
}

impl Store {
    pub fn new(cfg: StoreConfig) -> Result<Self> {
        fs::create_dir_all(cfg.models_dir())?;
        Ok(Self { cfg, locks: Mutex::new(HashMap::new()) })
    }

    fn lock_model<'a>(
        &'a self,
        name: &str,
        guard_map: &'a mut HashMap<String, Mutex<()>>,
    ) -> MutexGuard<'a, ()> {
        guard_map.entry(name.to_string()).or_insert_with(|| Mutex::new(()));
        // Safety: we hold the outer HashMap lock while accessing the inner Mutex.
        // This is acceptable for a short-lived CRUD operation.
        guard_map[name].lock().expect("model lock poisoned")
    }

    pub fn list(&self) -> Result<Vec<ModelManifest>> {
        let models_dir = self.cfg.models_dir();
        let mut manifests = Vec::new();

        // Walk two levels: org/ and repo/
        for org_entry in fs::read_dir(&models_dir)?.flatten() {
            if !org_entry.file_type()?.is_dir() {
                continue;
            }
            for repo_entry in fs::read_dir(org_entry.path())?.flatten() {
                if !repo_entry.file_type()?.is_dir() {
                    continue;
                }
                let manifest_path = repo_entry.path().join(MANIFEST_FILE);
                if manifest_path.exists() {
                    let data = fs::read_to_string(&manifest_path)?;
                    let m: ModelManifest = serde_json::from_str(&data)?;
                    manifests.push(m);
                }
            }
        }

        Ok(manifests)
    }

    pub fn get_manifest(&self, name: &str) -> Result<ModelManifest> {
        let path = self.cfg.model_dir(name).join(MANIFEST_FILE);
        if !path.exists() {
            return Err(Error::ModelNotFound(name.to_string()));
        }
        let data = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn write_manifest(&self, manifest: &ModelManifest) -> Result<()> {
        let dir = self.cfg.model_dir(&manifest.name);
        fs::create_dir_all(&dir)?;
        let path = dir.join(MANIFEST_FILE);
        let data = serde_json::to_string(manifest)?;
        fs::write(&path, data)?;
        Ok(())
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let mut map = self.locks.lock().expect("locks poisoned");
        let _guard = self.lock_model(name, &mut map);
        let dir = self.cfg.model_dir(name);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }

    /// Remove all cached models. Returns the count of removed model directories.
    pub fn clean(&self) -> Result<i32> {
        let manifests = self.list()?;
        let mut count = 0i32;
        for m in manifests {
            self.remove(&m.name)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn get_model_type(&self, name: &str) -> Result<ModelType> {
        let m = self.get_manifest(name)?;
        Ok(m.model_type)
    }

    /// Resolve a model name (with optional ":quant" suffix) to ModelPaths.
    pub fn get_paths(&self, name_with_quant: &str) -> Result<(String, ModelPaths)> {
        let (name, quant) = split_quant(name_with_quant);
        let manifest = self.get_manifest(name)?;
        let base_dir = self.cfg.model_dir(name);
        resolve_model_paths(&manifest, &base_dir, quant.as_deref())
    }

    pub fn model_file_path(&self, name: &str, file: &str) -> PathBuf {
        self.cfg.model_file_path(name, file)
    }
}

/// Split "org/repo:quant" into ("org/repo", Some("quant")) or ("org/repo", None).
fn split_quant(s: &str) -> (&str, Option<String>) {
    if let Some(pos) = s.rfind(':') {
        let name = &s[..pos];
        let quant = &s[pos + 1..];
        if quant.is_empty() {
            (name, None)
        } else {
            (name, Some(quant.to_string()))
        }
    } else {
        (s, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ModelFileInfo, ModelType};
    use std::collections::HashMap;

    fn make_store() -> Store {
        // leak the TempDir so the directory persists for the test duration
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().to_path_buf();
        std::mem::forget(tmp);
        let cfg = StoreConfig::new(path, None);
        Store::new(cfg).unwrap()
    }

    fn sample_manifest(name: &str) -> ModelManifest {
        let mut model_file = HashMap::new();
        model_file.insert(
            "Q4_K_M".to_string(),
            ModelFileInfo { name: "model-Q4_K_M.gguf".to_string(), downloaded: true, size: 100 },
        );
        ModelManifest {
            name: name.to_string(),
            model_name: "test-1b".to_string(),
            model_type: ModelType::Llm,
            plugin_id: "llama_cpp".to_string(),
            device_id: String::new(),
            min_sdk_version: String::new(),
            model_file,
            mmproj_file: ModelFileInfo::default(),
            tokenizer_file: ModelFileInfo::default(),
            extra_files: vec![],
        }
    }

    #[test]
    fn roundtrip_manifest() {
        let store = make_store();
        let m = sample_manifest("TestOrg/TestRepo");
        store.write_manifest(&m).unwrap();
        let loaded = store.get_manifest("TestOrg/TestRepo").unwrap();
        assert_eq!(loaded.name, "TestOrg/TestRepo");
    }

    #[test]
    fn list_returns_written_manifests() {
        let store = make_store();
        store.write_manifest(&sample_manifest("Org/A")).unwrap();
        store.write_manifest(&sample_manifest("Org/B")).unwrap();
        let list = store.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn remove_deletes_directory() {
        let store = make_store();
        store.write_manifest(&sample_manifest("Org/C")).unwrap();
        store.remove("Org/C").unwrap();
        assert!(!store.cfg.model_dir("Org/C").exists());
    }

    #[test]
    fn clean_returns_count() {
        let store = make_store();
        store.write_manifest(&sample_manifest("Org/D")).unwrap();
        store.write_manifest(&sample_manifest("Org/E")).unwrap();
        let n = store.clean().unwrap();
        assert_eq!(n, 2);
    }
}
