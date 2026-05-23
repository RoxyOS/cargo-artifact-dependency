use std::{env, path::PathBuf};

use crate::{ArtifactDependency, BuildProfile, utils::sanitize_path_component};

impl ArtifactDependency {
    pub(crate) fn install_root(&self) -> PathBuf {
        env::temp_dir()
            .join("cargo-artifact-dependency")
            .join(self.install_root_name())
    }

    fn install_root_name(&self) -> String {
        let profile = match &self.profile {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
            BuildProfile::Custom(profile) => profile,
        };

        [
            self.crate_name.as_str(),
            self.version.as_deref().unwrap_or("any"),
            self.path
                .as_ref()
                .map(|path| path.to_string_lossy())
                .as_deref()
                .unwrap_or("registry"),
            self.bin_name.as_deref().unwrap_or("any-bin"),
            profile,
            self.target.as_deref().unwrap_or("host"),
            if self.locked { "locked" } else { "unlocked" },
        ]
        .map(sanitize_path_component)
        .join("__")
    }
}
