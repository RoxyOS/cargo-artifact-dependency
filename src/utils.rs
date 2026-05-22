use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::Result;
use std::ffi::OsString;

// Creates a unique install root for `cargo install` commands.
pub(crate) fn unique_install_root() -> Result<PathBuf> {
    let dir = tempfile::Builder::new()
        .prefix("cargo-artifact-dependency-")
        .tempdir()?;
    Ok(dir.keep())
}

pub(crate) fn executable_name(bin_name: &str) -> std::ffi::OsString {
    #[cfg(windows)]
    {
        let mut file = OsString::from(bin_name);
        file.push(".exe");
        file
    }

    #[cfg(not(windows))]
    {
        OsString::from(bin_name)
    }
}

// Returns all the files in a directory
pub(crate) fn files_in_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }

    files.sort();
    Ok(files)
}
