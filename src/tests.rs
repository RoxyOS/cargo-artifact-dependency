use std::fs;

use crate::utils::executable_name;
use crate::{ArtifactDependencyBuilder, BuildProfile, Error, find_artifact};
use cargo_install::CargoInstallBuilder;

#[test]
fn builder_defaults_optional_fields() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .build()
        .unwrap();

    assert_eq!(request.crate_name, "ripgrep");
    assert_eq!(request.version, None);
    assert_eq!(request.bin_name, None);
    assert_eq!(request.profile, BuildProfile::Release);
    assert_eq!(request.target, None);
    assert!(request.locked);
}

#[test]
fn builder_requires_crate_name() {
    let err = ArtifactDependencyBuilder::default().build().unwrap_err();

    assert!(err.to_string().contains("crate_name"));
}

#[test]
fn builder_allows_disabling_locked_installs() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .locked(false)
        .build()
        .unwrap();

    assert!(!request.locked);
}

#[test]
fn install_builder_contains_expected_arguments() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("^14")
        .bin_name("rg")
        .profile(BuildProfile::Custom("dist".to_string()))
        .target("x86_64-unknown-linux-musl")
        .build()
        .unwrap();

    let builder = CargoInstallBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.0")
        .bin("rg")
        .root("/tmp/install-root")
        .locked(true)
        .target(request.target.as_deref().unwrap())
        .profile("dist");

    let install = builder.build().unwrap();

    let args: Vec<String> = install
        .args()
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    assert_eq!(
        args,
        vec![
            "install",
            "--root",
            "/tmp/install-root",
            "--version",
            "14.1.0",
            "--target",
            "x86_64-unknown-linux-musl",
            "--bin",
            "rg",
            "--profile",
            "dist",
            "--locked",
            "ripgrep",
        ]
    );
}

#[test]
fn finds_named_installed_binary_without_network() {
    let install_root = tempfile::tempdir().unwrap();
    let bin_dir = install_root.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    fs::write(bin_dir.join(executable_name("other")), "").unwrap();
    fs::write(bin_dir.join(executable_name("rg")), "").unwrap();

    let artifact_path = find_artifact(install_root.path(), Some("rg")).unwrap();

    assert_eq!(artifact_path, bin_dir.join(executable_name("rg")));
}

#[test]
fn finds_single_installed_binary_without_network() {
    let install_root = tempfile::tempdir().unwrap();
    let bin_dir = install_root.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    fs::write(bin_dir.join(executable_name("rg")), "").unwrap();

    let artifact_path = find_artifact(install_root.path(), None).unwrap();

    assert_eq!(artifact_path, bin_dir.join(executable_name("rg")));
}

#[test]
fn rejects_missing_named_binary_without_network() {
    let install_root = tempfile::tempdir().unwrap();
    fs::create_dir(install_root.path().join("bin")).unwrap();

    let err = find_artifact(install_root.path(), Some("rg")).unwrap_err();

    assert!(matches!(err, Error::InvalidArtifactPath { .. }));
}

#[test]
fn rejects_empty_binary_dir_without_network() {
    let install_root = tempfile::tempdir().unwrap();
    fs::create_dir(install_root.path().join("bin")).unwrap();

    let err = find_artifact(install_root.path(), None).unwrap_err();

    assert!(matches!(err, Error::NoInstalledBinaries { .. }));
}

#[test]
fn rejects_ambiguous_binaries_without_network() {
    let install_root = tempfile::tempdir().unwrap();
    let bin_dir = install_root.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    fs::write(bin_dir.join(executable_name("rg")), "").unwrap();
    fs::write(bin_dir.join(executable_name("rga")), "").unwrap();

    let err = find_artifact(install_root.path(), None).unwrap_err();

    assert!(matches!(err, Error::AmbiguousInstalledBinaries));
}

#[test]
#[ignore = "requires network access and runs cargo install"]
fn resolves_real_binary_crate() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .profile(BuildProfile::Release)
        .build()
        .unwrap();

    let artifact_path = request.resolve().unwrap();

    assert!(artifact_path.is_file());
}
