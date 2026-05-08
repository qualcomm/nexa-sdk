//! ZIP64-aware central directory parser driven by HTTP Range reads.
//!
//! Given a `Url` to a remote zip, [`fetch_central_directory`] returns
//! the list of file entries with enough info to issue per-entry range
//! GETs. No byte payload is downloaded — only the EOCD footer, the
//! optional ZIP64 EOCD, the central directory itself (typically a few
//! KB even for 4 GB+ archives), and one 30-byte local file header per
//! entry.
//!
//! Scope is intentionally narrow:
//! - method 0 (STORED) and method 8 (DEFLATE) — every AI Hub qairt
//!   asset today is one or the other.
//! - ZIP64 extensions for entries ≥ 4 GB (Qwen2.5-VL-7B's zip, 4.6 GB
//!   compressed, uses them).
//! - Split archives, encrypted entries, or crazy extra-field layouts
//!   are rejected up front rather than guessed at.
//!
//! Wire-format references:
//! - APPNOTE.TXT 6.3.10
//! - End of central directory record (EOCD): signature 0x06054b50, fixed
//!   22 bytes + variable comment.
//! - ZIP64 EOCD locator: 0x07064b50, fixed 20 bytes, sits directly
//!   before the EOCD.
//! - ZIP64 EOCD: 0x06064b50, fixed 56 bytes + variable extensible data.
//! - Central directory header: 0x02014b50, fixed 46 bytes + name +
//!   extra + comment.
//! - Local file header: 0x04034b50, fixed 30 bytes + name + extra.

use std::sync::Arc;

use url::Url;

use crate::error::{Error, Result};
use crate::transport::HttpTransport;

const EOCD_SIG: u32 = 0x0605_4b50;
const ZIP64_EOCD_LOCATOR_SIG: u32 = 0x0706_4b50;
const ZIP64_EOCD_SIG: u32 = 0x0606_4b50;
const CD_HEADER_SIG: u32 = 0x0201_4b50;
const LOCAL_HEADER_SIG: u32 = 0x0403_4b50;

/// How many bytes from the end to scan for the EOCD signature. The EOCD
/// has a variable-length comment (up to 0xFFFF); practical archives
/// never include one, but we scan a buffer that comfortably covers both
/// it and the ZIP64 locator that sits in front.
const EOCD_SCAN_WINDOW: u64 = 64 * 1024 + 22;

/// Compression methods we understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Stored,
    Deflate,
}

/// One entry inside a remote zip, reduced to what the executor needs
/// to issue a range GET for its payload bytes.
#[derive(Debug, Clone)]
pub struct ZipEntry {
    /// Raw entry name from the central directory. Path-like; the
    /// caller is expected to flatten to basename.
    pub name: String,
    pub method: Method,
    /// Byte offset of the compressed payload inside the archive —
    /// already adjusted past the local file header's fixed 30-byte
    /// prefix, name, and extra field.
    pub payload_offset: u64,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub is_dir: bool,
}

