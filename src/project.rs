use crate::cli::ProjectType;
use crate::error::CommitSenseError;
use anyhow::{Context, Result};
use log::{debug, info};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

/// Represents the project whose version is being managed.
/// Handles reading and writing version information to the appropriate file (`Cargo.toml` or `package.json`).
#[derive(Debug)]
pub struct Project {
    /// The root path of the project or sub-package being processed.
    root_path: PathBuf,
    /// The type of project (Rust or JavaScript/TypeScript).
    project_type: ProjectType,
    /// The path to the file containing the version number (e.g., `Cargo.toml` or `package.json`).
    version_file: PathBuf,
}

impl Project {
    /// Creates a new `Project` instance.
    ///
    /// Determines the project type and version file path based on the presence of
    /// `Cargo.toml` or `package.json`, unless `explicit_type` is provided.
    ///
    /// # Arguments
    /// * `path` - The path to the project directory.
    /// * `explicit_type` - An optional `ProjectType` provided via CLI arguments.
    pub fn new(path: &Path, explicit_type: Option<ProjectType>) -> Result<Self> {
        let cargo_path = path.join("Cargo.toml");
        let package_path = path.join("package.json");

        let (project_type, version_file) = match explicit_type {
            // If type is explicitly provided, verify the corresponding file exists.
            Some(pt) => {
                info!(
                    "Using explicitly provided project type: {}", pt
                );
                let vf = match pt {
                    ProjectType::Rust => cargo_path,
                    ProjectType::JavaScript => package_path,
                };
                if !vf.exists() {
                    return Err(CommitSenseError::Config(format!(
                        "Explicit project type '{}' specified, but the expected version file '{}' was not found in '{}'.",
                        pt, vf.file_name().map_or_else(|| "file".into(), |f| f.to_string_lossy()), path.display()
                    )).into());
                }
                (pt, vf)
            }
            // If not explicit, attempt auto-detection.
            None => {
                 info!(
                     "Attempting to auto-detect project type in '{}'...", path.display()
                 );
                if cargo_path.exists() {
                    info!("Auto-detected Rust project (found Cargo.toml).");
                    (ProjectType::Rust, cargo_path)
                } else if package_path.exists() {
                    info!("Auto-detected JavaScript/TypeScript project (found package.json).");
                    (ProjectType::JavaScript, package_path)
                } else {
                    // Neither file found, cannot determine project type.
                    return Err(CommitSenseError::Config(format!(
                        "Could not auto-detect project type. No 'Cargo.toml' or 'package.json' found in '{}'. Please specify the type using --project-type.",
                        path.display()
                    )).into());
                }
            }
        };

        debug!(
            "Project initialized: Type={}, VersionFile='{}'",
            project_type,
            version_file.display()
        );
        Ok(Project {
            root_path: path.to_path_buf(),
            project_type,
            version_file,
        })
    }

    /// Returns the type of the project.
    pub fn project_type(&self) -> ProjectType {
        self.project_type
    }

    /// Returns the path to the version file (`Cargo.toml` or `package.json`).
    pub fn version_file_path(&self) -> &Path {
        &self.version_file
    }

    /// Reads and returns the current version string from the project's version file.
    pub fn get_current_version(&self) -> Result<String> {
        info!("Reading current version from '{}'", self.version_file.display());
        let content = fs::read_to_string(&self.version_file).with_context(|| {
            format!(
                "Failed to read version file '{}'",
                self.version_file.display()
            )
        })?;

        match self.project_type {
            ProjectType::Rust => {
                // Parse Cargo.toml content
                let toml_value: TomlValue = toml::from_str(&content).map_err(CommitSenseError::TomlParse)?;
                // Extract version from [package] table
                let version = toml_value
                    .get("package")
                    .and_then(|p| p.get("version"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .ok_or_else(|| {
                        CommitSenseError::Project(
                            "Could not find '[package].version' string in Cargo.toml.".to_string(),
                        )
                    })?;
                debug!("Found Rust version: {}", version);
                Ok(version)
            }
            ProjectType::JavaScript => {
                // Parse package.json content
                let json_value: JsonValue = serde_json::from_str(&content)?; // Uses From<serde_json::Error> for CommitSenseError
                                                                              // Extract top-level "version" key
                let version = json_value
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .ok_or_else(|| {
                        CommitSenseError::Project(
                            "Could not find top-level 'version' string in package.json."
                                .to_string(),
                        )
                    })?;
                debug!("Found JS/TS version: {}", version);
                Ok(version)
            }
        }
    }

    /// Updates the version string in the project's version file.
    /// Note: This overwrites the file with potentially reformatted content.
    pub fn set_version(&mut self, new_version: &str) -> Result<()> {
        info!(
            "Updating version in '{}' to '{}'",
            self.version_file.display(),
            new_version
        );
        // Read the current content first
        let content = fs::read_to_string(&self.version_file).with_context(|| {
            format!(
                "Failed to read version file '{}' before update",
                self.version_file.display()
            )
        })?;

        let updated_content = match self.project_type {
            ProjectType::Rust => {
                // Parse TOML, update the version, and serialize back to string
                let mut toml_value: TomlValue = toml::from_str(&content)?; // Handles parse error

                // Get mutable access to the [package] table
                let package_table = toml_value
                    .get_mut("package")
                    .and_then(|v| v.as_table_mut())
                    .ok_or_else(|| {
                        CommitSenseError::Project(
                            "Could not find mutable [package] table in Cargo.toml for update."
                                .to_string(),
                        )
                    })?;

                // Insert or update the version field
                package_table.insert(
                    "version".to_string(),
                    TomlValue::String(new_version.to_string()),
                );

                // Serialize back using toml::to_string_pretty for better formatting preservation
                 toml::to_string_pretty(&toml_value).map_err(CommitSenseError::TomlSerialize)?
            }
            ProjectType::JavaScript => {
                // Parse JSON, update the version, and serialize back to string
                let mut json_value: JsonValue = serde_json::from_str(&content)?; // Handles parse error

                // Get mutable access to the root JSON object
                let obj = json_value.as_object_mut().ok_or_else(|| {
                    CommitSenseError::Project(
                        "package.json root is not a JSON object.".to_string(),
                    )
                })?;

                // Insert or update the version field
                obj.insert(
                    "version".to_string(),
                    JsonValue::String(new_version.to_string()),
                );

                // Serialize back using serde_json::to_string_pretty for standard JS formatting (2 spaces)
                let mut pretty_json = serde_json::to_string_pretty(&json_value)?;
                // Add trailing newline often expected in JS config files
                pretty_json.push('\n');
                pretty_json
            }
        };

        // Write the modified content back to the file, overwriting the original.
        fs::write(&self.version_file, updated_content).with_context(|| {
            format!(
                "Failed to write updated version to '{}'",
                self.version_file.display()
            )
        })?;
        debug!("Successfully wrote updated version to file.");
        Ok(())
    }
}