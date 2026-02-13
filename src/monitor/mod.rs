use crate::config::WatchedSite;
use crate::diff::{compute_diff, extract_content, filter_noise};
use crate::storage::Storage;
use crate::telegram::TelegramClient;
use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct Monitor {
    client: reqwest::Client,
    telegram: TelegramClient,
    storage: Storage,
}

#[derive(Debug, Clone)]
pub enum MonitorEvent {
    SiteChecked { site_id: String, changed: bool },
    SiteChanged { site_id: String, diff: String },
    Error { site_id: String, error: String },
}

impl Monitor {
    pub fn new(telegram: TelegramClient, storage: Storage) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client,
            telegram,
            storage,
        }
    }

    pub async fn check_site(&self, site: &mut WatchedSite) -> Result<Option<String>> {
        info!("Checking site: {} ({})", site.name, site.url);

        let response = self.client.get(&site.url).send().await?;
        let html = response.text().await?;

        let content = extract_content(&html, site.css_selector.as_deref())?;
        let filtered = filter_noise(&content);

        let hash = self.compute_hash(&filtered);
        site.last_checked = Some(Utc::now());

        if let Some(ref last_hash) = site.last_hash {
            if &hash != last_hash {
                let old_content = self.storage.get_snapshot(&site.id)?;
                let diff = compute_diff(&old_content, &filtered);

                self.storage.save_snapshot(&site.id, &filtered)?;
                site.last_hash = Some(hash);
                site.last_change = Some(Utc::now());

                return Ok(Some(diff));
            }
        } else {
            self.storage.save_snapshot(&site.id, &filtered)?;
            site.last_hash = Some(hash);
        }

        Ok(None)
    }

    fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn run(
        &self,
        _sites_rx: mpsc::Receiver<Vec<WatchedSite>>,
        events_tx: mpsc::Sender<MonitorEvent>,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(180));

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Monitor shutting down");
                    break;
                }
                _ = interval.tick() => {
                    let sites = self.storage.load_sites();
                    for mut site in sites {
                        match self.check_site(&mut site).await {
                            Ok(Some(diff)) => {
                                let site_id = site.id.clone();
                                let _ = self.storage.update_site(&site);

                                let message = format!(
                                    "🔄 <b>Change detected!</b>\n\n\
                                    <b>Site:</b> {}\n\
                                    <b>URL:</b> {}\n\
                                    <b>Time:</b> {}\n\n\
                                    <b>Changes:</b>\n{}",
                                    site.name,
                                    site.url,
                                    Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                                    self.format_diff_for_telegram(&diff)
                                );

                                if let Err(e) = self.telegram.send_message(&message).await {
                                    error!("Failed to send Telegram notification: {}", e);
                                }

                                if let Err(e) = events_tx.send(MonitorEvent::SiteChanged {
                                    site_id,
                                    diff,
                                }).await {
                                    error!("Failed to send event: {}", e);
                                }
                            }
                            Ok(None) => {
                                let _ = events_tx.send(MonitorEvent::SiteChecked {
                                    site_id: site.id.clone(),
                                    changed: false,
                                }).await;
                            }
                            Err(e) => {
                                let site_id = site.id.clone();
                                let _ = events_tx.send(MonitorEvent::Error {
                                    site_id,
                                    error: e.to_string(),
                                }).await;
                            }
                        }
                    }
                }
            }
        }
    }

    fn format_diff_for_telegram(&self, diff: &str) -> String {
        let lines: Vec<&str> = diff.lines().take(20).collect();
        let result = lines.join("\n");
        if diff.lines().count() > 20 {
            format!("{}\n\n<i>... (truncated)</i>", result)
        } else {
            result
        }
    }
}
