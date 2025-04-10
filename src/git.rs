use crate::error::CommitSenseError;
use anyhow::{Context, Result};
use glob::Pattern as GlobPattern;
use log::{debug, info, trace, warn};
use regex::Regex;
use semver::Version;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::str;

/// Trait for executing git commands, allowing mocking in tests
pub trait GitCommandExecutor {
    fn run_git_command(&self, path: &Path, args: &[String]) -> Result<String>;
}

/// Default implementation that uses real git commands
pub struct DefaultGitCommandExecutor;

impl GitCommandExecutor for DefaultGitCommandExecutor {
    fn run_git_command(&self, path: &Path, args: &[String]) -> Result<String> {
        // Convert Vec<String> to Vec<&str> for Command
        let args_str: Vec<&str> = args.iter().map(AsRef::as_ref).collect();

        trace!("Running git command: git {}", args_str.join(" "));
        let output = run_git_command_internal(path, &args_str)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(parse_output(output)?)
    }
}

/// GitInterface provides a facade for git operations
/// It uses a GitCommandExecutor for actual command execution
pub struct GitInterface {
    executor: Box<dyn GitCommandExecutor>,
}

impl GitInterface {
    /// Create a new GitInterface with the specified executor
    pub fn new(executor: Box<dyn GitCommandExecutor>) -> Self {
        Self { executor }
    }

    /// Create a new GitInterface with the default executor
    pub fn default() -> Self {
        Self {
            executor: Box::new(DefaultGitCommandExecutor)
        }
    }

    /// Check if the specified path is a git repository
    pub fn is_git_repo(&self, path: &Path) -> bool {
        let git_dir = path.join(".git");
        git_dir.exists() && git_dir.is_dir()
    }

    /// Get the latest commit OID
    pub fn get_latest_commit_oid(&self, path: &Path) -> Result<String> {
        let args = vec!["rev-parse".to_string(), "HEAD".to_string()];
        self.executor.run_git_command(path, &args)
    }

    /// Get commits since a specific OID
    pub fn get_commits_since_oid(&self, path: &Path, base_oid: &str, format: &str) -> Result<Vec<String>> {
        let range = format!("{}..HEAD", base_oid);
        let format_arg = format!("--format={}", format);
        let args = vec![
            "log".to_string(),
            range,
            format_arg,
            "--reverse".to_string()
        ];

        let output = self.executor.run_git_command(path, &args)?;
        Ok(output.lines().map(String::from).collect())
    }

    /// Find the latest version tag
    pub fn find_latest_version_tag(&self, path: &Path, pattern: Option<&str>) -> Result<String> {
        let mut args = vec!["tag".to_string(), "--sort=-v:refname".to_string()];

        if let Some(pattern) = pattern {
            args.push("--list".to_string());
            args.push(pattern.to_string());
        }

        let output = self.executor.run_git_command(path, &args)?;
        let tags: Vec<&str> = output.lines().collect();

        if tags.is_empty() {
            anyhow::bail!("No version tags found");
        }

        Ok(tags[0].to_string())
    }
}

// --- Helper Function for Running Git Commands (internal) ---

/// Executes a Git command and returns its output or an error.
/// Takes the project path as the current directory for the command.
fn run_git_command_internal(project_path: &Path, args: &[&str]) -> Result<Output, CommitSenseError> {
    trace!("Running git command: git {}", args.join(" "));
    let output = Command::new("git")
        .args(args)
        .current_dir(project_path) // Run command in the target project directory
        .stderr(Stdio::piped()) // Capture stderr for error reporting
        .stdout(Stdio::piped()) // Capture stdout
        .output() // Execute the command
        .map_err(|e| {
            // Handle errors like "git command not found"
            CommitSenseError::GitCommand(format!(
                "Failed to execute git command. Is 'git' installed and in PATH? Error: {}",
                e
            ))
        })?;

    if !output.status.success() {
        // Git command failed, provide stderr output for context
        let stderr = str::from_utf8(&output.stderr)
            .unwrap_or("Failed to read git stderr")
            .trim();
        let stdout = str::from_utf8(&output.stdout)
            .unwrap_or("Failed to read git stdout")
            .trim();
        let error_message = format!(
            "Git command `git {}` failed with status {}.\nStderr: {}\nStdout: {}",
            args.join(" "),
            output.status,
            stderr,
            stdout
        );
        warn!("Git command failed: {}", error_message); // Log the full error
        return Err(CommitSenseError::GitCommand(format!(
            "Git command `git {}` failed. Stderr: {}", // Shorter error for propagation
            args.join(" "),
            stderr
        )));
    }

    Ok(output)
}

/// Parses the UTF-8 output from a successful Git command.
fn parse_output(output: Output) -> Result<String, CommitSenseError> {
    str::from_utf8(&output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|e| CommitSenseError::GitCommand(format!("Failed to parse git output: {}", e)))
}

