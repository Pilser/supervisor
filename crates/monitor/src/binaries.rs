//! On-demand guest binaries: version-gated fetch from the wrapper.
//!
//! The image carries NO zw/engram binaries or zw assets. On boot (and on
//! crash-loop redemand) the supervisor fetches a manifest, downloads only
//! changed files in 256KB base64 chunks, verifies sha256 and installs
//! atomically (tmp + rename — never overwrite a running binary). Local
//! versions live in `<dir>/versions.json`, so steady-state boots download
//! nothing.

use std::collections::HashMap;
use futures_util::StreamExt;

/// Guest dir holding demanded binaries + zw assets + versions.json.
pub const BINARIES_DIR: &str = "/data/supervisor/binaries";

fn local_versions(dir: &str) -> HashMap<String, String> {
    std::fs::read_to_string(format!("{dir}/versions.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_versions(dir: &str, versions: &HashMap<String, String>) {
    let _ = std::fs::create_dir_all(dir);
    if let Ok(s) = serde_json::to_string(versions) {
        let _ = std::fs::write(format!("{dir}/versions.json"), s);
    }
}

/// Fetch one file over HTTP (streamed to disk in 256KB pieces — a 92MB
/// engram never lands in guest RAM), then verify sha256 and install
/// atomically (tmp + rename: never overwrite a running binary).
async fn fetch_file(
    base: &str,
    token: &str,
    dir: &str,
    path: &str,
    sha: &str,
    size: u64,
) -> anyhow::Result<()> {
    let url = format!("{base}/api/supervisor/binaries/file?path={}", url_encode(path));
    let resp = reqwest::Client::new()
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("binaries file {path}: HTTP {}", resp.status());
    }
    let dst = format!("{dir}/{path}");
    if let Some(parent) = std::path::Path::new(&dst).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = format!("{dst}.dl-tmp");
    {
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::File::create(&tmp).await?;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            f.write_all(&chunk?).await?;
        }
        f.flush().await?;
    }
    let meta = tokio::fs::metadata(&tmp).await?;
    if meta.len() != size {
        let _ = tokio::fs::remove_file(&tmp).await;
        anyhow::bail!("size mismatch for {path}: got {}, want {size}", meta.len());
    }
    // Verify sha256 off the file, streaming (no full-file buffer).
    {
        use sha2::Digest;
        use tokio::io::AsyncReadExt;
        let mut f = tokio::fs::File::open(&tmp).await?;
        let mut hasher = sha2::Sha256::new();
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            let n = f.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let digest = format!("{:x}", hasher.finalize());
        if digest != sha {
            let _ = tokio::fs::remove_file(&tmp).await;
            anyhow::bail!("sha256 mismatch for {path}");
        }
    }
    if !path.contains('/') {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).await?;
    }
    tokio::fs::rename(&tmp, &dst).await?;
    Ok(())
}

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' | '/' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

/// Local stamp says every file is present: skip without touching the
/// network. Binaries move ONLY on admin push (which deletes the stamp) or
/// first boot (no stamp yet) — never on their own, so the host stays idle.
fn stamp_complete(dir: &str) -> bool {
    let versions: HashMap<String, String> = local_versions(dir);
    if versions.is_empty() {
        return false;
    }
    versions
        .keys()
        .all(|p| std::path::Path::new(&format!("{dir}/{p}")).exists())
}

/// Demand changed binaries (version-gated). Retries forever like launchers.
pub async fn demand_binaries(
    http_base: &str,
    token: &str,
    dir: &str,
    telemetry: &telemetry::Telemetry,
) {
    // Binaries are served by the wrapper's public HTTP listener: plain
    // streaming, no NDJSON/base64 framing, no in-RAM assembly.
    if stamp_complete(dir) {
        telemetry
            .builder()
            .level(telemetry::Level::Info)
            .kind("binaries.skipped")
            .category("deployment")
            .message("guest binaries present, no admin push pending".to_string())
            .emit();
        return;
    }
    let mut wait = std::time::Duration::from_secs(5);
    loop {
        match sync_once(http_base, token, dir, telemetry).await {
            Ok(0) => {
                telemetry
                    .builder()
                    .level(telemetry::Level::Info)
                    .kind("binaries.uptodate")
                    .category("deployment")
                    .message("guest binaries up to date".to_string())
                    .emit();
                return;
            }
            Ok(n) => {
                telemetry
                    .builder()
                    .level(telemetry::Level::Info)
                    .kind("binaries.installed")
                    .category("deployment")
                    .message(format!("installed {n} guest binaries"))
                    .emit();
                return;
            }
            Err(e) => {
                telemetry
                    .builder()
                    .level(telemetry::Level::Warn)
                    .kind("binaries.demand_failed")
                    .category("deployment")
                    .message(format!("binaries demand failed, retrying in {wait:?}: {e:#}"))
                    .emit();
            }
        }
        tokio::time::sleep(wait).await;
        wait = std::cmp::min(wait * 2, std::time::Duration::from_secs(60));
    }
}

async fn sync_once(
    base: &str,
    token: &str,
    dir: &str,
    telemetry: &telemetry::Telemetry,
) -> anyhow::Result<usize> {
    let _ = telemetry;
    let resp = reqwest::Client::new()
        .get(format!("{base}/api/supervisor/binaries/manifest"))
        .bearer_auth(token)
        .send()
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("manifest: HTTP {}", resp.status());
    }
    let body: serde_json::Value = resp.json().await?;
    let files = body
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or_else(|| anyhow::anyhow!("manifest: no files"))?;
    let mut local = local_versions(dir);
    let mut n = 0usize;
    for f in files {
        let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let sha = f.get("sha256").and_then(|v| v.as_str()).unwrap_or("");
        let size = f.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
        if path.is_empty() || sha.is_empty() {
            continue;
        }
        if local.get(path).map(|s| s.as_str()) == Some(sha) {
            continue;
        }
        fetch_file(base, token, dir, path, sha, size).await?;
        local.insert(path.to_string(), sha.to_string());
        n += 1;
    }
    save_versions(dir, &local);
    Ok(n)
}
