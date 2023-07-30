use crate::error::LibErrorCode;

use std::ffi::NulError;
use std::path::PathBuf;
use std::str::Utf8Error;

use thiserror::Error;
use uuid::Error as UuidError;

/// Errors that can be raised by the glue between this Rust library and the original [libbtrfsutil]
/// C library.
///
/// [libbtrfsutil]: https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum GlueError {
    /// Unknown errno.
    #[error("Unknown error code: {0}.")]
    UnknownErrno(LibErrorCode),
    /// Null pointer received from the C library when a null pointer was not expected.
    #[error("Null pointer received.")]
    NullPointerReceived,
    /// Utf8Error. Wrapper around an [std::str::Utf8Error]. May arise when converting a [CString] into
    /// a Rust [String].
    ///
    /// [std::str::Utf8Error]: https://doc.rust-lang.org/stable/std/str/struct.Utf8Error.html
    /// [CString]: https://doc.rust-lang.org/stable/std/ffi/struct.CString.html
    /// [String]: https://doc.rust-lang.org/stable/std/string/struct.String.html
    #[error("{0}")]
    Utf8Error(Utf8Error),
    /// Bad path. May arise when a conversion from a [PathBuf] into a [&str] fails.
    ///
    /// [PathBuf]: https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html
    /// [&str]: https://doc.rust-lang.org/stable/std/primitive.str.html
    #[error("Bad path: {0}")]
    BadPath(PathBuf),
    /// NulError. Wrapper around [std::ffi::NulError]. May arise when trying to create a [CString]
    /// from an [&str] that contains null bytes.
    ///
    /// [std::ffi::NulError]: https://doc.rust-lang.org/stable/std/ffi/struct.NulError.html
    /// [CString]: https://doc.rust-lang.org/stable/std/ffi/struct.CString.html
    /// [&str]: https://doc.rust-lang.org/stable/std/primitive.str.html
    #[error("{0}")]
    NulError(NulError),
    /// UuidError. Wrapper around [uuid::Error]. May arise when trying to create a [Uuid] for a
    /// [SubvolumeInfo] from a byte array.
    ///
    /// [uuid::Error]: https://docs.rs/uuid/0.8.1/uuid/struct.Error.html
    /// [Uuid]: https://docs.rs/uuid/0.8.1/uuid/struct.Uuid.html
    /// [SubvolumeInfo]: ../subvolume/struct.SubvolumeInfo.html
    #[error("{0}")]
    UuidError(UuidError),
    /// Bad timespec. May arise when a conversion from a [timespec] to a [NaiveDateTime] fails. The
    /// error message contains a debug-formatted representation of the timespec struct.
    ///
    /// [timespec]: ../bindings/struct.timespec.html
    /// [NaiveDateTime]: https://docs.rs/chrono/0.4.11/chrono/naive/struct.NaiveDateTime.html
    #[error("Bad timespec: {0}")]
    BadTimespec(String),
    /// Bad id. May arise when an id is smaller than [BTRFS_FS_TREE_OBJECTID].
    ///
    /// [BTRFS_FS_TREE_OBJECTID]: ../bindings/constant.BTRFS_FS_TREE_OBJECTID.html
    #[error("Bad id: {0}")]
    BadId(u64),
}

/// Macro for handling a potential glue error.
#[cfg(feature = "enable-glue-errors")]
macro_rules! glue_error {
    ($condition: expr, $glue_err: expr) => {
        if $condition {
            return crate::Result::Err(crate::BtrfsUtilError::Glue($glue_err.into()));
        }
    };
    ($glue_err: expr) => {
        return crate::Result::Err(crate::BtrfsUtilError::Glue($glue_err.into()))
    };
}

/// Macro for handling a potential glue error.
#[cfg(not(feature = "enable-glue-errors"))]
macro_rules! glue_error {
    ($condition: expr, $glue_err: expr) => {
        if $condition {
            panic!("Glue error: {}", $glue_err)
        }
    };
    ($glue_err: expr) => {
        panic!("Glue error: {}", $glue_err)
    };
}
