use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;
use std::path::PathBuf;

/// Convert a Path into a CString safely.
#[inline]
pub(crate) fn path_to_cstr(path: &Path) -> CString {
    // unwrapping here is safe since on unix systems strings are natively held inside cstrings
    CString::new(path.as_os_str().as_bytes()).unwrap()
}

/// Convert a Path into a CString safely.
#[inline]
pub(crate) fn cstr_to_path(path: &CStr) -> PathBuf {
    PathBuf::from(OsString::from_vec(path.to_bytes().into()))
}

/// Macro for preparing for an unsafe function execution and reacting to its
/// error code
macro_rules! unsafe_wrapper {
    ($unsafe_block: block) => {{
        let errcode: LibErrorCode = unsafe { $unsafe_block };
        match errcode {
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_OK => Result::Ok(()),
            err => {
                let err = LibError::try_from(err).unwrap();
                Result::Err(err.into())
            }
        }
    }};
}
