use anyhow::{Context, Result};
use chrono::Local;
use log::{debug, info};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// The standard name for the changelog file.
const CHANGELOG_FILE: &str = "CHANGELOG.md";
/// A standard header for new changelog files.
const CHANGELOG_HEADER: &str = "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n";
/// Marker indicating the start of version sections, used for insertion logic.
const VERSION_SECTION_MARKER: &str = "## [";

/// Formats the AI-generated changes into a standard Markdown section for the changelog.
///
/// # Arguments
/// * `version` - The version string for this release (e.g., "1.2.3").
/// * `changes_markdown` - The raw Markdown bullet points provided by the AI.
///
/// # Returns
/// A formatted string representing the complete section for this version.
pub fn format_changelog_section(version: &str, changes_markdown: &str) -> String {
    let today = Local::now().format("%Y-%m-%d");
    // Ensure the heading level is H2 (##) and includes the date.
    // Trim whitespace from AI output just in case, and ensure proper spacing.
    format!(
        "{} {}] - {}\n\n{}", // Uses VERSION_SECTION_MARKER
        VERSION_SECTION_MARKER,
        version,
        today,
        changes_markdown.trim()
    )
}

/// Prepends the newly formatted changelog section to the `CHANGELOG.md` file.
/// Creates the file with a header if it doesn't exist.
///
/// # Arguments
/// * `project_path` - The root path of the project where `CHANGELOG.md` should reside.
/// * `new_section` - The fully formatted Markdown section for the new version.
pub fn write_changelog(project_path: &Path, new_section: &str) -> Result<()> {
    let changelog_path = project_path.join(CHANGELOG_FILE);
    info!("Updating changelog file: {}", changelog_path.display());

    let mut existing_content = String::new();
    let file_existed = changelog_path.exists();

    if file_existed {
        // Read the existing content if the file exists.
        debug!("Reading existing changelog content.");
        let mut file = File::open(&changelog_path)
            .with_context(|| format!("Failed to open existing {}", CHANGELOG_FILE))?;
        file.read_to_string(&mut existing_content)
            .with_context(|| format!("Failed to read existing {}", CHANGELOG_FILE))?;
    } else {
        // If the file doesn't exist, start with the default header.
        info!("{} not found. Creating file with default header.", CHANGELOG_FILE);
        existing_content.push_str(CHANGELOG_HEADER);
    }

    // Determine where to insert the new section.
    // We want to insert it *after* the main header but *before* any previous version sections.
    let insert_pos = if file_existed {
        // Find the position of the first version section marker (e.g., "## [")
        existing_content
            .find(VERSION_SECTION_MARKER)
            // If no previous version sections, find the position after the header.
            // This assumes the header ends with two newlines.
            .unwrap_or_else(|| existing_content.find("\n\n\n").map_or(existing_content.len(), |p| p + 2)) // after header or end

    } else {
         // If creating the file, insert after the header we just added.
        CHANGELOG_HEADER.len()
    };

    debug!("Determined insertion position: {}", insert_pos);

    // Construct the new complete content by inserting the new section.
    let mut new_content = String::with_capacity(existing_content.len() + new_section.len() + 5); // Pre-allocate
    new_content.push_str(&existing_content[..insert_pos]); // Part before insertion point (e.g., header)
    new_content.push_str(new_section); // The new version section itself
    new_content.push_str("\n\n"); // Add extra spacing after the new section
    new_content.push_str(&existing_content[insert_pos..]); // The rest of the old content (previous versions)


    // Write the combined content back to the file, overwriting it.
    // Using OpenOptions to ensure creation if it didn't exist.
    let mut file = OpenOptions::new()
        .write(true)
        .create(true) // Create if it doesn't exist
        .truncate(true) // Overwrite existing content
        .open(&changelog_path)
        .with_context(|| format!("Failed to open {} for writing", CHANGELOG_FILE))?;

    file.write_all(new_content.as_bytes())
        .with_context(|| format!("Failed to write updated content to {}", CHANGELOG_FILE))?;

    info!("Successfully updated {}", CHANGELOG_FILE);
    Ok(())
}