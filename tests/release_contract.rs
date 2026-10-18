//! Keep the release workflow's required notes present before tagging.
#[test]
fn current_version_has_nonempty_release_notes() {
    let heading = format!("## [{}]", env!("CARGO_PKG_VERSION"));
    let changelog = include_str!("../CHANGELOG.md");
    let mut section = changelog
        .lines()
        .skip_while(|line| !line.starts_with(&heading));
    assert!(section.next().is_some(), "missing {heading} release notes");
    assert!(
        section
            .take_while(|line| !line.starts_with("## ["))
            .any(|line| line.starts_with("- ")),
        "release notes must describe at least one change"
    );
}
