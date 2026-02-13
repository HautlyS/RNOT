use crate::config::Config;
use crate::monitor::Monitor;
use crate::service::ServiceManager;
use crate::storage::Storage;
use crate::telegram::TelegramClient;
use crate::tui::run_tui;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rnot")]
#[command(about = "Website Monitor - Track changes on websites", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the TUI dashboard")]
    Tui,

    #[command(about = "Add a new site to watch")]
    Add {
        #[arg(help = "URL of the website to watch")]
        url: String,
        #[arg(short, long, help = "Name for the site")]
        name: Option<String>,
        #[arg(short, long, help = "CSS selector to extract specific content")]
        selector: Option<String>,
    },

    #[command(about = "Remove a site from watching")]
    Remove {
        #[arg(help = "ID or URL of the site to remove")]
        site: String,
    },

    #[command(about = "List all watched sites")]
    List,

    #[command(about = "Setup Telegram token (will be encrypted)")]
    SetToken {
        #[arg(help = "Telegram bot token")]
        token: String,
    },

    #[command(about = "Clear stored Telegram token")]
    ClearToken,

    #[command(about = "Setup Telegram chat ID")]
    TelegramSetup,

    #[command(about = "Run the monitor daemon (background service)")]
    Daemon,

    #[command(about = "Check all sites once")]
    Check,

    #[command(about = "Show current configuration status")]
    Status,

    #[command(about = "Install system service (run on boot)")]
    InstallService {
        #[arg(long, help = "Skip confirmation prompts")]
        yes: bool,
    },

    #[command(about = "Uninstall system service")]
    UninstallService,

    #[command(about = "Check service status")]
    ServiceStatus,
}

pub async fn run(mut config: Config) -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tui => {
            let token = config.get_telegram_token();
            if token.is_none() {
                println!(
                    "Warning: No Telegram token set. Use 'rnot set-token <TOKEN>' to set one."
                );
            }

            let (events_tx, events_rx) = tokio::sync::mpsc::channel(100);

            let telegram = TelegramClient::new(token, config.app_config.telegram_chat_id.clone());
            let storage = Storage::new(config.data_dir.clone(), config.config_dir.clone());
            let monitor = Monitor::new(telegram, storage);

            let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
            let shutdown_tx_clone = shutdown_tx.clone();
            let monitor_handle = tokio::spawn(async move {
                let (_sites_tx, sites_rx) = tokio::sync::mpsc::channel(1);
                monitor
                    .run(sites_rx, events_tx, shutdown_tx_clone.subscribe())
                    .await;
            });

            run_tui(&mut config, events_rx)?;

            let _ = shutdown_tx.send(());
            monitor_handle.abort();
        }
        Commands::Add {
            url,
            name,
            selector,
        } => {
            let site_name = name.unwrap_or_else(|| {
                url::Url::parse(&url)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| url.clone())
            });

            let id = config.add_site(url, site_name.clone(), selector)?;
            println!("Added site '{}' with ID: {}", site_name, id);
        }
        Commands::Remove { site } => {
            if config.remove_site(&site)? {
                println!("Site removed successfully");
            } else {
                println!("Site not found");
            }
        }
        Commands::List => {
            if config.app_config.sites.is_empty() {
                println!("No sites being watched");
            } else {
                println!("Watched Sites:");
                println!("{:-<60}", "");
                for site in &config.app_config.sites {
                    let status = if site.enabled { "✓" } else { "✗" };
                    let last = site
                        .last_checked
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Never".to_string());
                    println!("{} {} [{}]", status, site.name, site.id);
                    println!("  URL: {}", site.url);
                    println!("  Last checked: {}", last);
                    if let Some(ref sel) = site.css_selector {
                        println!("  Selector: {}", sel);
                    }
                    println!();
                }
            }
        }
        Commands::SetToken { token } => {
            config.set_telegram_token(&token)?;
            println!("Telegram token stored securely (encrypted)");
        }
        Commands::ClearToken => {
            config.clear_telegram_token()?;
            println!("Telegram token cleared");
        }
        Commands::TelegramSetup => {
            let token = config.get_telegram_token().ok_or_else(|| {
                anyhow::anyhow!("No Telegram token set. Use 'rnot set-token <TOKEN>' first.")
            })?;

            let telegram = TelegramClient::new(Some(token), None);

            println!("Send any message to your bot on Telegram...");

            for _ in 0..30 {
                if let Ok(Some(chat_id)) = telegram.get_chat_id_from_updates().await {
                    config.set_telegram_chat_id(chat_id.clone())?;
                    println!("Chat ID set to: {}", chat_id);
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            println!(
                "Timeout: No messages received. Please send a message to your bot and try again."
            );
        }
        Commands::Daemon => {
            let token = config.get_telegram_token();

            if token.is_none() {
                println!("Warning: No Telegram token set. Notifications will be disabled.");
                println!("Use 'rnot set-token <TOKEN>' to enable notifications.");
            }

            println!("Starting daemon mode...");

            let telegram = TelegramClient::new(token, config.app_config.telegram_chat_id.clone());
            let storage = Storage::new(config.data_dir.clone(), config.config_dir.clone());
            let monitor = Monitor::new(telegram, storage);

            let (events_tx, _) = tokio::sync::mpsc::channel(100);
            let (_sites_tx, sites_rx) = tokio::sync::mpsc::channel(1);
            let (_, shutdown_rx) = tokio::sync::broadcast::channel(1);

            monitor.run(sites_rx, events_tx, shutdown_rx).await;
        }
        Commands::Check => {
            let token = config.get_telegram_token();
            let telegram = TelegramClient::new(token, config.app_config.telegram_chat_id.clone());
            let storage = Storage::new(config.data_dir.clone(), config.config_dir.clone());
            let monitor = Monitor::new(telegram, storage);

            for mut site in config.app_config.sites.clone() {
                match monitor.check_site(&mut site).await {
                    Ok(Some(diff)) => {
                        println!(
                            "Changed: {} - {}",
                            site.name,
                            diff.chars().take(100).collect::<String>()
                        );
                        config.update_site(&site)?;
                    }
                    Ok(None) => {
                        println!("No change: {}", site.name);
                    }
                    Err(e) => {
                        println!("Error checking {}: {}", site.name, e);
                    }
                }
            }
        }
        Commands::Status => {
            println!("RNOT Configuration Status");
            println!("{:-<40}", "");

            let has_token = config.has_telegram_token();
            println!(
                "Telegram Token: {}",
                if has_token {
                    "✓ Set (encrypted)"
                } else {
                    "✗ Not set"
                }
            );

            let chat_id = config
                .app_config
                .telegram_chat_id
                .as_ref()
                .map(|_| "✓ Set")
                .unwrap_or("✗ Not set");
            println!("Telegram Chat ID: {}", chat_id);

            println!(
                "Check Interval: {} seconds",
                config.app_config.check_interval_secs
            );
            println!("Watched Sites: {}", config.app_config.sites.len());
            println!("Config Dir: {}", config.config_dir.display());
            println!("Data Dir: {}", config.data_dir.display());
        }
        Commands::InstallService { yes } => {
            ServiceManager::install(yes)?;
        }
        Commands::UninstallService => {
            ServiceManager::uninstall()?;
        }
        Commands::ServiceStatus => {
            ServiceManager::status()?;
        }
    }

    Ok(())
}
