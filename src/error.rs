use thiserror::Error;

/// Defines the specific errors that can occur within the CommitSense tool.
#[derive(Error, Debug)]
pub enum CommitSenseError {
    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("Git command execution failed: {0}")] // Renamed/added variant
    GitCommand(String), // Covers failures running git or parsing its output

    #[error("Project file handling error: {0}")]
    Project(String),

    #[error("OpenAI API Error: {0}")]
    Api(String),

    #[error("Versioning Error: {0}")]
    Version(String),

    #[error("Changelog generation/writing error: {0}")]
    Changelog(String),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    // --- Library Error Wrappers ---
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON parsing/serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Reqwest HTTP client error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),

    // ** Git2 variant removed **
}