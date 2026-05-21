mod error;
mod utils;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use apply_if::ApplyIf;
use cargo_install::CargoInstallBuilder;
use derive_builder::Builder;
use utils::unique_install_root;

pub use crate::error::{Error, Result};
use crate::utils::{executable_name, files_in_dir};

#[derive(Clone, Debug, PartialEq, Default, Eq)]
pub enum BuildProfile {
    Debug,
    #[default]
    Release,
    Custom(String),
}

#[derive(Builder, Clone, Debug, Default, PartialEq, Eq)]
#[builder(pattern = "owned", setter(into, strip_option))]
pub struct ArtifactDependency {
    pub crate_name: String,
    #[builder(default)]
    pub version: Option<String>,
    #[builder(default)]
    pub bin_name: Option<String>,
    #[builder(default)]
    pub profile: BuildProfile,
    #[builder(default)]
    pub target: Option<String>,
}

impl ArtifactDependency {
    pub fn resolve(&self) -> Result<PathBuf> {
        let install_root = unique_install_root()?;

        CargoInstallBuilder::default()
            .crate_name(&self.crate_name)
            .root(&install_root)
            .locked(true)
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

// Find the artifact in the provided root
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

// Find the singular binary in the binary dir. Used when binary name is not provided.
fn find_single_binary(dir: PathBuf) -> Result<PathBuf> {
    let mut binaries = files_in_dir(&dir)?;

    match binaries.len() {
        0 => Err(Error::NoInstalledBinaries { dir }),
        1 => Ok(binaries.remove(0)),
        _ => Err(Error::AmbiguousInstalledBinaries),
    }
}
