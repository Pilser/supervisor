//! Buffers events and appends them to a per-day JSON-lines log.

use std::path::PathBuf;

use chrono::Datelike;

use super::{buffer::FloatBuffer, rotation::Rotation};
use crate::config::DiskSettings;
use crate::models::Event;

pub struct FileWriter {
    settings: DiskSettings,
    buffer: FloatBuffer,
    rotation: Rotation,
    current_date: Option<String>,
}

impl FileWriter {
    pub fn new(settings: DiskSettings) -> Self {
        Self {
            settings,
            buffer: FloatBuffer::new(),
            rotation: Rotation::new(),
            current_date: None,
        }
    }

    pub async fn write(&mut self, ev: &Event) -> anyhow::Result<()> {
        let date = timestamp_date(&ev.timestamp);
        if self.current_date.as_deref() != Some(date.as_str()) {
            self.flush().await?;
            self.rotation.retain(&self.settings, &date)?;
            self.current_date = Some(date);
        }
        self.buffer.push(ev.clone());
        if self.buffer.len() >= self.settings.buffer_size {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        let date = self.current_date.clone().unwrap_or_else(today);
        let events = self.buffer.drain();
        if events.is_empty() {
            return Ok(());
        }
        let path = self.file_path(&date);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        for ev in &events {
            writeln!(f, "{}", serde_json::to_string(ev)?)?;
        }
        Ok(())
    }

    fn file_path(&self, date: &str) -> PathBuf {
        let name = self.settings.filename_pattern.replace("{date}", date);
        PathBuf::from(&self.settings.path).join(name)
    }
}

pub fn timestamp_date(ts: &str) -> String {
    ts.chars().take(10).collect()
}

pub fn today() -> String {
    let now = chrono::Utc::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}