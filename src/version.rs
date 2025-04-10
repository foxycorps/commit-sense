use semver::Version;

/// Calculates the expected next version based on a strict interpretation of the bump type.
///
/// This is primarily used as a helper to *validate* or *compare against* the version
/// suggested by the AI, rather than being the source of truth itself.
///
/// # Arguments
/// * `current_version` - The current semantic version.
/// * `bump_type` - The type of bump ("major", "minor", "patch", "none") intended.
///
/// # Returns
/// The calculated `semver::Version`. If `bump_type` is "none" or invalid,
/// it returns a clone of the `current_version`.
pub fn calculate_expected_version(current_version: &Version, bump_type: &str) -> Version {
    let mut expected_version = current_version.clone();

    match bump_type {
        "major" => {
            expected_version.major += 1;
            expected_version.minor = 0;
            expected_version.patch = 0;
            // Clear pre-release and build metadata on major bumps
            expected_version.pre = semver::Prerelease::EMPTY;
            expected_version.build = semver::BuildMetadata::EMPTY;
        }
        "minor" => {
            expected_version.minor += 1;
            expected_version.patch = 0;
            // Clear pre-release and build metadata on minor bumps
            expected_version.pre = semver::Prerelease::EMPTY;
            expected_version.build = semver::BuildMetadata::EMPTY;
        }
        "patch" => {
            expected_version.patch += 1;
            // Clear pre-release and build metadata on patch bumps
            expected_version.pre = semver::Prerelease::EMPTY;
            expected_version.build = semver::BuildMetadata::EMPTY;
        }
        "none" | _ => {
            // If "none" or an unexpected bump type is provided (though it should be validated earlier),
            // the expected version is simply the current version.
            // No changes needed, expected_version is already a clone.
        }
    }
    expected_version
}