use anyhow::Result;
use commit_sense::version::calculate_expected_version;
use semver::Version;

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
