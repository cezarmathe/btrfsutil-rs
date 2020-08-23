use crate::error::GlueError;
use crate::Result;

use std::ffi::CString;
use std::path::Path;

/// Convert a PathBuf into a CString.
#[inline]
pub(crate) fn path_to_cstr(path: &Path) -> Result<CString> {
    let path_str: &str;
    match path.to_str() {
        Some(val) => path_str = val,
        None => glue_error!(GlueError::BadPath(path.to_owned())),
    }
    match CString::new(path_str) {
        Ok(val) => Ok(val),
        Err(e) => glue_error!(GlueError::NulError(e)),
    }
}

/// Macro for preparing for an unsafe function execution and reacting to its
/// error code
macro_rules! unsafe_wrapper {
    ($unsafe_block: block) => {{
        let errcode: LibErrorCode = unsafe { $unsafe_block };
        match errcode {
            bindings::btrfs_util_error_BTRFS_UTIL_OK => Result::Ok(()),
            err => {
                let err = LibError::try_from(err).unwrap();
                Result::Err(err.into())
            }
        }
    }};
}
