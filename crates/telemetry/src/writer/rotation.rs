//! Daily file rotation and retention pruning.

use std::path::Path;

use crate::config::DiskSettings;

pub struct Rotation {
    current: Option<String>,
}

impl Rotation {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn retain(&mut self, settings: &DiskSettings, new_date: &str) -> anyhow::Result<()> {
        self.current = Some(new_date.to_string());
        if settings.retention_days == 0 {
            return Ok(());
        }
        let dir = Path::new(&settings.path);
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("telemetry_") || !name.ends_with(".json") {
                continue;
            }
            let date = name
                .trim_start_matches("telemetry_")
                .trim_end_matches(".json");
            if date.len() == 10 && is_older_than(date, settings.retention_days) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        Ok(())
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::new()
    }
}

fn is_older_than(date: &str, days: u32) -> bool {
    let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return false;
    };
    let today = chrono::Utc::now().date_naive();
    let Some(limit) = today.checked_sub_days(chrono::Days::new(days as u64)) else {
        return false;
    };
    parsed < limit
}