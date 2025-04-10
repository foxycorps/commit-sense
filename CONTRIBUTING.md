# Contributing to CommitSense

Thank you for considering contributing to CommitSense! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs

If you find a bug, please create an issue using the bug report template. Include:

1. A clear, descriptive title
2. Steps to reproduce the issue
3. Expected behavior vs. actual behavior
4. Environment details (OS, Rust version, etc.)
5. Any relevant logs or screenshots

### Suggesting Features

We welcome feature suggestions! When creating a feature request:

1. Provide a clear description of the feature
2. Explain how it would benefit users
3. Outline potential implementation approaches if possible

### Pull Requests

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Write and test your changes
4. Ensure your code follows the project's style guidelines
5. Run tests with `cargo test`
6. Commit your changes with descriptive commit messages
7. Push to your branch
8. Open a Pull Request

## Development Environment Setup

```bash
# Clone the repository
git clone https://github.com/foxycorps/commit-sense.git
cd commit-sense

# Build the project
cargo build

# Run tests
cargo test
```

## Coding Standards

- Follow standard Rust formatting using `rustfmt`
- Run `cargo clippy` to catch common mistakes
- Add comments for complex logic
- Write unit tests for new functionality

## Documentation

- Update the README.md if you change command-line options
- Document public functions and types with rustdoc comments

## Commit Messages

Please use clear, descriptive commit messages that explain why the change was made (not just what was changed).

## License

By contributing to CommitSense, you agree that your contributions will be licensed under the project's MIT license.
