use crate::config::{AppConfig, WatchedSite};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub struct Storage {
    data_dir: PathBuf,
    config_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            data_dir,
            config_dir,
        }
    }

    pub fn save_snapshot(&self, site_id: &str, content: &str) -> Result<()> {
        let snapshot_dir = self.data_dir.join("snapshots");
        std::fs::create_dir_all(&snapshot_dir)?;

        let snapshot_file = snapshot_dir.join(format!("{}.txt", site_id));
        std::fs::write(&snapshot_file, content)?;

        Ok(())
    }

    pub fn get_snapshot(&self, site_id: &str) -> Result<String> {
        let snapshot_file = self
            .data_dir
            .join("snapshots")
            .join(format!("{}.txt", site_id));

        if snapshot_file.exists() {
            Ok(std::fs::read_to_string(&snapshot_file)?)
        } else {
            Ok(String::new())
        }
    }

    pub fn load_sites(&self) -> Vec<WatchedSite> {
        let config_file = self.config_dir.join("config.toml");

        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config.sites;
                }
            }
        }

        Vec::new()
    }

    pub fn update_site(&self, site: &WatchedSite) -> Result<()> {
        let config_file = self.config_dir.join("config.toml");
        let content = std::fs::read_to_string(&config_file)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        if let Some(existing) = config.sites.iter_mut().find(|s| s.id == site.id) {
            *existing = site.clone();
        }

        let updated = toml::to_string_pretty(&config)?;
        std::fs::write(&config_file, updated)?;

        Ok(())
    }

    pub fn save_change_record(
        &self,
        site_id: &str,
        diff: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<()> {
        let history_dir = self.data_dir.join("history");
        std::fs::create_dir_all(&history_dir)?;

        let filename = format!("{}_{}.json", site_id, timestamp.format("%Y%m%d_%H%M%S"));
        let history_file = history_dir.join(filename);

        let record = serde_json::json!({
            "site_id": site_id,
            "timestamp": timestamp.to_rfc3339(),
            "diff": diff,
        });

        std::fs::write(&history_file, serde_json::to_string_pretty(&record)?)?;

        Ok(())
    }
}
