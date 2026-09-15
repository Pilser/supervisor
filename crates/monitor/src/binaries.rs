//! On-demand guest binaries: version-gated fetch from the wrapper.
//!
//! The image carries NO zw/engram binaries or zw assets. On boot (and on
//! crash-loop redemand) the supervisor fetches a manifest, downloads only
//! changed files in 256KB base64 chunks, verifies sha256 and installs
//! atomically (tmp + rename — never overwrite a running binary). Local
//! versions live in `<dir>/versions.json`, so steady-state boots download
//! nothing.

use std::collections::HashMap;

/// Guest dir holding demanded binaries + zw assets + versions.json.
pub const BINARIES_DIR: &str = "/data/supervisor/binaries";

fn decode_b64(s: &str) -> anyhow::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut buf: u32 = 0;
    let mut nbits = 0;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => anyhow::bail!("bad base64 char"),
        } as u32;
        buf = (buf << 6) | v;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            out.push((buf >> nbits) as u8 & 0xFF);
        }
    }
    Ok(out)
}

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

/// Fetch one file in chunks, verify sha256, install atomically (+x for
/// top-level binaries).
async fn fetch_file(
    address: &str,
    port: u16,
    token: &str,
    tenant_id: &str,
    dir: &str,
    path: &str,
    sha: &str,
    size: u64,
) -> anyhow::Result<()> {
    let client = proxy::ProxyClient::new(address, port, token)?;
    let mut data = Vec::with_capacity(size as usize);
    let mut offset: u64 = 0;
    loop {
        let envelope = serde_json::json!({
            "type": "binaries.get",
            "tenant_id": tenant_id,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
            "payload": {"token": token, "path": path, "offset": offset, "len": 262144},
        });
        let resp = client.send_json(envelope).await?;
        let payload = resp
            .get("payload")
            .ok_or_else(|| anyhow::anyhow!("binaries.get: no payload"))?;
        let chunk = payload
            .get("data_base64")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("binaries.get: no data"))?;
        let bytes = decode_b64(chunk)?;
        if bytes.is_empty() {
            break;
        }
        data.extend_from_slice(&bytes);
        offset += bytes.len() as u64;
        if (data.len() as u64) >= size {
            break;
        }
    }
    if data.len() as u64 != size {
        anyhow::bail!("size mismatch for {path}: got {}, want {size}", data.len());
    }
    {
        use sha2::Digest;
        let digest = format!("{:x}", sha2::Sha256::digest(&data));
        if digest != sha {
            anyhow::bail!("sha256 mismatch for {path}");
        }
    }
    let dst = format!("{dir}/{path}");
    if let Some(parent) = std::path::Path::new(&dst).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = format!("{dst}.dl-tmp");
    std::fs::write(&tmp, &data)?;
    if !path.contains('/') {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }
    std::fs::rename(&tmp, &dst)?;
    Ok(())
}

/// Demand changed binaries (version-gated). Retries forever like launchers.
pub async fn demand_binaries(
    address: &str,
    port: u16,
    token: &str,
    tenant_id: &str,
    dir: &str,
    telemetry: &telemetry::Telemetry,
) {
    let mut wait = std::time::Duration::from_secs(5);
    loop {
        match sync_once(address, port, token, tenant_id, dir, telemetry).await {
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
                    .message(format!("installed {n} guest binaries".to_string()))
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
    address: &str,
    port: u16,
    token: &str,
    tenant_id: &str,
    dir: &str,
    telemetry: &telemetry::Telemetry,
) -> anyhow::Result<usize> {
    let _ = telemetry;
    let client = proxy::ProxyClient::new(address, port, token)?;
    let envelope = serde_json::json!({
        "type": "binaries.manifest",
        "tenant_id": tenant_id,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default(),
        "payload": {"token": token},
    });
    let resp = client.send_json(envelope).await?;
    let files = resp
        .get("payload")
        .and_then(|p| p.get("files"))
        .and_then(|f| f.as_array())
        .ok_or_else(|| anyhow::anyhow!("binaries.manifest: no files"))?;
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
        fetch_file(address, port, token, tenant_id, dir, path, sha, size).await?;
        local.insert(path.to_string(), sha.to_string());
        n += 1;
    }
    save_versions(dir, &local);
    Ok(n)
}
