use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};
use std::ffi::OsString;

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

pub(crate) fn files_in_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|file_type| file_type.is_file())
                .map(|_| entry.path())
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
