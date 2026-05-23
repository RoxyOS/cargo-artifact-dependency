use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use crate::error::Result;
use std::ffi::OsString;

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

pub(crate) fn sanitize_path_component(component: &str) -> String {
    component
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn cargo_install_version_req(version: &str) -> Cow<'_, str> {
    if is_bare_major_minor_version(version) {
        Cow::Owned(format!("^{version}"))
    } else {
        Cow::Borrowed(version)
    }
}

fn is_bare_major_minor_version(version: &str) -> bool {
    let mut parts = version.split('.');

    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), None)
            if !major.is_empty()
                && !minor.is_empty()
                && major.chars().all(|ch| ch.is_ascii_digit())
                && minor.chars().all(|ch| ch.is_ascii_digit())
    )
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
