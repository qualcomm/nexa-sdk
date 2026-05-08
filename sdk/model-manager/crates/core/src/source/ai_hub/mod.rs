//! Qualcomm AI Hub [`ModelSource`].
//!
//! Resolves the asset URL for a `(display_name, chipset)` pair via the
//! public S3 protojson chain, then reads the remote archive's ZIP64
//! central directory over HTTP Range reads to produce a complete
//! [`Plan`] — manifest + one per-entry [`BytesSource`] — without
//! downloading the 4 GB+ payload.
//!
//! See `source::ai_hub::remote_zip` for the CD parser. See
//! `source::ai_hub::selector` for the chipset matcher lifted verbatim
//! from the old `hub::s3` code.

pub mod cache;
pub mod detect;
pub mod manifest;
pub mod remote_zip;
pub mod selector;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use url::Url;

use crate::error::{Error, Result};
use crate::manifest::{ModelFileInfo, ModelManifest, ModelType};
use crate::transport::{HttpTransport, ReqwestTransport};

use self::manifest::{ModelReleaseAssets, PlatformInfo, ReleaseManifest};
use self::remote_zip::{fetch_central_directory, Method, ZipEntry};
use self::selector::{match_asset, UnavailableChipset};

use super::{BytesSource, FileSpec, ModelSource, Plan};

const MANIFEST_FILENAME: &str = "manifest.json";
const PLATFORM_FILENAME: &str = "platform.json";
const MAX_INDEX_BYTES: u64 = 8 * 1024 * 1024;

/// Config knobs for the AI Hub source. Mirrors today's `S3Config` but
/// is owned by the source itself rather than a detached pull function.
#[derive(Debug, Clone)]
pub struct AiHubConfig {
    pub endpoint: String,
    pub version: String,
    /// Empty → auto-detect via [`detect::detect_host_chipset`].
    pub chipset: String,
    pub cache_dir: PathBuf,
    pub skip_cache: bool,
}

impl AiHubConfig {
    pub fn new(
        endpoint: impl Into<String>,
        version: impl Into<String>,
        chipset: impl Into<String>,
        cache_dir: PathBuf,
        skip_cache: bool,
    ) -> Self {
        let endpoint: String = endpoint.into().trim_end_matches('/').to_string();
        let version: String = version.into().trim_matches('/').to_string();
        Self {
            endpoint,
            version,
            chipset: chipset.into(),
            cache_dir,
            skip_cache,
        }
    }
}

pub struct AiHubSource {
    display_name: String,
    model_name: String,
    cfg: AiHubConfig,
    transport: Arc<dyn HttpTransport>,
}

impl AiHubSource {
    pub fn new(display_name: String, model_name: String, cfg: AiHubConfig) -> Result<Self> {
        let transport: Arc<dyn HttpTransport> = Arc::new(ReqwestTransport::new()?);
        Ok(Self {
            display_name,
            model_name,
            cfg,
            transport,
        })
    }

    pub fn with_transport(
        display_name: String,
        model_name: String,
        cfg: AiHubConfig,
        transport: Arc<dyn HttpTransport>,
    ) -> Self {
        Self {
            display_name,
            model_name,
            cfg,
            transport,
        }
    }
}

