# Contributing to RNOT

Thank you for your interest in contributing to RNOT! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A Telegram bot token (for testing notifications)

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/RNOT.git
   cd RNOT
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

5. Run the application:
   ```bash
   cargo run -- --help
   ```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Ensure no clippy warnings: `cargo clippy -- -D warnings`
- Write clear, concise comments
- Use meaningful variable and function names

## Testing

- Add tests for new features
- Ensure all tests pass before submitting PR
- Run the test suite: `./test.sh`

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/your-feature-name`
2. Make your changes
3. Run tests and linting:
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```
4. Commit with clear messages following [Conventional Commits](https://www.conventionalcommits.org/)
5. Push to your fork
6. Open a Pull Request with:
   - Clear description of changes
   - Reference to any related issues
   - Screenshots (if UI changes)

## Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Example:
```
feat(monitor): add support for JSON content detection

- Detect JSON responses and format them properly
- Add tests for JSON parsing
- Update documentation

Closes #123
```

## Areas for Contribution

- **Features**: New monitoring capabilities, notification channels
- **Bug Fixes**: Report and fix bugs
- **Documentation**: Improve README, add examples
- **Tests**: Increase test coverage
- **Performance**: Optimize monitoring and diff algorithms
- **UI/UX**: Improve TUI interface

## Questions?

Open an issue with the `question` label or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
