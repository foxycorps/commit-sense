use anyhow::Result;
use commit_sense::changelog::*;
use std::fs;
use tempfile::tempdir;
use chrono::Local;

#[test]
fn test_changelog_creation() -> Result<()> {
    // Create a temporary directory for the test
    let dir = tempdir()?;
    let path = dir.path().to_path_buf();
    
    // Format today's date
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    // Create a changelog entry
    let version = "1.1.0";
    let changes = "- Added new feature\n- Fixed critical bug";
    let formatted_section = format!("## [{}] - {}\n\n{}", version, today, changes);
    
    // Write the changelog
    let result = write_changelog(&path, &formatted_section);
    
    // Should succeed
    assert!(result.is_ok());
    
    // Check that CHANGELOG.md now exists
    let changelog_path = path.join("CHANGELOG.md");
    assert!(changelog_path.exists());
    
    // Check content
    let content = fs::read_to_string(changelog_path)?;
    assert!(content.contains("# Changelog"));
    assert!(content.contains(&format!("## [{}] - {}", version, today)));
    assert!(content.contains("- Added new feature"));
    assert!(content.contains("- Fixed critical bug"));
    
    Ok(())
}

#[test]
fn test_changelog_append() -> Result<()> {
    // Create a temporary directory for the test
    let dir = tempdir()?;
    let path = dir.path().to_path_buf();
    
    // Format today's date
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    // Create an existing CHANGELOG.md
    let initial_content = "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n## [1.0.0] - 2025-04-01\n\n- Initial release\n";
    fs::write(path.join("CHANGELOG.md"), initial_content)?;
    
    // Create a changelog entry
    let version = "1.1.0";
    let changes = "- Added new feature";
    let formatted_section = format!("## [{}] - {}\n\n{}", version, today, changes);
    
    // Append to the changelog
    let result = write_changelog(&path, &formatted_section);
    assert!(result.is_ok());
    
    // Check content
    let content = fs::read_to_string(path.join("CHANGELOG.md"))?;
    assert!(content.contains("# Changelog"));
    assert!(content.contains(&format!("## [{}] - {}", version, today)));
    assert!(content.contains("- Added new feature"));
    assert!(content.contains("## [1.0.0] - 2025-04-01"));
    assert!(content.contains("- Initial release"));
    
    Ok(())
}

#[test]
fn test_format_and_write() -> Result<()> {
    // Create a temporary directory for the test
    let dir = tempdir()?;
    let path = dir.path().to_path_buf();
    
    // Format today's date
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    // Create a changelog entry
    let version = "2.0.0";
    let changes = "- Major update\n- Breaking changes";
    let formatted_section = format!("## [{}] - {}\n\n{}", version, today, changes);
    
    // Write the changelog
    let result = write_changelog(&path, &formatted_section);
    
    // Should succeed
    assert!(result.is_ok());
    
    // Check content
    let content = fs::read_to_string(path.join("CHANGELOG.md"))?;
    assert!(content.contains("# Changelog"));
    assert!(content.contains(&format!("## [{}] - {}", version, today)));
    assert!(content.contains("- Major update"));
    assert!(content.contains("- Breaking changes"));
    
    Ok(())
}
