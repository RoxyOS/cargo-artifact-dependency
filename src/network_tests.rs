use std::fs;

use crate::{ArtifactDependency, ArtifactDependencyBuilder, BuildProfile};

fn ripgrep_request() -> ArtifactDependency {
    ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .bin_name("rg")
        .profile(BuildProfile::Release)
        .build()
        .unwrap()
}

#[test]
#[ignore = "requires network access and runs cargo install"]
fn resolves_real_binary_crate() {
    let request = ripgrep_request();

    let artifact_path = request.resolve().unwrap();

    assert!(artifact_path.is_file());
}

#[test]
#[ignore = "requires network access and runs cargo install"]
fn resolves_real_binary_crate_from_cache_on_second_run() {
    let request = ripgrep_request();
    let install_root = request.install_root();
    _ = fs::remove_dir_all(&install_root);

    let first_artifact_path = request.resolve().unwrap();
    let first_modified = first_artifact_path.metadata().unwrap().modified().unwrap();
    let second_artifact_path = request.resolve().unwrap();
    let second_modified = second_artifact_path.metadata().unwrap().modified().unwrap();

    assert_eq!(first_artifact_path, second_artifact_path);
    assert_eq!(first_modified, second_modified);
}

#[test]
#[ignore = "requires network access and runs cargo install"]
fn falls_back_to_registry_when_local_path_is_unavailable() {
    let request = ArtifactDependencyBuilder::default()
        .crate_name("ripgrep")
        .version("14.1.1")
        .path("/definitely/not/existing/cargo-artifact-dependency-ripgrep")
        .bin_name("rg")
        .profile(BuildProfile::Release)
        .build()
        .unwrap();
    let install_root = request.install_root();
    _ = fs::remove_dir_all(&install_root);

    let artifact_path = request.resolve().unwrap();

    assert!(artifact_path.is_file());
}
