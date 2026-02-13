# RNOT

Website change monitoring with Telegram notifications.

```
 ____  _   _  ___ _____ 
|  _ \| \ | |/ _ \_   _|
| |_) |  \| | | | || |  
|  _ <| |\  | |_| || |  
|_| \_\_| \_|\___/ |_|  
                        
Website Monitor
```

## Features

- Smart change detection with noise filtering
- AES-256-GCM encrypted token storage
- Terminal UI dashboard
- CSS selector support for targeted monitoring
- Configurable check intervals (default: 180s)
- Cross-platform: Linux, macOS, Windows
- Background daemon mode
- System service integration

## Installation

### Pre-built Binaries

```bash
# Linux x86_64
curl -L https://github.com/HautlyS/RNOT/releases/latest/download/rnot-linux-x86_64 -o rnot
chmod +x rnot
sudo mv rnot /usr/local/bin/

# macOS x86_64
curl -L https://github.com/HautlyS/RNOT/releases/latest/download/rnot-macos-x86_64 -o rnot
chmod +x rnot
sudo mv rnot /usr/local/bin/

# macOS ARM64 (M1/M2)
curl -L https://github.com/HautlyS/RNOT/releases/latest/download/rnot-macos-aarch64 -o rnot
chmod +x rnot
sudo mv rnot /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/HautlyS/RNOT/releases/latest/download/rnot-windows-x86_64.exe" -OutFile "rnot.exe"
```

### Build from Source

```bash
git clone https://github.com/HautlyS/RNOT.git
cd RNOT
cargo build --release
sudo cp target/release/rnot /usr/local/bin/
```

## Quick Start

### 1. Setup Telegram Bot

Create a bot via @BotFather on Telegram, then:

```bash
rnot set-token YOUR_BOT_TOKEN
rnot telegram-setup
```

Send a message to your bot when prompted.

### 2. Add Websites

```bash
# Basic monitoring
rnot add https://example.com --name "Example Site"

# Monitor specific content with CSS selector
rnot add https://news.ycombinator.com --name "HN" --selector ".storylink"

# List monitored sites
rnot list
```

### 3. Run Monitor

```bash
rnot tui      # Interactive dashboard
rnot daemon   # Background service
rnot check    # One-time check
```

## CLI Commands

```bash
rnot tui                    # Start TUI dashboard
rnot add <URL>              # Add site to watch
rnot remove <ID|URL>        # Remove site
rnot list                   # List all sites
rnot set-token <TOKEN>      # Set Telegram token (encrypted)
rnot clear-token            # Clear stored token
rnot telegram-setup         # Configure chat ID
rnot daemon                 # Run as background service
rnot check                  # Check all sites once
rnot status                 # Show configuration
```

## TUI Keybindings

- `a` - Add new site
- `d` - Delete selected site
- `t` - Set Telegram token
- `r` - Refresh selected site
- `j/↓` - Move down
- `k/↑` - Move up
- `?` - Show help
- `q` - Quit

## Auto-Start Service (All Platforms)

RNOT can automatically start on system boot or user login:

```bash
# Interactive installation (recommended)
rnot install-service

# You'll be prompted to choose when to start:
#   1) System boot - Starts when computer boots (requires sudo on Linux)
#   2) User login - Starts when you log in (recommended, no sudo needed)

# Skip prompts (defaults to boot)
rnot install-service --yes

# Check service status
rnot service-status

# Uninstall service
rnot uninstall-service
```

### Platform-Specific Details

Linux (systemd)
- Boot: Creates system service in /etc/systemd/system/ (requires sudo)
- Login: Creates user service in ~/.config/systemd/user/ (no sudo)
- Commands:
  systemctl --user status rnot      # User service status
  sudo systemctl status rnot        # System service status
  journalctl --user -u rnot -f      # View user service logs
  sudo journalctl -u rnot -f        # View system service logs

macOS (LaunchAgent)
- Creates plist in ~/Library/LaunchAgents/
- Commands:
  launchctl list | grep rnot
  tail -f ~/Library/Logs/rnot.log

Windows (Task Scheduler)
- Creates scheduled task (requires Administrator)
- Commands:
  schtasks /Query /TN RNOT-Monitor
  schtasks /Run /TN RNOT-Monitor
  schtasks /End /TN RNOT-Monitor

## Configuration

Configuration is stored in:
- Linux: ~/.config/rnot/
- macOS: ~/Library/Application Support/rnot/
- Windows: %APPDATA%\rnot\

Files:
- config.toml - Sites and settings
- .token - Encrypted Telegram token
- .key - Encryption key (auto-generated)

### config.toml Example

```toml
telegram_chat_id = "123456789"
check_interval_secs = 180

[[sites]]
id = "abc123def456"
url = "https://example.com"
name = "Example Site"
enabled = true
css_selector = ".content"
last_checked = "2026-02-13T12:00:00Z"
```

## Advanced Usage

### CSS Selectors

Monitor specific page sections:

```bash
# Monitor only article titles
rnot add https://blog.example.com --selector "article h2"

# Monitor price changes
rnot add https://shop.example.com/product --selector ".price"

# Monitor multiple elements
rnot add https://news.site.com --selector ".headline, .breaking-news"
```

### Change Interval

Edit `~/.config/rnot/config.toml`:

```toml
check_interval_secs = 300  # 5 minutes
```

## Security

- Tokens are encrypted with AES-256-GCM
- Encryption keys stored with 0600 permissions (Unix)
- No plaintext secrets in config files
- Optional system keyring integration

## Troubleshooting

No notifications received:
```bash
rnot status  # Check token and chat ID
rnot telegram-setup  # Reconfigure
```

Build errors:
```bash
# Update Rust
rustup update stable

# Clean build
cargo clean
cargo build --release
```

Permission denied (Linux service):
```bash
sudo chown $USER:$USER /usr/local/bin/rnot
chmod +x /usr/local/bin/rnot
```

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- daemon

# Format code
cargo fmt

# Lint
cargo clippy
```

## Release Process

For maintainers:

```bash
# Run release script
./scripts/release.sh 0.2.0

# Push to GitHub
git push origin main --tags
```

The CI pipeline will:
1. Run pre-flight checks (fmt, clippy, tests)
2. Build binaries for all platforms
3. Run cross-platform tests
4. Create GitHub release with binaries and checksums
5. Generate release notes with ASCII art

All builds must pass 100% before release is published.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file

## Credits

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Ratatui](https://ratatui.rs/) - TUI framework
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [Scraper](https://github.com/causal-agent/scraper) - HTML parsing
- [AES-GCM](https://github.com/RustCrypto/AEADs) - Encryption
