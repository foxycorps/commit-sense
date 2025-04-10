# CommitSense 🤖

CommitSense is a command-line tool and GitHub Action designed to automate semantic versioning and changelog generation by analyzing Git commit messages using AI. It leverages language models like OpenAI's GPT series to understand the significance of changes since the last release point and suggest the appropriate next version.

## Features

* **AI-Powered Analysis:** Uses AI (configurable model) to interpret commit messages.
* **Semantic Versioning:** Suggests `major`, `minor`, or `patch` bumps based on Conventional Commits patterns (or AI's interpretation).
* **Automated Changelog:** Generates a Markdown changelog section summarizing key changes.
* **Flexible Release Point Discovery:** Finds the "last release" using:
    * Explicit Git ref (`--base-ref`).
    * Glob patterns for tags (`--tag-pattern`).
    * Regex patterns for tags (`--tag-regex`).
    * Conventional commit messages (`release: ...`).
    * Latest SemVer tag (fallback).
    * Initial repository commit (ultimate fallback).
* **Project Type Support:** Works with Rust (`Cargo.toml`) and JavaScript/TypeScript (`package.json`) projects. Auto-detects or allows explicit type setting.
* **Monorepo Friendly:** Use the `--path` argument to target specific packages within a monorepo.
* **GitHub Action:** Easily integrates into your CI/CD pipeline.
* **Dry Run Mode:** Runs read-only by default; use `--write` to modify files.
* **Nightly Releases:** Generate nightly versions with date-based pre-release identifiers using `--nightly`.

## Installation (CLI - Requires Rust)

1.  Ensure you have Rust installed: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2.  Clone the repository: `git clone https://github.com/foxycorps/commit-sense.git`
3.  Navigate to the directory: `cd commit-sense`
4.  Build the release binary: `cargo build --release`
5.  The executable will be at `./target/release/commitsense`. You can copy this to a location in your PATH.

## Usage (CLI)

```bash
# Ensure OPENAI_API_KEY is set in your environment
export OPENAI_API_KEY="sk-..."

# --- Examples ---

# Dry run in current directory (auto-detect project type, find last release automatically)
./target/release/commitsense

# Dry run for a JS sub-package, finding last release via tag pattern
./target/release/commitsense --path ./packages/my-lib --project-type js --tag-pattern "v*.*.*"

# Dry run, explicitly comparing against the 'main' branch
./target/release/commitsense --base-ref main

# Apply changes (update version file and CHANGELOG.md) in current dir
./target/release/commitsense --write

# Generate a nightly release with date-based pre-release identifier
./target/release/commitsense --nightly --write

# Get help
./target/release/commitsense --help
```

## GitHub Action Usage

Add the following workflow to your GitHub repository:

```yaml
name: CommitSense

on:
  pull_request:
    branches: [ main, master ]
  workflow_dispatch:

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Run CommitSense
        uses: foxycorps/commit-sense@0.1.7
        with:
          openai_api_key: ${{ secrets.OPENAI_API_KEY }}
          # Optional parameters:
          # project-path: ./packages/my-package  # For monorepos
          # project-type: rust  # Explicitly set project type
          # write: "true"  # Enable write mode
          # nightly: "true"  # Generate nightly release
```

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--write`, `-w` | Apply version bumps and update changelog files | false (dry run) |
| `--path`, `-p` | Path to project root or specific package | Current directory |
| `--project-type`, `-t` | Project type (`rust` or `js`) | Auto-detected |
| `--tag-pattern` | Git tag glob pattern to find last release | |
| `--tag-regex` | Git tag regex pattern to find last release | |
| `--base-ref` | Git ref to compare against | |
| `--openai-model` | OpenAI model to use | gpt-4o |
| `--nightly` | Generate nightly release with date-based pre-release identifier | false |

## Nightly Releases

CommitSense supports generating nightly releases with date-based pre-release identifiers. This is useful for development builds or testing.

### How it works

1. When the `--nightly` flag is enabled, CommitSense adds a pre-release identifier to the version in the format `-nightly.YYYYMMDD`.
2. For example, version `1.2.3` becomes `1.2.3-nightly.20250410` (where `20250410` is the current date).
3. Nightly releases can be generated even if there are no changes that warrant a version bump.
4. When using the GitHub Action, the nightly version is available as the `nightly_version` output.

### CLI Example

```bash
# Generate a nightly release
commit-sense --api-key YOUR_API_KEY --nightly --write
```

### GitHub Action Example

```yaml
- name: Run CommitSense with Nightly Release
  id: commitsense
  uses: foxycorps/commit-sense@0.1.7
  with:
    openai-api-key: ${{ secrets.OPENAI_API_KEY }}
    project-type: rust
    write: "true"
    nightly: "true"

# Access the nightly version in subsequent steps
- name: Use Nightly Version
  run: echo "Nightly version is ${{ steps.commitsense.outputs.nightly_version }}"
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/foxycorps/commit-sense.git
cd commit-sense

# Install development dependencies
cargo build

# Run tests
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Security

When using CommitSense, never commit your OpenAI API key to your repository. Always use environment variables or GitHub secrets.