/// Parses the output into lines, filtering empty ones.
fn parse_output_lines(output: Output) -> Result<Vec<String>, CommitSenseError> {
    str::from_utf8(&output.stdout)
        .map_err(|e| CommitSenseError::GitCommand(format!("Failed to parse git output: {}", e)))
        .map(|s| s.lines().map(str::to_string).filter(|l| !l.is_empty()).collect())
}

// --- Core Git Logic Implementation ---

/// Retrieves the commit time (Unix timestamp) for a given Git reference (tag, commit).
fn get_commit_time(project_path: &Path, git_ref: &str) -> Result<i64, CommitSenseError> {
    let output = run_git_command_internal(project_path, &["log", "-1", "--format=%ct", git_ref])?; // %ct = committer date, UNIX timestamp
    let time_str = parse_output(output)?;
    time_str
        .parse::<i64>()
        .map_err(|e| CommitSenseError::GitCommand(format!("Failed to parse commit time '{}' for ref '{}': {}", time_str, git_ref, e)))
}

/// Retrieves the commit OID (hash) for a given Git reference, ensuring it points to a commit.
fn get_commit_oid(project_path: &Path, git_ref: &str) -> Result<String, CommitSenseError> {
    // Use rev-parse to get the OID. Appending '@{}' resolves tags to the commit they point to.
    let output = run_git_command_internal(project_path, &["rev-parse", &format!("{}@{{}}", git_ref)])?; // Use `@{}` to handle branches correctly, falls back for tags/hashes
    let oid = parse_output(output)?;
    if oid.is_empty() {
         return Err(CommitSenseError::GitCommand(format!(
             "Failed to resolve ref '{}' to an OID.",
             git_ref
         )));
    }
    // Optionally, verify it's a commit object (might be overkill if rev-parse succeeds)
    // run_git_command(project_path, &["cat-file", "-t", &oid])?;
    Ok(oid)
}


/// Finds the OID of the base commit using a prioritized strategy via `git` CLI calls.
/// (See documentation in the `git2` version for priority order)
pub fn find_base_commit_oid(
    project_path: &Path, // Pass project path for command execution context
    base_ref_opt: Option<&str>,
    tag_pattern_opt: Option<&str>,
    tag_regex_opt: Option<&str>,
) -> Result<String> { // Return String (OID) instead of git2::Oid

    // --- Strategy 1: Explicit Base Ref Override ---
    if let Some(base_ref) = base_ref_opt {
        info!("Using explicit base reference provided: '{}'", base_ref);
        return get_commit_oid(project_path, base_ref)
            .with_context(|| format!("Failed to resolve explicit base ref '{}'", base_ref));
    }

    // Get all tags once
    let all_tags = match run_git_command_internal(project_path, &["tag", "--list"]) {
        Ok(output) => parse_output_lines(output)?,
        Err(e) => {
            warn!("Failed to list git tags: {}. Proceeding without tag-based discovery.", e);
            Vec::new() // Proceed without tags if listing fails
        }
    };
    debug!("Found {} tags.", all_tags.len());


    // --- Strategy 2 & 3: Pattern/Regex Tag Matching ---
    let mut potential_tags: Vec<(i64, String)> = Vec::new(); // (commit_time, tag_name)
    let pattern_match = if let Some(pattern_str) = tag_pattern_opt {
        info!("Searching for tags matching glob pattern: {}", pattern_str);
        let pattern = GlobPattern::new(pattern_str)?;
        for tag_name in &all_tags {
            if pattern.matches(tag_name) {
                match get_commit_time(project_path, tag_name) {
                    Ok(time) => potential_tags.push((time, tag_name.clone())),
                    Err(e) => warn!("Could not get commit time for tag '{}' matching pattern: {}", tag_name, e),
                }
            }
        }
        true
    } else if let Some(regex_str) = tag_regex_opt {
        info!("Searching for tags matching regex pattern: {}", regex_str);
        let regex = Regex::new(regex_str)?;
        for tag_name in &all_tags {
            if regex.is_match(tag_name) {
                match get_commit_time(project_path, tag_name) {
                    Ok(time) => potential_tags.push((time, tag_name.clone())),
                    Err(e) => warn!("Could not get commit time for tag '{}' matching regex: {}", tag_name, e),
                }
            }
        }
        true
    } else {
        false
    };

    // If pattern/regex yielded results, find the latest one by commit time
    if pattern_match && !potential_tags.is_empty() {
        potential_tags.sort_unstable_by_key(|k| k.0); // Sort by time (oldest first)
        if let Some((_, latest_tag_name)) = potential_tags.last() {
            info!(
                "Found latest matching tag based on pattern/regex: {}",
                latest_tag_name
            );
            return get_commit_oid(project_path, latest_tag_name)
                .with_context(|| format!("Failed to resolve commit OID for tag '{}'", latest_tag_name));
        } else {
             warn!("Pattern/regex provided, but no matching tags with valid commit times found.");
        }
    } else if pattern_match {
         warn!("Pattern/regex provided, but no matching tags found.");
    }

    // --- Strategy 4: Conventional Commit Fallback ---
    info!("Searching for latest 'release: ' conventional commit as base...");
    // Use log with max-count 1 and grep. Format: OID<space>Subject
    match run_git_command_internal(project_path, &[
        "log",
        "--grep=^release: ", // Match prefix
        "-i",                // Case insensitive
        "-E",                // Use extended regex (for grep)
        "-n", "1",           // Max 1 commit
        "--format=%H",       // Only print the commit hash
        "HEAD"               // Start from HEAD
        ])
    {
        Ok(output) => {
            let oid = parse_output(output)?;
            if !oid.is_empty() {
                info!(
                    "Using latest conventional release commit {} as base.",
                    oid
                );
                return Ok(oid);
            } else {
                 debug!("No conventional 'release: ' commits found.");
            }
        },
        Err(e) => warn!("Failed to search for conventional release commits: {}", e), // Log error but continue
    }


    // --- Strategy 5: Default Fallback - Latest SemVer Tag ---
    warn!("No conventional release commit found. Searching for latest SemVer tag...");
    let mut latest_semver_tag: Option<(Version, String, i64)> = None; // (version, name, time)

    for tag_name in &all_tags {
        let version_str = tag_name.strip_prefix('v').unwrap_or(tag_name);
        if let Ok(version) = Version::parse(version_str) {
            match get_commit_time(project_path, tag_name) {
                Ok(time) => {
                    let is_newer = match latest_semver_tag {
                        Some((ref latest_v, _, latest_time)) => {
                            version > *latest_v || (version == *latest_v && time > latest_time)
                        }
                        None => true,
                    };
                    if is_newer {
                        latest_semver_tag = Some((version, tag_name.clone(), time));
                    }
                }
                Err(e) => warn!("Could not get commit time for potential SemVer tag '{}': {}", tag_name, e),
            }
        }
    }

    if let Some((version, tag_name, _)) = latest_semver_tag {
        info!(
            "Using latest SemVer tag '{}' (version {}) as base.",
            tag_name, version
        );
        return get_commit_oid(project_path, &tag_name)
            .with_context(|| format!("Failed to resolve commit OID for SemVer tag '{}'", tag_name));
    }


    // --- Strategy 6: Ultimate Fallback - Initial Commit ---
    warn!("No base ref, pattern/regex match, conventional release, or SemVer tag found.");
    info!("Using initial commit of the repository as base.");
    // Use rev-list to find the commit(s) with no parents
    let output = run_git_command_internal(project_path, &["rev-list", "--max-parents=0", "HEAD"])?;
    let oids = parse_output_lines(output)?;
    if let Some(initial_oid) = oids.first() {
        info!("Found initial commit {}.", initial_oid);
        Ok(initial_oid.clone())
    } else {
         // This should be very rare unless the repository is completely empty or in a strange state
         Err(CommitSenseError::GitCommand(
            "Could not find the initial commit (no commits with zero parents found from HEAD).".to_string()
        ).into()) // Convert to anyhow::Error
    }
}


