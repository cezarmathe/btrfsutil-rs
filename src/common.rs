use crate::error::GlueError;
use crate::Result;

use std::ffi::CString;
use std::path::PathBuf;

/// Convert an Into<PathBuf> into a CString.
#[inline]
pub(crate) fn into_path_to_cstr<T: Into<PathBuf>>(path: T) -> Result<CString> {
    path_to_cstr(path.into())
}

/// Convert a PathBuf into a CString.
#[inline]
pub(crate) fn path_to_cstr(path: PathBuf) -> Result<CString> {
    let path_str: &str;
    match path.to_str() {
        Some(val) => path_str = val,
        None => glue_error!(GlueError::BadPath(path)),
    }
    match CString::new(path_str) {
        Ok(val) => Ok(val),
        Err(e) => glue_error!(GlueError::NulError(e)),
    }
}

/// Convert an Option<Into<PathBuf>> to a CString.
#[inline]
pub(crate) fn optional_into_path_to_cstr<T: Into<PathBuf>>(path: Option<T>) -> Result<CString> {
    let path: PathBuf = if let Some(val) = path {
        val.into()
    } else {
        "/".into()
    };
    path_to_cstr(path)
}

/// Macro for simplifying an `if let Some(val) {} else {}` statement.
macro_rules! if_let_some {
    ($option: ident, $val_name: ident, $some: expr, $none: expr) => {
        if let Some($val_name) = $option {
            $some
        } else {
            $none
        }
    };
}

/// Macro for preparing for an unsafe function execution and reacting to it's error code
macro_rules! unsafe_wrapper {
    ($errcode: ident, $unsafe_block: block) => {
        let $errcode: LibErrorCode;
        unsafe { $unsafe_block }
        if $errcode > 0 {
            let err = LibError::try_from($errcode)?;
            return Result::Err(err.into());
        }
    };
}
