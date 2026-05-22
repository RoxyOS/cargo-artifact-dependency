use std::io;
use std::path::PathBuf;

use cargo_install::CargoInstallError;
use thiserror::Error;

/// Convenience result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
/// Errors returned while resolving an artifact path.
pub enum Error {
    /// Local filesystem or process I/O failed.
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    /// Resolution failed while building or obtaining the artifact.
    #[error(transparent)]
    CargoInstallFailed(#[from] CargoInstallError),
    /// More than one binary was available and no `bin_name` was provided.
    #[error("multiple installed binaries found and no `bin_name` was provided")]
    AmbiguousInstalledBinaries,
    /// No binary artifacts were found.
    #[error("no installed binaries found in `{dir}`")]
    NoInstalledBinaries { dir: PathBuf },
    /// The expected binary path was not present.
    #[error("expected built artifact at `{}`", path.display())]
    InvalidArtifactPath { path: PathBuf },
}