/// Retrieves commit messages (full message) since a given base commit OID, up to HEAD.
/// Returns commits in chronological order (oldest relevant commit first).
pub fn get_commits_since_oid(project_path: &Path, base_oid: &str) -> Result<Vec<String>> {
    info!("Getting commits since base OID: {}", base_oid);

    // Get HEAD OID to check if base and HEAD are the same
    let head_oid = get_commit_oid(project_path, "HEAD")?;
    if head_oid == base_oid {
         info!("HEAD OID ({}) is the same as the base OID. No new commits.", head_oid);
         return Ok(Vec::new());
    }

    // Use git log with a specific format to easily parse messages.
    // %H: commit hash (optional, could use for debugging)
    // %B: raw body (subject + body)
    // <EOM>: Custom End-Of-Message marker
    // --reverse: Output oldest first (chronological)
    let range = format!("{}..HEAD", base_oid); // Range for log command
    let format_string = "--format=%B%n<EOM>"; // Message body, newline, and marker
    let output = run_git_command_internal(project_path, &["log", &range, format_string, "--reverse"])?;

    // Parse the output based on the <EOM> marker
    let output_str = str::from_utf8(&output.stdout)
         .map_err(|e| CommitSenseError::GitCommand(format!("Failed to parse git log output: {}", e)))?;

    let commits = output_str
        .split("\n<EOM>\n") // Split by the marker (with surrounding newlines)
        .map(|s| s.trim()) // Trim whitespace from each message block
        .filter(|s| !s.is_empty()) // Filter out empty blocks (e.g., potential trailing split)
        .map(String::from)
        .collect::<Vec<String>>();

    info!("Collected {} commit messages since base commit {}", commits.len(), base_oid);
    Ok(commits)
}

// --- Legacy Functions ---
// These are kept for backward compatibility

/// Check if the specified path is a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    let git_dir = path.join(".git");
    git_dir.exists() && git_dir.is_dir()
}