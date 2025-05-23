// Declare modules comprising the application
mod changelog;
mod cli;
mod error;
mod git; // Implementation now uses std::process::Command
mod openai;
mod project;
mod version;

// --- Imports ---
use crate::{
    cli::Cli,                // Bring CLI definitions into scope
};
use anyhow::{Context, Result}; // For easy error handling and context addition
use clap::Parser; // To parse command-line arguments
use log::{error, info, warn}; // For logging different levels of information

/// Entry point of the CommitSense application.
/// Parses arguments, sets up logging, and orchestrates the main logic.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger based on the RUST_LOG environment variable
    // (e.g., RUST_LOG=info, RUST_LOG=commitsense=debug)
    // Defaults to a reasonable level if RUST_LOG is not set.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init() // Use try_init to avoid panic if logger is already initialized (e.g. in tests)
        .err() // Ignore the result, logging initialization failure isn't critical enough to stop
        .map(|e| eprintln!("Warning: Failed to initialize logger: {}", e)); // Print warning to stderr

    // Parse command-line arguments using the definition in `cli.rs`
    let cli_args = Cli::parse();
    info!("Starting CommitSense v{}...", env!("CARGO_PKG_VERSION"));

    // Execute the core logic, handling potential errors
    if let Err(e) = run_commitsense(&cli_args).await {
        // Log the error details for debugging
        // Use {:?} for detailed error information, including context chain from anyhow
        error!("CommitSense execution failed: {:?}", e);
        // Return the error to indicate failure to the OS
        return Err(e);
    }

    info!("CommitSense finished successfully.");
    Ok(()) // Indicate successful execution
}

// --- Core Logic Function ---

