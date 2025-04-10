//! CommitSense library for analyzing Git commits and generating semantic version bumps
//! and changelog entries using AI.

pub mod changelog;
pub mod cli;
pub mod error;
pub mod git;
pub mod openai;
pub mod project;
pub mod version;

// Re-export commonly used types
pub use crate::error::CommitSenseError;
// Re-export the ProjectType enum to make it public
pub use crate::cli::ProjectType;