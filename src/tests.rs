use std::fs;

use crate::utils::{executable_name, sanitize_path_component};
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
fn install_root_is_stable_for_same_dependency() {
    let first = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .build()
        .unwrap();
    let second = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .build()
        .unwrap();

    assert_eq!(first.install_root(), second.install_root());
}

#[test]
fn install_root_changes_for_different_dependency_settings() {
    let release = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .profile(BuildProfile::Release)
        .build()
        .unwrap();
    let debug = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .profile(BuildProfile::Debug)
        .build()
        .unwrap();

    assert_ne!(release.install_root(), debug.install_root());
}

#[test]
fn install_root_changes_for_version_bin_target_and_locked() {
    let base = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .target("x86_64-unknown-linux-gnu")
        .locked(true)
        .build()
        .unwrap();
    let different_version = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.0")
        .bin_name("rg")
        .target("x86_64-unknown-linux-gnu")
        .locked(true)
        .build()
        .unwrap();
    let different_bin = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg-alt")
        .target("x86_64-unknown-linux-gnu")
        .locked(true)
        .build()
        .unwrap();
    let different_target = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .target("aarch64-unknown-linux-gnu")
        .locked(true)
        .build()
        .unwrap();
    let different_locked = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .target("x86_64-unknown-linux-gnu")
        .locked(false)
        .build()
        .unwrap();

    assert_ne!(base.install_root(), different_version.install_root());
    assert_ne!(base.install_root(), different_bin.install_root());
    assert_ne!(base.install_root(), different_target.install_root());
    assert_ne!(base.install_root(), different_locked.install_root());
}

#[test]
fn install_root_sanitizes_path_components() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("some/crate")
        .version("^14")
        .bin_name("bin:name")
        .profile(BuildProfile::Custom("release+thin".to_string()))
        .target("x86_64/unknown/linux")
        .build()
        .unwrap();

    let root = request.install_root();
    let root_name = root.file_name().unwrap().to_string_lossy();

    assert_eq!(
        root_name,
        "some_crate___14__bin_name__release_thin__x86_64_unknown_linux__locked"
    );
}

#[test]
fn resolves_existing_artifact_from_stable_install_root_without_network() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .bin_name("rg")
        .build()
        .unwrap();
    let install_root = request.install_root();
    let bin_dir = install_root.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(bin_dir.join(executable_name("rg")), "").unwrap();

    let artifact_path = request.resolve().unwrap();

    assert_eq!(artifact_path, bin_dir.join(executable_name("rg")));
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
fn sanitize_path_component_replaces_path_unsafe_characters() {
    assert_eq!(sanitize_path_component("abc-DEF_123.4"), "abc-DEF_123.4");
    assert_eq!(
        sanitize_path_component("^14/bin:name+thin"),
        "_14_bin_name_thin"
    );
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
