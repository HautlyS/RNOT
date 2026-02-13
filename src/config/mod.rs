use crate::crypto::TokenEncryption;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedSite {
    pub id: String,
    pub url: String,
    pub name: String,
    pub last_hash: Option<String>,
    pub last_checked: Option<DateTime<Utc>>,
    pub last_change: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub css_selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteChange {
    pub site_id: String,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub diff: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub telegram_chat_id: Option<String>,
    pub check_interval_secs: u64,
    pub sites: Vec<WatchedSite>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            telegram_chat_id: None,
            check_interval_secs: 180,
            sites: Vec::new(),
        }
    }
}

pub struct Config {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub app_config: AppConfig,
    encryption: TokenEncryption,
    cached_token: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rnot");

        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rnot");

        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;

        let config_file = config_dir.join("config.toml");
        let app_config = if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            toml::from_str(&content)?
        } else {
            let default_config = AppConfig::default();
            let content = toml::to_string_pretty(&default_config)?;
            std::fs::write(&config_file, content)?;
            default_config
        };

        let encryption = TokenEncryption::new(config_dir.clone());
        let cached_token = None;

        Ok(Self {
            config_dir,
            data_dir,
            app_config,
            encryption,
            cached_token,
        })
    }

    pub fn get_telegram_token(&self) -> Option<String> {
        if let Some(ref token) = self.cached_token {
            return Some(token.clone());
        }

        let token_file = self.config_dir.join(".token");
        if token_file.exists() {
            if let Ok(encrypted) = std::fs::read_to_string(&token_file) {
                if let Ok(token) = self.encryption.decrypt(&encrypted) {
                    return Some(token);
                }
            }
        }

        None
    }

    pub fn set_telegram_token(&mut self, token: &str) -> Result<()> {
        let encrypted = self.encryption.encrypt(token)?;
        let token_file = self.config_dir.join(".token");
        std::fs::write(&token_file, &encrypted)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&token_file, std::fs::Permissions::from_mode(0o600))?;
        }

        self.cached_token = Some(token.to_string());
        Ok(())
    }

    pub fn clear_telegram_token(&mut self) -> Result<()> {
        let token_file = self.config_dir.join(".token");
        if token_file.exists() {
            std::fs::remove_file(&token_file)?;
        }
        self.cached_token = None;
        Ok(())
    }

    pub fn has_telegram_token(&self) -> bool {
        self.get_telegram_token().is_some()
    }

    pub fn save(&self) -> Result<()> {
        let config_file = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(&self.app_config)?;
        std::fs::write(&config_file, content)?;
        Ok(())
    }

    pub fn add_site(
        &mut self,
        url: String,
        name: String,
        css_selector: Option<String>,
    ) -> Result<String> {
        let id = self.generate_id(&url);
        let site = WatchedSite {
            id: id.clone(),
            url,
            name,
            last_hash: None,
            last_checked: None,
            last_change: None,
            enabled: true,
            css_selector,
        };
        self.app_config.sites.push(site);
        self.save()?;
        Ok(id)
    }

    pub fn remove_site(&mut self, id: &str) -> Result<bool> {
        let initial_len = self.app_config.sites.len();
        self.app_config.sites.retain(|s| s.id != id && s.url != id);
        if self.app_config.sites.len() < initial_len {
            self.save()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn set_telegram_chat_id(&mut self, chat_id: String) -> Result<()> {
        self.app_config.telegram_chat_id = Some(chat_id);
        self.save()
    }

    pub fn update_site(&mut self, site: &WatchedSite) -> Result<()> {
        if let Some(existing) = self.app_config.sites.iter_mut().find(|s| s.id == site.id) {
            *existing = site.clone();
            self.save()?;
        }
        Ok(())
    }

    fn generate_id(&self, url: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..8])
    }
}
