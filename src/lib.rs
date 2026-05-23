//! Stable crate alternative for [cargo artifact dependency](https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies).
//!
//! Warning:
//! This crate currently only supports binary artifacts. If you need other
//! artifact types, please open an [issue on github](https://github.com/RoxyOS/cargo-artifact-dependency/issues).
//!
//! # Why
//!
//! Cargo artifact dependencies are still an unstable Cargo feature and may
//! still have bugs. This crate exists as a temporary alternative while artifact
//! dependency support remains unstable.
//!
//! Use [`ArtifactDependencyBuilder`] to describe a dependency and call
//! [`ArtifactDependency::resolve`] to resolve its artifact path.
//!
//! # Example
//!
//! ```no_run
//! use cargo_artifact_dependency::{ArtifactDependencyBuilder, BuildProfile};
//! // Describe the ripgrep dependency and resolve its artifact.
//! let artifact_path = ArtifactDependencyBuilder::default()
//!     .crate_name("ripgrep")
//!     .version("^14")
//!     .bin_name("rg")
//!     .profile(BuildProfile::Release)
//!     .build()
//!     .unwrap()
//!     .resolve()?;
//!
//! // Use the resolved artifact path in your own workflow.
//! println!("{}", artifact_path.display());
//! # Ok::<(), cargo_artifact_dependency::Error>(())
//! ```

mod error;
mod install_root;
mod utils;

#[cfg(test)]
mod network_tests;
#[cfg(test)]
mod tests;

use std::{
    io,
    path::{Path, PathBuf},
};

use apply_if::ApplyIf;
use cargo_install::CargoInstallBuilder;
use derive_builder::Builder;

pub use crate::error::{Error, Result};
use crate::utils::{executable_name, files_in_dir};

#[derive(Clone, Debug, PartialEq, Default, Eq)]
pub enum BuildProfile {
    Debug,
    #[default]
    Release,
    Custom(String),
}

/// Describes an artifact dependency.
///
/// Use [`ArtifactDependencyBuilder`] to construct values. `crate_name` is
/// required; all other fields are optional.
#[derive(Builder, Clone, Debug, PartialEq, Eq)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct ArtifactDependency {
    /// Crate name to resolve.
    pub crate_name: String,
    #[builder(default)]
    /// Version requirement.
    pub version: Option<String>,
    #[builder(default)]
    /// Binary name.
    pub bin_name: Option<String>,
    #[builder(default)]
    /// Build profile.
    pub profile: BuildProfile,
    #[builder(default)]
    /// Target triple.
    pub target: Option<String>,
    #[builder(default = "true")]
    /// Whether to use the exact versions from the dependency's lockfile.
    pub locked: bool,
}

impl Default for ArtifactDependency {
    fn default() -> Self {
        Self {
            crate_name: String::new(),
            version: None,
            bin_name: None,
            profile: BuildProfile::Release,
            target: None,
            locked: true,
        }
    }
}

impl ArtifactDependency {
    /// Resolves the artifact path.
    pub fn resolve(&self) -> Result<PathBuf> {
        let install_root = self.install_root();

        if let Some(artifact_path) = cached_artifact(&install_root, self.bin_name.as_deref())? {
            return Ok(artifact_path);
        }

        CargoInstallBuilder::default()
            .crate_name(&self.crate_name)
            .root(&install_root)
            .locked(self.locked)
            .apply_if_some(self.version.as_deref(), |builder, version_req| {
                builder.version(version_req)
            })
            .apply_if_some(self.bin_name.as_deref(), |builder, bin_name| {
                builder.bin(bin_name)
            })
            .apply_if_some(self.target.as_deref(), |builder, target| {
                builder.target(target)
            })
            .apply_if(matches!(self.profile, BuildProfile::Debug), |builder| {
                builder.debug(true)
            })
            .apply_if_some(
                match &self.profile {
                    BuildProfile::Custom(profile) => Some(profile.as_str()),
                    _ => None,
                },
                |builder, profile| builder.profile(profile),
            )
            .build()
            .expect("CargoInstallBuilder should not fail with optional-only fields")
            .run()?;

        find_artifact(&install_root, self.bin_name.as_deref())
    }
}

// Returns the artifact path when the install root already contains one.
fn cached_artifact(install_root: &Path, bin_name: Option<&str>) -> Result<Option<PathBuf>> {
    match find_artifact(install_root, bin_name) {
        Ok(artifact_path) => Ok(Some(artifact_path)),
        Err(Error::Io(err)) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(Error::NoInstalledBinaries { .. } | Error::InvalidArtifactPath { .. }) => Ok(None),
        Err(err) => Err(err),
    }
}

// Find the artifact in the provided root.
fn find_artifact(install_root: &Path, bin_name: Option<&str>) -> Result<PathBuf> {
    let bin_dir = install_root.join("bin");

    match bin_name {
        Some(bin_name) => find_binary_with_name(bin_dir, bin_name),
        None => find_single_binary(bin_dir),
    }
}

fn find_binary_with_name(dir: PathBuf, name: &str) -> Result<PathBuf> {
    let artifact_path = dir.join(executable_name(name));
    if artifact_path.is_file() {
        Ok(artifact_path)
    } else {
        Err(Error::InvalidArtifactPath {
            path: artifact_path,
        })
    }
}

// Find the singular binary in the binary directory when no name is provided.
fn find_single_binary(dir: PathBuf) -> Result<PathBuf> {
    let mut binaries = files_in_dir(&dir)?;

    match binaries.len() {
        0 => Err(Error::NoInstalledBinaries { dir }),
        1 => Ok(binaries.remove(0)),
        _ => Err(Error::AmbiguousInstalledBinaries),
    }
}