/// Orchestrates the main workflow of CommitSense using `std::process::Command` for Git.
async fn run_commitsense(config: &Cli) -> Result<()> {
    // 1. Resolve Project Path
    // Ensure the specified path exists and is accessible.
    let project_path = config.path.canonicalize().with_context(|| {
        format!(
            "Failed to find canonical path for directory '{}'. Does it exist?",
            config.path.display()
        )
    })?;
    info!("Operating in target directory: {}", project_path.display());

    // Note: We no longer open a `git2::Repository` object here.
    // The git functions in `src/git.rs` now take `project_path` as an argument
    // and execute `git` commands within that directory.

    // 2. Initialize Project Details (Detect Type, Read Current Version)
    // This step remains the same, handling Cargo.toml/package.json.
    let mut project = project::Project::new(&project_path, config.project_type)?;
    let current_version_str = project.get_current_version()?;
    info!(
        "Detected project type: {} | Current version: {}",
        project.project_type(), // Display formatted project type
        current_version_str
    );

    // 3. Determine Base Commit OID for Analysis
    // This function now uses `git` CLI commands internally. It returns a String OID.
    let base_oid = git::find_base_commit_oid(
        &project_path,              // Pass the project path for command execution context
        config.base_ref.as_deref(), // Pass optional explicit ref
        config.tag_pattern.as_deref(), // Pass optional glob pattern
        config.tag_regex.as_deref(), // Pass optional regex pattern
    )
    .context("Failed to determine the base commit for analysis")?; // Add context to potential errors
    info!("Using base commit OID {} for analysis.", base_oid);

    // 4. Retrieve Commits Since Base Commit
    // This function now uses `git log` internally.
    let commits = git::get_commits_since_oid(&project_path, &base_oid)
        .context("Failed to retrieve commits since the base OID")?;

    // Check if there are any new commits to analyze.
    if commits.is_empty() {
        warn!(
            "No new commits found since base commit {}. No version bump or changelog needed.",
            base_oid
        );
        println!("No new commits detected since the last identified release point.");
        // Output for GitHub Actions to indicate no change
        if let Ok(github_output) = std::env::var("GITHUB_OUTPUT") {
            // Using the new environment file approach
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new().append(true).open(github_output) {
                writeln!(file, "bump_type=none").ok();
                writeln!(file, "next_version={}", current_version_str).ok();
                writeln!(file, "changelog=No changes detected since last release.").ok();
            }
        } else {
            // Fallback for local runs or older GitHub Actions
            println!("bump_type: none");
            println!("next_version: {}", current_version_str);
            println!("changelog: No changes detected since last release.");
        }
        return Ok(()); // Exit successfully, nothing more to do.
    }
    info!(
        "Found {} commit messages since base commit {}.",
        commits.len(),
        base_oid
    );

    // 5. Interact with OpenAI API
    // This section remains unchanged as it doesn't depend on the Git implementation details.
    info!("Initializing OpenAI client...");
    let openai_client = openai::OpenAIClient::new(
        config.api_key.clone(), // Clone API key (String)
        config.api_url.clone(), // Clone API URL (String)
        config.model.clone(),   // Clone model name (String)
    );

    // Get the AI's suggestion (includes validation within the method)
    let ai_suggestion = openai_client
        .get_version_and_changelog(
            &current_version_str,
            &commits,
            project.project_type(),
        )
        .await
        .context("Failed to get and validate suggestion from OpenAI API")?;

    info!(
        "Received and validated AI suggestion: Bump='{}', NextVersion='{}'",
        ai_suggestion.bump_type, ai_suggestion.next_version
    );

    // Apply nightly versioning if requested
    let mut final_version = ai_suggestion.next_version.clone();
    if config.nightly {
        // Parse the version string
        let parsed_version = semver::Version::parse(&ai_suggestion.next_version)
            .context("Failed to parse AI suggested version for nightly conversion")?;

        // Create a nightly version
        let nightly_version = version::create_nightly_version(&parsed_version);
        final_version = nightly_version.to_string();

        info!("Applied nightly versioning: {} -> {}", ai_suggestion.next_version, final_version);
    }

    // 6. Format the Changelog Section
    let changelog_section = changelog::format_changelog_section(
        &final_version, // Use the final version (may be nightly)
        &ai_suggestion.changelog_markdown,
    );

    // 7. Output Results to Console
    println!("\n--- CommitSense Analysis ---");
    println!("Suggested Bump Type: {}", ai_suggestion.bump_type);
    println!("Suggested Next Version: {}", ai_suggestion.next_version);
    if config.nightly {
        println!("Nightly Version: {}", final_version);
    }
    println!("\nGenerated Changelog Section:");
    println!("----------------------------");
    println!("{}", changelog_section);
    println!("----------------------------");

    // 8. Set Outputs for GitHub Actions
    info!("Setting GitHub Actions outputs...");
    if let Ok(github_output) = std::env::var("GITHUB_OUTPUT") {
        // Using the new environment file approach
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new().append(true).open(github_output) {
            writeln!(file, "bump_type={}", ai_suggestion.bump_type).ok();
            writeln!(file, "next_version={}", ai_suggestion.next_version).ok();

            // Add nightly version output if nightly flag is set
            if config.nightly {
                writeln!(file, "nightly_version={}", final_version).ok();
            }

            // For multiline values, we need to use a special delimiter syntax
            let delimiter = format!("EOF_{}", std::process::id());
            writeln!(file, "changelog<<{}", delimiter).ok();
            writeln!(file, "{}", changelog_section).ok();
            writeln!(file, "{}", delimiter).ok();
        }
    } else {
        // Fallback for local runs or older GitHub Actions
        println!("bump_type: {}", ai_suggestion.bump_type);
        println!("next_version: {}", ai_suggestion.next_version);
        if config.nightly {
            println!("nightly_version: {}", final_version);
        }
        println!("\nChangelog:\n{}", changelog_section);
    }

    // 9. Write Changes to Files (if --write flag is enabled)
    // This section remains unchanged.
    if config.write {
        // Only proceed with writing if a version bump actually occurred or nightly is enabled.
        if ai_suggestion.bump_type != "none" || config.nightly {
            info!("--write flag detected and bump type is not 'none' or nightly is enabled. Applying changes...");

            // Update the version in Cargo.toml or package.json
            project
                .set_version(&final_version) // Use the final version (may be nightly)
                .context("Failed to update project version file")?;
            info!(
                "Successfully updated version in {} to {}",
                project.version_file_path().display(),
                final_version
            );

            // Prepend the generated section to CHANGELOG.md
            changelog::write_changelog(&project_path, &changelog_section)
                .context("Failed to update CHANGELOG.md")?;
            info!("Successfully updated CHANGELOG.md");

            println!(
                "\nChanges applied: Project version updated to {} and CHANGELOG.md updated.",
                final_version
            );
        } else {
            info!("--write flag detected, but bump type is 'none' and nightly is not enabled. No file changes needed.");
            println!("\n(No file changes applied as suggested bump type was 'none')");
        }
    } else {
        // If --write was not specified, indicate dry-run mode.
        info!("Dry run mode (--write flag not set). No files were modified.");
        println!("\n(Dry Run - No files were changed)");
    }

    Ok(()) // Indicate success
}