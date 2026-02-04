# Contributing to nucr

Thank you for your interest in contributing to nucr! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/nucr.git`
3. Create a branch: `git checkout -b your-feature-name`
4. Make your changes
5. Test your changes: `cargo test`
6. Ensure code is formatted: `cargo fmt`
7. Check clippy: `cargo clippy`
8. Commit your changes: `git commit -m "Add your commit message"`
9. Push to your fork: `git push origin your-feature-name`
10. Open a Pull Request

## Development Setup

1. Install Rust (stable toolchain recommended)
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`

## Code Style

- We use `rustfmt` for code formatting. Run `cargo fmt` before committing
- We use `clippy` for linting. Run `cargo clippy` before committing
- All public APIs should be documented
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

## Testing

- Add tests for new features
- Ensure all existing tests pass
- Tests are located in `src/main.rs` under the `#[cfg(test)]` module
- Run tests with: `cargo test` or `cargo nextest run`

## Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers if applicable (e.g., "Fix #123: ...")
- Keep commits focused and atomic

## Pull Request Process

1. Update the README.md with details of changes if needed
2. Ensure all tests pass and CI checks succeed
3. Request review from maintainers
4. Address any feedback

## Questions?

Feel free to open an issue for any questions about contributing.