#[async_trait]
impl ModelSource for AiHubSource {
    async fn plan(&self) -> Result<Plan> {
        let endpoint = self.cfg.endpoint.as_str();
        let version = self.cfg.version.as_str();

        // 1. manifest.json → pick the entry we care about.
        let manifest_url = format!("{endpoint}/releases/{version}/{MANIFEST_FILENAME}");
        let manifest_cache = self.cfg.cache_dir.join(MANIFEST_FILENAME);
        let manifest_bytes = fetch_with_cache(
            &manifest_url,
            &manifest_cache,
            self.cfg.skip_cache,
            &self.transport,
        )
        .await?;
        let release_manifest: ReleaseManifest =
            serde_json::from_slice(&manifest_bytes).map_err(|source| Error::ManifestParse {
                what: "manifest.json",
                source,
            })?;

        let entry = release_manifest
            .models
            .iter()
            .find(|m| m.display_name == self.display_name)
            .ok_or_else(|| {
                Error::Hub(format!(
                    "model {:?} not found in AI Hub manifest",
                    self.display_name
                ))
            })?;

        if !selector::is_domain_supported(&entry.domain) {
            return Err(Error::Hub(format!(
                "AI Hub model {:?} has unsupported domain {:?}",
                self.display_name, entry.domain
            )));
        }

        let release_assets_url = &entry.manifest_urls.release_assets;
        if release_assets_url.is_empty() {
            return Err(Error::Hub(format!(
                "AI Hub model {:?} has no release_assets URL",
                self.display_name
            )));
        }

        // 2. release-assets.json (per-model, URL rotates each release).
        let release_assets_bytes = fetch_direct(release_assets_url, &self.transport).await?;
        let release_assets: ModelReleaseAssets = serde_json::from_slice(&release_assets_bytes)
            .map_err(|source| Error::ManifestParse {
                what: "release-assets.json",
                source,
            })?;

        // 3. platform.json → alias resolution.
        let platform_url = format!("{endpoint}/releases/{version}/{PLATFORM_FILENAME}");
        let platform_cache = self.cfg.cache_dir.join(PLATFORM_FILENAME);
        let platform_bytes = fetch_with_cache(
            &platform_url,
            &platform_cache,
            self.cfg.skip_cache,
            &self.transport,
        )
        .await?;
        let platform: PlatformInfo =
            serde_json::from_slice(&platform_bytes).map_err(|source| Error::ManifestParse {
                what: "platform.json",
                source,
            })?;

        // 4. Resolve chipset (possibly via host auto-detect).
        let chipset: String = if self.cfg.chipset.is_empty() {
            detect::detect_host_chipset().ok_or_else(|| {
                Error::Hub(
                    "chipset not provided and host auto-detect is not supported on this platform"
                        .to_string(),
                )
            })?
        } else {
            self.cfg.chipset.clone()
        };

        let asset = match match_asset(&release_assets, &platform, &chipset) {
            Ok(a) => a,
            Err(UnavailableChipset {
                requested,
                available,
            }) => {
                return Err(Error::ChipsetUnavailable {
                    requested,
                    available,
                });
            }
        };

        let download_url = Url::parse(&asset.download_url).map_err(|e| {
            Error::Hub(format!(
                "invalid asset download_url {:?}: {e}",
                asset.download_url
            ))
        })?;

        // 5. Pull the remote central directory. No payload fetched yet.
        let raw_entries = fetch_central_directory(&self.transport, &download_url).await?;

        // 6. Filter + flat-extract-name planning. Mirrors the old
        //    `extract::extract_flat` pre-scan: skip dirs, __MACOSX and
        //    ._* metadata, and reject basename collisions up front.
        let flat_entries = prepare_flat_entries(&raw_entries)?;

        // 7. Pick the entrypoint — lex-first `.bin` shard, same as the
        //    Go CLI's ExtractFlat.
        let entrypoint_name = flat_entries
            .iter()
            .find(|(name, _)| name.to_ascii_lowercase().ends_with(".bin"))
            .map(|(name, _)| name.clone())
            .ok_or_else(|| Error::Hub("no .bin shard in archive".to_string()))?;

        // 8. Synthesise the qairt manifest.
        let precision_label = asset
            .precision
            .strip_prefix("PRECISION_")
            .unwrap_or(&asset.precision)
            .to_string();

        let total_uncompressed: u64 = flat_entries.iter().map(|(_, e)| e.uncompressed_size).sum();

        let mut model_file: HashMap<String, ModelFileInfo> = HashMap::new();
        model_file.insert(
            "N/A".to_string(),
            ModelFileInfo {
                name: entrypoint_name.clone(),
                downloaded: true,
                size: total_uncompressed as i64,
            },
        );
        let extra_files: Vec<ModelFileInfo> = flat_entries
            .iter()
            .filter(|(name, _)| name != &entrypoint_name)
            .map(|(name, e)| ModelFileInfo {
                name: name.clone(),
                downloaded: true,
                size: e.uncompressed_size as i64,
            })
            .collect();

        let model_type = if entry.domain == "MODEL_DOMAIN_MULTIMODAL" {
            ModelType::Vlm
        } else {
            ModelType::Llm
        };
        let manifest = ModelManifest {
            name: self.model_name.clone(),
            model_name: if entry.id.is_empty() {
                self.display_name.clone()
            } else {
                entry.id.clone()
            },
            model_type,
            plugin_id: "qairt".to_string(),
            precision: precision_label,
            model_file,
            mmproj_file: ModelFileInfo::default(),
            tokenizer_file: ModelFileInfo::default(),
            extra_files,
        };

        // 9. Emit one FileSpec per flat entry, picking BytesSource by
        //    compression method.
        let files: Vec<FileSpec> = flat_entries
            .into_iter()
            .map(|(name, e)| {
                let bytes = match e.method {
                    Method::Stored => BytesSource::HttpRange {
                        url: download_url.clone(),
                        auth: None,
                        offset: e.payload_offset,
                        len: e.compressed_size,
                    },
                    Method::Deflate => BytesSource::HttpDeflate {
                        url: download_url.clone(),
                        auth: None,
                        offset: e.payload_offset,
                        compressed_len: e.compressed_size,
                    },
                };
                FileSpec {
                    name,
                    size: e.uncompressed_size,
                    bytes,
                }
            })
            .collect();

        Ok(Plan { manifest, files })
    }
}

