use crate::{ArtifactDependencyBuilder, BuildProfile};
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
}

#[test]
fn builder_requires_crate_name() {
    let err = ArtifactDependencyBuilder::default().build().unwrap_err();

    assert!(err.to_string().contains("crate_name"));
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