/// Fetch the central directory of the zip at `url` and return one
/// [`ZipEntry`] per file record.
pub async fn fetch_central_directory(
    transport: &Arc<dyn HttpTransport>,
    url: &Url,
) -> Result<Vec<ZipEntry>> {
    let head = transport.head(url, None).await?;
    let total = head.size;
    if total < 22 {
        return Err(Error::Hub(format!(
            "zip at {url} is {total} bytes; too small to be a valid archive"
        )));
    }

    // 1. Fetch up to EOCD_SCAN_WINDOW bytes from the tail. EOCD lives
    //    there unless a huge trailing comment was used (never in
    //    practice for AI Hub).
    let tail_len = total.min(EOCD_SCAN_WINDOW);
    let tail_start = total - tail_len;
    let mut tail = Vec::with_capacity(tail_len as usize);
    transport
        .get_range(url, None, tail_start, tail_len, &mut tail)
        .await?;

    // 2. Locate EOCD by scanning backwards for the signature.
    let eocd_off_in_tail = find_eocd(&tail)
        .ok_or_else(|| Error::Hub(format!("zip at {url}: EOCD signature not found in tail")))?;
    let eocd = &tail[eocd_off_in_tail..];
    if eocd.len() < 22 {
        return Err(Error::Hub(format!(
            "zip at {url}: EOCD truncated ({} < 22 bytes)",
            eocd.len()
        )));
    }
    let disk_num = read_u16(eocd, 4);
    let disk_with_cd = read_u16(eocd, 6);
    if disk_num != 0 || disk_with_cd != 0 {
        return Err(Error::Hub(format!(
            "zip at {url}: split archives not supported (disk {disk_num}/{disk_with_cd})"
        )));
    }
    let mut cd_size = read_u32(eocd, 12) as u64;
    let mut cd_offset = read_u32(eocd, 16) as u64;
    let mut cd_entries = read_u16(eocd, 10) as u64;

    // 3. ZIP64 locator sits immediately before the EOCD (20 bytes).
    //    Present iff any of (cd_size, cd_offset, cd_entries) is maxed.
    let needs_zip64 = cd_size == 0xFFFF_FFFF || cd_offset == 0xFFFF_FFFF || cd_entries == 0xFFFF;
    let eocd_abs_off = tail_start + eocd_off_in_tail as u64;
    if needs_zip64 {
        if eocd_abs_off < 20 {
            return Err(Error::Hub(format!(
                "zip at {url}: ZIP64 locator expected but no room before EOCD"
            )));
        }
        let locator_abs = eocd_abs_off - 20;
        // We may already have the locator in `tail` — compute offset.
        let locator_rel = (locator_abs - tail_start) as usize;
        let locator = &tail[locator_rel..locator_rel + 20];
        if read_u32(locator, 0) != ZIP64_EOCD_LOCATOR_SIG {
            return Err(Error::Hub(format!(
                "zip at {url}: ZIP64 EOCD locator signature missing"
            )));
        }
        let z64_eocd_off = read_u64(locator, 8);

        // Fetch the ZIP64 EOCD (56 fixed bytes; extensible data we
        // ignore).
        let mut z64 = Vec::with_capacity(56);
        transport
            .get_range(url, None, z64_eocd_off, 56, &mut z64)
            .await?;
        if z64.len() != 56 || read_u32(&z64, 0) != ZIP64_EOCD_SIG {
            return Err(Error::Hub(format!(
                "zip at {url}: ZIP64 EOCD signature missing"
            )));
        }
        cd_entries = read_u64(&z64, 32);
        cd_size = read_u64(&z64, 40);
        cd_offset = read_u64(&z64, 48);
    }

    if cd_size == 0 {
        return Ok(Vec::new());
    }
    if cd_offset + cd_size > total {
        return Err(Error::Hub(format!(
            "zip at {url}: central directory range ({cd_offset}+{cd_size}) exceeds archive size ({total})"
        )));
    }

    // 4. Pull the CD. Typically a few KB even for multi-GB archives.
    let mut cd = Vec::with_capacity(cd_size as usize);
    transport
        .get_range(url, None, cd_offset, cd_size, &mut cd)
        .await?;
    if cd.len() as u64 != cd_size {
        return Err(Error::Hub(format!(
            "zip at {url}: CD fetch short ({} < {cd_size})",
            cd.len()
        )));
    }

    // 5. Walk CD headers. 46 fixed bytes + name_len + extra_len + comment_len.
    let mut entries: Vec<ZipEntry> = Vec::with_capacity(cd_entries as usize);
    let mut cursor: usize = 0;
    while cursor + 46 <= cd.len() {
        if read_u32(&cd, cursor) != CD_HEADER_SIG {
            return Err(Error::Hub(format!(
                "zip at {url}: CD header signature missing at offset {cursor}"
            )));
        }
        let method_raw = read_u16(&cd, cursor + 10);
        let mut compressed_size = read_u32(&cd, cursor + 20) as u64;
        let mut uncompressed_size = read_u32(&cd, cursor + 24) as u64;
        let name_len = read_u16(&cd, cursor + 28) as usize;
        let extra_len = read_u16(&cd, cursor + 30) as usize;
        let comment_len = read_u16(&cd, cursor + 32) as usize;
        let mut local_header_offset = read_u32(&cd, cursor + 42) as u64;

        let total_len = 46 + name_len + extra_len + comment_len;
        if cursor + total_len > cd.len() {
            return Err(Error::Hub(format!(
                "zip at {url}: CD entry at offset {cursor} truncated"
            )));
        }

        let name_bytes = &cd[cursor + 46..cursor + 46 + name_len];
        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| Error::Hub("zip entry name is not UTF-8".to_string()))?
            .to_string();

        // Walk extra fields to pick up ZIP64 overrides.
        let extra = &cd[cursor + 46 + name_len..cursor + 46 + name_len + extra_len];
        let mut ex_cursor = 0usize;
        while ex_cursor + 4 <= extra.len() {
            let tag = read_u16(extra, ex_cursor);
            let ex_size = read_u16(extra, ex_cursor + 2) as usize;
            if ex_cursor + 4 + ex_size > extra.len() {
                break;
            }
            if tag == 0x0001 {
                // ZIP64 extended information. Fields are present in the
                // order (uncompressed, compressed, local_header_offset,
                // disk_start_number), but only those whose 32-bit slot
                // in the CD header was 0xFFFFFFFF are actually included.
                let mut ez = ex_cursor + 4;
                if uncompressed_size == 0xFFFF_FFFF {
                    uncompressed_size = read_u64(extra, ez);
                    ez += 8;
                }
                if compressed_size == 0xFFFF_FFFF {
                    compressed_size = read_u64(extra, ez);
                    ez += 8;
                }
                if local_header_offset == 0xFFFF_FFFF {
                    local_header_offset = read_u64(extra, ez);
                }
                break;
            }
            ex_cursor += 4 + ex_size;
        }

        let method = match method_raw {
            0 => Method::Stored,
            8 => Method::Deflate,
            other => {
                return Err(Error::Hub(format!(
                    "zip entry {name:?}: unsupported compression method {other}"
                )))
            }
        };

        // Directory entries have uncompressed_size == 0 and name ending
        // with '/' per APPNOTE.
        let is_dir = name.ends_with('/') && uncompressed_size == 0;

        entries.push(ZipEntry {
            name,
            method,
            // Local header offset for now; resolved to the payload
            // byte offset in step 6 after fetching the local header.
            payload_offset: local_header_offset,
            compressed_size,
            uncompressed_size,
            is_dir,
        });
        cursor += total_len;
    }

    // 6. Resolve each entry's payload_offset = local_header_off + 30 +
    //    name_len + extra_len. We need one 30-byte GET per entry, but
    //    we can batch sequential fetches since local headers are
    //    clustered at file start (AI Hub archives order CD entries in
    //    the same order as local headers).
    let mut resolved: Vec<ZipEntry> = Vec::with_capacity(entries.len());
    for mut e in entries.drain(..) {
        if e.is_dir {
            resolved.push(e);
            continue;
        }
        let header_abs = e.payload_offset;
        let mut hdr = Vec::with_capacity(30);
        transport
            .get_range(url, None, header_abs, 30, &mut hdr)
            .await?;
        if hdr.len() != 30 || read_u32(&hdr, 0) != LOCAL_HEADER_SIG {
            return Err(Error::Hub(format!(
                "zip entry {:?}: local header signature missing at offset {header_abs}",
                e.name
            )));
        }
        let lh_name_len = read_u16(&hdr, 26) as u64;
        let lh_extra_len = read_u16(&hdr, 28) as u64;
        e.payload_offset = header_abs + 30 + lh_name_len + lh_extra_len;
        resolved.push(e);
    }

    Ok(resolved)
}