/// Reduce the raw central-directory entries to a flat list of
/// `(basename, entry)` pairs, skipping directories + macOS AppleDouble
/// metadata and rejecting basename collisions.
///
/// Returns entries in stable central-directory order so progress is
/// predictable; the caller can sort after if needed.
fn prepare_flat_entries(raw: &[ZipEntry]) -> Result<Vec<(String, ZipEntry)>> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<(String, ZipEntry)> = Vec::new();
    for e in raw {
        if e.is_dir {
            continue;
        }
        if is_macos_metadata(&e.name) {
            continue;
        }
        let base = basename(&e.name);
        if base.is_empty() || base == "." || base == ".." {
            continue;
        }
        if !seen.insert(base.clone()) {
            return Err(Error::Hub(format!(
                "duplicate basename {base:?} in archive (from entry {:?})",
                e.name
            )));
        }
        out.push((base, e.clone()));
    }
    Ok(out)
}

fn basename(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or("").to_string()
}

fn is_macos_metadata(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with("__MACOSX/") || normalized.contains("/__MACOSX/") {
        return true;
    }
    basename(&normalized).starts_with("._")
}

async fn fetch_with_cache(
    url: &str,
    cache_path: &Path,
    skip_cache: bool,
    transport: &Arc<dyn HttpTransport>,
) -> Result<Vec<u8>> {
    if !skip_cache {
        if let Some(bytes) = cache::read_if_fresh(cache_path, cache::DEFAULT_TTL) {
            return Ok(bytes);
        }
    }
    let bytes = fetch_direct(url, transport).await?;
    if !skip_cache {
        cache::write(cache_path, &bytes);
    }
    Ok(bytes)
}

async fn fetch_direct(url: &str, transport: &Arc<dyn HttpTransport>) -> Result<Vec<u8>> {
    let parsed = Url::parse(url).map_err(|e| Error::Hub(format!("invalid url {url:?}: {e}")))?;
    let head = transport.head(&parsed, None).await?;
    if head.size > MAX_INDEX_BYTES {
        return Err(Error::Hub(format!(
            "index at {url} is {} bytes, exceeds {MAX_INDEX_BYTES}-byte cap",
            head.size
        )));
    }
    let mut buf: Vec<u8> = Vec::with_capacity(head.size as usize);
    transport
        .get_range(&parsed, None, 0, head.size, &mut buf)
        .await?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_basename_collision() {
        let raw = vec![
            ZipEntry {
                name: "a/model.bin".to_string(),
                method: Method::Stored,
                payload_offset: 0,
                compressed_size: 1,
                uncompressed_size: 1,
                is_dir: false,
            },
            ZipEntry {
                name: "b/model.bin".to_string(),
                method: Method::Stored,
                payload_offset: 100,
                compressed_size: 1,
                uncompressed_size: 1,
                is_dir: false,
            },
        ];
        assert!(prepare_flat_entries(&raw).is_err());
    }

    #[test]
    fn skips_macos_metadata() {
        let raw = vec![
            ZipEntry {
                name: "dir/model.bin".to_string(),
                method: Method::Stored,
                payload_offset: 0,
                compressed_size: 2,
                uncompressed_size: 2,
                is_dir: false,
            },
            ZipEntry {
                name: "__MACOSX/dir/._model.bin".to_string(),
                method: Method::Stored,
                payload_offset: 100,
                compressed_size: 4,
                uncompressed_size: 4,
                is_dir: false,
            },
        ];
        let out = prepare_flat_entries(&raw).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "model.bin");
    }
}
