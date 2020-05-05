//! Library errors

#[cfg(feature = "enable-glue-errors")]
use thiserror::Error;

#[macro_use]
pub(crate) mod glue;
pub(crate) mod lib;

pub use glue::GlueError;
pub use lib::LibError;
pub(crate) use lib::LibErrorCode;

/// Generic library error type. May be either a [LibError] or a [GlueError].
///
/// [GlueError]: enum.LibError.html
/// [GlueError]: enum.GlueError.html
#[cfg(feature = "enable-glue-errors")]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum BtrfsUtilError {
    /// Glue error
    #[error("{0}")]
    Glue(GlueError),
    /// Library error
    #[error("{0}")]
    Lib(LibError),
}

#[cfg(not(feature = "enable-glue-errors"))]
/// Generic library error type. If [GlueError]s happen, they will panic.
///
/// [GlueError]: enum.GlueError.html
pub type BtrfsUtilError = LibError;
