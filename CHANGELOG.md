# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-13

### Added
- Initial release of RNOT
- Cross-platform support (Linux, macOS, Windows)
- TUI dashboard with interactive keybindings
- CLI commands for site management
- AES-256-GCM encrypted token storage
- Telegram bot integration for notifications
- Smart change detection with noise filtering
- CSS selector support for targeted monitoring
- Configurable check intervals (default: 3 minutes)
- Systemd service support for daemon mode
- GitHub Actions CI/CD for automated builds
- Comprehensive documentation and examples

### Features
- Add/remove websites to monitor
- Real-time change detection
- Filter out ads, timestamps, and noise
- HTML content extraction and comparison
- Encrypted credential storage
- Cross-platform configuration directories
- Activity logging in TUI
- One-time check mode
- Background daemon mode
- Status reporting

### Security
- AES-256-GCM encryption for tokens
- Secure key generation and storage
- File permissions (0600) on sensitive files
- No plaintext secrets in configuration

[0.1.0]: https://github.com/HautlyS/RNOT/releases/tag/v0.1.0
