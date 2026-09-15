//! Launcher scripts demanded from the wrapper.
//!
//! No launchers are baked into the image. The supervisor demands them over
//! the wrapper TCP link (`launchers.get` envelope), writes them to the
//! configured program paths, and launches. On repeated crashes it waits and
//! re-demands: a fixed portal-side script eventually arrives and the
//! programs start working.

use std::collections::HashMap;
use std::time::Duration;

/// Where to demand launchers from + where to write them.
#[derive(Debug, Clone)]
pub struct LauncherSource {
    pub address: String,
    pub port: u16,
    pub token: String,
    pub tenant_id: String,
    /// program name -> file path to write the received script to
    /// (from `[programs.*].binary` in config.toml).
    pub paths: HashMap<String, String>,
}

impl LauncherSource {
    /// Demand both launcher scripts once. Returns program -> (content,
    /// source). Errors when unreachable or when a program has no script
    /// anywhere (the caller retries: the supervisor must keep demanding).
    pub async fn fetch_once(&self) -> anyhow::Result<HashMap<String, (String, String)>> {
        let client = proxy::ProxyClient::new(&self.address, self.port, &self.token)?;
        let envelope = serde_json::json!({
            "type": "launchers.get",
            "tenant_id": self.tenant_id,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
            "payload": {"token": self.token},
        });
        let resp = client.send_json(envelope).await?;
        let scripts = resp
            .get("payload")
            .and_then(|p| p.get("scripts"))
            .and_then(|s| s.as_object())
            .ok_or_else(|| anyhow::anyhow!("launchers.get: no scripts in response"))?;
        let mut out = HashMap::new();
        for name in ["zw", "engram"] {
            let entry = scripts.get(name);
            let content = entry
                .and_then(|e| e.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            if content.trim().is_empty() {
                let source = entry
                    .and_then(|e| e.get("source"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("none");
                anyhow::bail!("no launcher script for '{name}' (source: {source})");
            }
            let source = entry
                .and_then(|e| e.get("source"))
                .and_then(|s| s.as_str())
                .unwrap_or("db")
                .to_string();
            out.insert(name.to_string(), (content.to_string(), source));
        }
        Ok(out)
    }

    /// Write fetched scripts to their configured paths (chmod +x).
    pub fn install(&self, scripts: &HashMap<String, (String, String)>) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        for (name, (content, _)) in scripts {
            let path = self.paths.get(name).ok_or_else(|| {
                anyhow::anyhow!("no output path configured for launcher '{name}'")
            })?;
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    }

    /// Demand scripts + binaries (version-gated). Boot and crash-loop entry
    /// point: returns only when scripts are installed AND binaries are up
    /// to date. Binaries live outside the workspace agent root so a binary
    /// replace never touches agent data.
    pub async fn demand_all(&self, telemetry: &telemetry::Telemetry) {
        self.demand_loop(telemetry).await;
        super::binaries::demand_binaries(
            &self.address,
            self.port,
            &self.token,
            &self.tenant_id,
            super::binaries::BINARIES_DIR,
            telemetry,
        )
        .await;
    }

    /// Demand + install, retrying forever (5s base backoff). Returns when
    /// both scripts are on disk. The supervisor calls this at boot and after
    /// repeated crashes: no scripts => no programs, never give up.
    pub async fn demand_loop(&self, telemetry: &telemetry::Telemetry) {
        let mut wait = Duration::from_secs(5);
        loop {
            match self.fetch_once().await {
                Ok(scripts) => match self.install(&scripts) {
                    Ok(()) => {
                        let sources: Vec<String> = scripts
                            .iter()
                            .map(|(n, (_, s))| format!("{n}={s}"))
                            .collect();
                        telemetry
                            .builder()
                            .level(telemetry::Level::Info)
                            .kind("launcher.installed")
                            .category("deployment")
                            .message(format!("launcher scripts installed ({})", sources.join(" ")))
                            .emit();
                        return;
                    }
                    Err(e) => {
                        telemetry
                            .builder()
                            .level(telemetry::Level::Warn)
                            .kind("launcher.install_failed")
                            .category("deployment")
                            .message(format!("launcher install failed: {e:#}"))
                            .emit();
                    }
                },
                Err(e) => {
                    telemetry
                        .builder()
                        .level(telemetry::Level::Warn)
                        .kind("launcher.demand_failed")
                        .category("deployment")
                        .message(format!("launcher demand failed, retrying in {wait:?}: {e:#}"))
                        .emit();
                }
            }
            tokio::time::sleep(wait).await;
            wait = std::cmp::min(wait * 2, Duration::from_secs(60));
        }
    }
}
