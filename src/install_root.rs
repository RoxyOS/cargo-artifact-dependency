use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{ArtifactDependency, BuildProfile, utils::sanitize_path_component};

impl ArtifactDependency {
    pub(crate) fn install_root(&self) -> PathBuf {
        let profile = match &self.profile {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
            BuildProfile::Custom(profile) => profile,
        };
        let path = self.path();

        let root_name = [
            self.crate_name.as_str(),
            self.version.as_deref().unwrap_or("any"),
            path.map(|path| path.to_string_lossy())
                .as_deref()
                .unwrap_or("registry"),
            self.bin_name.as_deref().unwrap_or("any-bin"),
            profile,
            self.target.as_deref().unwrap_or("host"),
            if self.locked { "locked" } else { "unlocked" },
        ]
        .map(sanitize_path_component)
        .join("__");

        env::temp_dir()
            .join("cargo-artifact-dependency")
            .join(root_name)
    }

    /// Returns self.path only if it exists, otherwise returns None
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path
            .as_deref()
            .and_then(|path| path.exists().then_some(path))
    }
}
