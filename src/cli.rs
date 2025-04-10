use clap::Parser;
use std::path::PathBuf;
use std::str::FromStr; // Required for custom enum parsing with clap v4+

/// Defines the command-line interface structure and arguments for CommitSense.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "CommitSense: AI-powered git commit analysis for versioning and changelogs",
    long_about = "Analyzes git commits since the last release using an AI model (like OpenAI's GPT) \
                   to suggest the next semantic version and generate a corresponding changelog section. \
                   Supports Rust (Cargo.toml) and JavaScript/TypeScript (package.json) projects, \
                   including monorepos via the --path argument."
)]
pub struct Cli {
    /// Path to the project repository root or a specific sub-package within a monorepo.
    #[arg(short, long, value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Your OpenAI API Key. Can also be set via the OPENAI_API_KEY environment variable.
    #[arg(long, env = "OPENAI_API_KEY", hide_env_values = true)] // Required, read from env if not passed
    pub api_key: String,

    /// The base URL for the OpenAI API. Defaults to the official OpenAI endpoint.
    /// Can also be set via the OPENAI_API_URL environment variable.
    #[arg(
        long,
        env = "OPENAI_API_URL",
        default_value = "https://api.openai.com/v1"
    )]
    pub api_url: String,

    /// The specific OpenAI model to use for analysis (e.g., gpt-4o, gpt-3.5-turbo).
    /// Can also be set via the OPENAI_MODEL environment variable.
    #[arg(long, env = "OPENAI_MODEL", default_value = "gpt-4o")]
    pub model: String,

    /// Explicitly specify the project type ('rust' or 'js'/'ts').
    /// If omitted, CommitSense will attempt to auto-detect based on file presence (Cargo.toml or package.json).
    #[arg(long, value_parser = clap::value_parser!(ProjectType))] // Use value_parser for custom enum
    pub project_type: Option<ProjectType>,

    /// Explicitly set the Git ref (tag, branch, commit hash) marking the start point for commit analysis.
    /// If provided, this overrides all other discovery methods (`--tag-pattern`, `--tag-regex`, conventional commits, etc.).
    #[arg(long, value_name = "REF")]
    pub base_ref: Option<String>,

    /// Use a glob pattern to find the *latest* tag matching it, marking the last release.
    /// Example: --tag-pattern "v*.*.*"
    /// Conflicts with --tag-regex.
    #[arg(long, value_name = "GLOB_PATTERN", conflicts_with = "tag_regex")]
    pub tag_pattern: Option<String>,

    /// Use a regular expression to find the *latest* tag matching it, marking the last release.
    /// Example: --tag-regex "^v\\d+\\.\\d+\\.\\d+$" (use appropriate shell escaping for regex)
    /// Conflicts with --tag-pattern.
    #[arg(long, value_name = "REGEX", conflicts_with = "tag_pattern")]
    pub tag_regex: Option<String>,

    /// Actually perform the changes: update the version in the project file (Cargo.toml/package.json)
    /// and prepend the generated section to CHANGELOG.md.
    /// If false (default), runs in dry-run mode, only printing suggestions.
    #[arg(long, default_value_t = false)]
    pub write: bool,
}

/// Enum representing the supported project types for version file handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")] // Used if serializing ProjectType itself
pub enum ProjectType {
    Rust,
    JavaScript,
}

/// Allows clap to parse the project type from a string input.
/// Handles common aliases for JavaScript/TypeScript.
impl FromStr for ProjectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(ProjectType::Rust),
            "js" | "ts" | "javascript" | "typescript" | "node" => Ok(ProjectType::JavaScript),
            _ => Err(format!(
                "Invalid project type '{}'. Supported types are 'rust', 'js', 'ts', 'javascript', 'typescript', 'node'.",
                s
            )),
        }
    }
}

/// Provides a user-friendly display name for the project type.
impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::JavaScript => write!(f, "JavaScript/TypeScript"),
        }
    }
}