# RNOT Quick Reference

## Installation

```bash
# Linux
curl -L https://github.com/HautlyS/RNOT/releases/latest/download/rnot-linux-x86_64 -o rnot
chmod +x rnot && sudo mv rnot /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/HautlyS/RNOT/releases/latest/download/rnot-macos-x86_64 -o rnot
chmod +x rnot && sudo mv rnot /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/HautlyS/RNOT/releases/latest/download/rnot-macos-aarch64 -o rnot
chmod +x rnot && sudo mv rnot /usr/local/bin/
```

## Quick Setup

```bash
# 1. Set Telegram token
rnot set-token YOUR_BOT_TOKEN

# 2. Configure chat ID
rnot telegram-setup

# 3. Add websites
rnot add https://example.com --name "Example"

# 4. Start monitoring
rnot tui  # or: rnot daemon
```

## Common Commands

```bash
rnot add <URL>                    # Add site
rnot add <URL> --selector ".news" # Add with CSS selector
rnot list                         # List all sites
rnot remove <ID>                  # Remove site
rnot check                        # Check all sites once
rnot status                       # Show configuration
rnot tui                          # Start TUI dashboard
rnot daemon                       # Run as daemon
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| `a` | Add new site |
| `d` | Delete selected site |
| `t` | Set Telegram token |
| `r` | Refresh selected site |
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `?` | Show help |
| `q` | Quit |

## Configuration Files

- **Linux**: `~/.config/rnot/`
- **macOS**: `~/Library/Application Support/rnot/`
- **Windows**: `%APPDATA%\rnot\`

Files:
- `config.toml` - Sites and settings
- `.token` - Encrypted Telegram token
- `.key` - Encryption key

## Systemd Service (Linux)

```bash
# Create service
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

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable --now rnot

# Check status
sudo systemctl status rnot
journalctl -u rnot -f
```

## CSS Selectors Examples

```bash
# Monitor article titles
rnot add https://blog.example.com --selector "article h2"

# Monitor prices
rnot add https://shop.example.com/product --selector ".price"

# Monitor multiple elements
rnot add https://news.site.com --selector ".headline, .breaking"
```

## Troubleshooting

```bash
# Check configuration
rnot status

# Reconfigure Telegram
rnot clear-token
rnot set-token NEW_TOKEN
rnot telegram-setup

# View logs (systemd)
journalctl -u rnot -f

# Test single check
rnot check
```

## Environment Variables

```bash
RUST_LOG=debug rnot daemon  # Enable debug logging
```

## Build from Source

```bash
git clone https://github.com/HautlyS/RNOT.git
cd RNOT
cargo build --release
sudo cp target/release/rnot /usr/local/bin/
```

## Support

- Issues: https://github.com/HautlyS/RNOT/issues
- Discussions: https://github.com/HautlyS/RNOT/discussions
