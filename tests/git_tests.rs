use anyhow::Result;
use commit_sense::git::*;
use std::path::{PathBuf, Path};
use tempfile::{TempDir, tempdir};
use std::process::Command;

/// Helper function to set up a mock git repository
/// Returns the TempDir and the path to it
fn setup_mock_git_repo() -> Result<(TempDir, PathBuf)> {
    // Create a temporary directory
    let dir = tempdir()?;
    let path = dir.path().to_path_buf();
    
    // Initialize git repo
    let status = Command::new("git")
        .args(["init"])
        .current_dir(&path)
        .status()?;
        
    assert!(status.success());
    
    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&path)
        .status()?;
        
    Command::new("git")
        .args(["config", "user.email", "test@foxycorps.com"])
        .current_dir(&path)
        .status()?;
    
    // Create a dummy file and add it to git
    std::fs::write(path.join("test.txt"), "Hello, world!")?;
    
    // Add the file to git
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&path)
        .status()?;
    
    // Make a commit to ensure the repository is fully initialized
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&path)
        .status()?;
    
    // Return both the TempDir (to keep it alive) and the path
    Ok((dir, path))
}

#[test]
fn test_is_git_repo_positive() -> Result<()> {
    // Keep the TempDir alive by binding it to a variable that lives until the end of the function
    let (dir, repo_path) = setup_mock_git_repo()?;
    
    // Debug: Check if .git directory exists and print path
    let git_dir = repo_path.join(".git");
    println!("Git dir path: {:?}", git_dir);
    println!("Git dir exists: {}", git_dir.exists());
    println!("Git dir is directory: {}", git_dir.is_dir());
    
    // List files in the directory
    println!("Files in repo dir:");
    if let Ok(entries) = std::fs::read_dir(&repo_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {:?}", entry.path());
            }
        }
    }
    
    // Test if our mock repo is recognized as a git repository
    let result = is_git_repo(&repo_path);
    println!("is_git_repo result: {}", result);
    assert!(result);
    
    // Keep dir in scope until the end to prevent early deletion
    let _ = &dir;
    
    Ok(())
}

#[test]
fn test_is_git_repo_negative() -> Result<()> {
    let dir = tempdir()?;
    
    // A new temp directory without git init should not be a git repo
    let result = is_git_repo(dir.path());
    assert!(!result);
    
    Ok(())
}

// Mock implementation for testing
struct MockGitCommandExecutor;

impl GitCommandExecutor for MockGitCommandExecutor {
    fn run_git_command(&self, _path: &Path, args: &[String]) -> Result<String> {
        // Mock different responses based on the command
        if args.contains(&"rev-parse".to_string()) {
            if args.contains(&"HEAD".to_string()) {
                return Ok("abcdef1234567890".to_string());
            }
            if args.contains(&"--is-inside-work-tree".to_string()) {
                return Ok("true".to_string());
            }
        }
        
        if args.contains(&"log".to_string()) {
            // Check if this is a commit format request
            for arg in args {
                if arg.starts_with("--format=") {
                    // Simple format - just return commit messages
                    if arg.contains("%s") {
                        return Ok("feat: Add new feature\nfix: Fix bug\ndocs: Update README".to_string());
                    }
                    // More complex format with body
                    if arg.contains("%B") {
                        return Ok("feat: Add new feature\n\nThis is the body\n<EOM>\nfix: Fix bug\n<EOM>\ndocs: Update README\n<EOM>".to_string());
                    }
                }
            }
        }
        
        if args.contains(&"tag".to_string()) {
            if args.contains(&"--sort=-v:refname".to_string()) {
                return Ok("v1.0.0\nv0.9.0\nv0.8.5".to_string());
            }
            // Simple tag list
            return Ok("v1.0.0\nv0.9.0\nv0.8.5".to_string());
        }
        
        // Default fallback
        Ok("".to_string())
    }
}

#[test]
fn test_get_latest_commit_oid() {
    let executor = MockGitCommandExecutor;
    let git = GitInterface::new(Box::new(executor));
    
    let path = PathBuf::from("/fake/path");
    let result = git.get_latest_commit_oid(&path);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "abcdef1234567890");
}

#[test]
fn test_get_commits_since_oid() {
    let executor = MockGitCommandExecutor;
    let git = GitInterface::new(Box::new(executor));
    
    let path = PathBuf::from("/fake/path");
    let result = git.get_commits_since_oid(&path, "1234567", "%s");
    
    assert!(result.is_ok());
    let commits = result.unwrap();
    assert_eq!(commits.len(), 3);
    assert_eq!(commits[0], "feat: Add new feature");
    assert_eq!(commits[1], "fix: Fix bug");
    assert_eq!(commits[2], "docs: Update README");
}

#[test]
fn test_find_latest_version_tag() {
    let executor = MockGitCommandExecutor;
    let git = GitInterface::new(Box::new(executor));
    
    let path = PathBuf::from("/fake/path");
    let result = git.find_latest_version_tag(&path, None);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "v1.0.0");
}

#[test]
fn test_find_latest_version_tag_with_pattern() {
    let executor = MockGitCommandExecutor;
    let git = GitInterface::new(Box::new(executor));
    
    let path = PathBuf::from("/fake/path");
    let result = git.find_latest_version_tag(&path, Some("v*.*.*"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "v1.0.0");
}
