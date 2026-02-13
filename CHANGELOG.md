# Changelog

All notable changes to RNOT will be documented in this file.

## [0.2.0] - 2026-02-13

### Added
- Interactive service installation with boot/login trigger selection
- Cross-platform auto-start support (systemd, launchd, Task Scheduler)
- User service option for Linux (no sudo required)
- System service option for Linux (requires sudo, runs on boot)
- Service status and uninstall commands
- Improved CI/CD pipeline with pre-flight checks
- Comprehensive release automation with checksums

### Changed
- README rewritten with ASCII aesthetic (removed emojis and AI slop)
- Simplified documentation with technical focus
- Service installation now prompts user for trigger preference
- Release workflow now requires all builds to pass before publishing

### Fixed
- Service detection for both user and system services on Linux
- Proper cleanup when uninstalling services

## [0.1.0] - 2026-02-13

### Added
- Initial release
- Website change monitoring with smart diff detection
- Telegram bot integration for notifications
- AES-256-GCM encrypted token storage
- Terminal UI dashboard with ratatui
- CSS selector support for targeted monitoring
- Configurable check intervals
- Cross-platform support (Linux, macOS, Windows)
- Background daemon mode
- Basic systemd service support
