mod cli;
mod config;
mod crypto;
mod diff;
mod monitor;
mod service;
mod storage;
mod telegram;
mod tui;

use anyhow::Result;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::new()?;

    cli::run(config).await
}