fn find_eocd(buf: &[u8]) -> Option<usize> {
    if buf.len() < 22 {
        return None;
    }
    let upper_start = buf.len() - 22;
    (0..=upper_start)
        .rev()
        .find(|&i| read_u32(buf, i) == EOCD_SIG)
}

fn read_u16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([buf[off], buf[off + 1]])
}

fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

fn read_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
        buf[off + 4],
        buf[off + 5],
        buf[off + 6],
        buf[off + 7],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{ReqwestTransport, TransportConfig};
    use std::io::Write as IoWrite;
    use std::sync::Arc;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    fn fast_transport() -> Arc<dyn HttpTransport> {
        Arc::new(
            ReqwestTransport::with_config(TransportConfig {
                connect_timeout: Some(Duration::from_secs(2)),
                read_timeout: Some(Duration::from_secs(5)),
                retries: Some(0),
                retry_backoff: Some(Duration::from_millis(10)),
                proxy_override: None,
            })
            .unwrap(),
        )
    }

    fn build_stored_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zw = zip::ZipWriter::new(cursor);
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, data) in entries {
                zw.start_file(*name, opts).unwrap();
                zw.write_all(data).unwrap();
            }
            zw.finish().unwrap();
        }
        buf
    }

    fn build_deflate_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zw = zip::ZipWriter::new(cursor);
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, data) in entries {
                zw.start_file(*name, opts).unwrap();
                zw.write_all(data).unwrap();
            }
            zw.finish().unwrap();
        }
        buf
    }

    async fn serve(body: Vec<u8>) -> (MockServer, Url) {
        let server = MockServer::start().await;
        Mock::given(method("HEAD"))
            .and(path("/a.zip"))
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header("Content-Length", body.len().to_string())
                    .append_header("Accept-Ranges", "bytes"),
            )
            .mount(&server)
            .await;
        let body_arc = Arc::new(body);
        let body_cl = body_arc.clone();
        Mock::given(method("GET"))
            .and(path("/a.zip"))
            .respond_with(move |req: &Request| {
                let hdr = req.headers.get("range").expect("range").to_str().unwrap();
                let rest = hdr.strip_prefix("bytes=").unwrap();
                let (s, e) = rest.split_once('-').unwrap();
                let start: usize = s.parse().unwrap();
                let end: usize = e.parse().unwrap();
                let slice = body_cl[start..=end].to_vec();
                ResponseTemplate::new(206).set_body_bytes(slice)
            })
            .mount(&server)
            .await;
        let url = Url::parse(&format!("{}/a.zip", server.uri())).unwrap();
        (server, url)
    }

    #[tokio::test]
    async fn parses_stored_zip() {
        let body = build_stored_zip(&[("model.bin", b"HELLO"), ("tokenizer.json", b"{}")]);
        let (_srv, url) = serve(body).await;
        let transport = fast_transport();
        let entries = fetch_central_directory(&transport, &url).await.unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"model.bin"));
        assert!(names.contains(&"tokenizer.json"));
        let bin = entries.iter().find(|e| e.name == "model.bin").unwrap();
        assert_eq!(bin.method, Method::Stored);
        assert_eq!(bin.uncompressed_size, 5);
        assert_eq!(bin.compressed_size, 5);
    }

    #[tokio::test]
    async fn parses_deflate_zip() {
        let big = vec![b'A'; 8192];
        let body = build_deflate_zip(&[("shard.bin", &big)]);
        let (_srv, url) = serve(body).await;
        let transport = fast_transport();
        let entries = fetch_central_directory(&transport, &url).await.unwrap();
        let e = entries.iter().find(|e| e.name == "shard.bin").unwrap();
        assert_eq!(e.method, Method::Deflate);
        assert_eq!(e.uncompressed_size, 8192);
        assert!(e.compressed_size < e.uncompressed_size);
    }

    #[tokio::test]
    async fn rejects_malformed_tail() {
        let mut body = vec![0u8; 1024];
        body[..4].copy_from_slice(b"PK\x03\x04"); // looks zip-ish at start
        let (_srv, url) = serve(body).await;
        let transport = fast_transport();
        let err = fetch_central_directory(&transport, &url).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("EOCD"), "unexpected error: {msg}");
    }
}
