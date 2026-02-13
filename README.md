# RNOT - Website Change Monitor

A fast, cross-platform website monitoring tool with Telegram notifications. Track changes on any website and get instant alerts when content updates.

## Features

- 🔍 **Smart Change Detection** - Filters out ads, timestamps, and noise
- 🔐 **Encrypted Token Storage** - Secure AES-256-GCM encryption
- 🖥️ **TUI Dashboard** - Beautiful terminal interface
- ⚡ **Fast & Lightweight** - Written in Rust
- 🌐 **Cross-Platform** - Linux, macOS, Windows
- 🤖 **Telegram Notifications** - Real-time alerts
- 🎯 **CSS Selectors** - Monitor specific page sections
- 🔄 **Auto-Refresh** - Configurable check intervals (default: 3 minutes)
- 📦 **Systemd Service** - Run as background daemon

## Installation

### Pre-built Binaries

Download the latest release for your platform:

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

1. Create a bot via [@BotFather](https://t.me/botfather) on Telegram
2. Copy the bot token
3. Set the token (encrypted storage):

```bash
rnot set-token YOUR_BOT_TOKEN
```

4. Send a message to your bot, then run:

```bash
rnot telegram-setup
```

### 2. Add Websites to Monitor

```bash
# Add a website
rnot add https://example.com --name "Example Site"

# Add with CSS selector (monitor specific content)
rnot add https://news.ycombinator.com --name "HN" --selector ".storylink"

# List all monitored sites
rnot list
```

### 3. Run the Monitor

**Option A: TUI Dashboard**
```bash
rnot tui
```

**Option B: Daemon Mode**
```bash
rnot daemon
```

**Option C: One-time Check**
```bash
rnot check
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

## Systemd Service (Linux)

Run RNOT as a system service that starts on boot:

```bash
# Create service file
sudo tee /etc/systemd/system/rnot.service > /dev/null <<EOF
[Unit]
Description=Website Monitor Service
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME
ExecStart=/usr/local/bin/rnot daemon
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable rnot
sudo systemctl start rnot

# Check status
sudo systemctl status rnot

# View logs
journalctl -u rnot -f
```

## Configuration

Configuration is stored in:
- **Linux**: `~/.config/rnot/`
- **macOS**: `~/Library/Application Support/rnot/`
- **Windows**: `%APPDATA%\rnot\`

Files:
- `config.toml` - Sites and settings
- `.token` - Encrypted Telegram token
- `.key` - Encryption key (auto-generated)

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

**No notifications received:**
```bash
rnot status  # Check token and chat ID
rnot telegram-setup  # Reconfigure
```

**Build errors:**
```bash
# Update Rust
rustup update stable

# Clean build
cargo clean
cargo build --release
```

**Permission denied (Linux service):**
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

---

**Made with ❤️ by the RNOT Team**
