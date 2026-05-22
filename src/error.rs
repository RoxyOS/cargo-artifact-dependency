use std::io;
use std::path::PathBuf;

use cargo_install::CargoInstallError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    CargoInstallFailed(#[from] CargoInstallError),
    #[error("multiple installed binaries found and no `bin_name` was provided")]
    AmbiguousInstalledBinaries,
    #[error("no installed binaries found in `{dir}`")]
    NoInstalledBinaries { dir: PathBuf },
    /// The expected binary path was not present.
    #[error("expected built artifact at `{}`", path.display())]
    InvalidArtifactPath { path: PathBuf },
}
