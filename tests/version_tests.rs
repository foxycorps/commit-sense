use anyhow::Result;
use commit_sense::version::{calculate_expected_version, create_nightly_version};
use semver::Version;
use chrono::Utc;

#[test]
fn test_calculate_expected_version_major() -> Result<()> {
    let current = Version::parse("1.2.3")?;
    let expected = calculate_expected_version(&current, "major");

    assert_eq!(expected.to_string(), "2.0.0");
    Ok(())
}

#[test]
fn test_calculate_expected_version_minor() -> Result<()> {
    let current = Version::parse("1.2.3")?;
    let expected = calculate_expected_version(&current, "minor");

    assert_eq!(expected.to_string(), "1.3.0");
    Ok(())
}

#[test]
fn test_calculate_expected_version_patch() -> Result<()> {
    let current = Version::parse("1.2.3")?;
    let expected = calculate_expected_version(&current, "patch");

    assert_eq!(expected.to_string(), "1.2.4");
    Ok(())
}

#[test]
fn test_calculate_expected_version_none() -> Result<()> {
    let current = Version::parse("1.2.3")?;
    let expected = calculate_expected_version(&current, "none");

    assert_eq!(expected.to_string(), "1.2.3");
    Ok(())
}

#[test]
fn test_calculate_expected_version_prerelease() -> Result<()> {
    let current = Version::parse("1.2.3-alpha.1+build.123")?;

    // Major bump should clear pre-release and build metadata
    let expected_major = calculate_expected_version(&current, "major");
    assert_eq!(expected_major.to_string(), "2.0.0");

    // Minor bump should clear pre-release and build metadata
    let expected_minor = calculate_expected_version(&current, "minor");
    assert_eq!(expected_minor.to_string(), "1.3.0");

    // Patch bump should clear pre-release and build metadata
    let expected_patch = calculate_expected_version(&current, "patch");
    assert_eq!(expected_patch.to_string(), "1.2.4");

    Ok(())
}

#[test]
fn test_calculate_expected_version_invalid_bump() -> Result<()> {
    let current = Version::parse("1.2.3")?;
    let expected = calculate_expected_version(&current, "invalid");

    // Should return the original version for invalid bump types
    assert_eq!(expected.to_string(), "1.2.3");
    Ok(())
}

#[test]
fn test_create_nightly_version() -> Result<()> {
    let version = Version::parse("1.2.3")?;
    let nightly = create_nightly_version(&version);

    // Check that the major, minor, and patch versions remain the same
    assert_eq!(nightly.major, 1);
    assert_eq!(nightly.minor, 2);
    assert_eq!(nightly.patch, 3);

    // Check that the pre-release identifier starts with "nightly."
    assert!(nightly.pre.to_string().starts_with("nightly."));

    // Check that the pre-release identifier contains a date in the format YYYYMMDD
    let today = Utc::now().format("%Y%m%d").to_string();
    assert!(nightly.pre.to_string().contains(&today));

    // Check that the build metadata is empty
    assert!(nightly.build.is_empty());

    Ok(())
}
