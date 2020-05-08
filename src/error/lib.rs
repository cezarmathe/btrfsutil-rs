use crate::error::GlueError;
use crate::BtrfsUtilError;
use crate::Result;

#[cfg(feature = "enable-glue-errors")]
use std::convert::Into;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::os::raw::c_char;

use thiserror::Error;

/// Error code returned by the [libbtrfsutil] C library.
///
/// [libbtrfsutil]: https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil
pub(crate) type LibErrorCode = u32;

/// Errors that can be raised by the [libbtrfsutil] C library itself.
///
/// [libbtrfsutil]: https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LibError {
    /// Success
    #[error("Success")]
    Ok = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_OK as isize,
    /// Stop iteration
    #[error("Stop iteration")]
    StopIteration = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_STOP_ITERATION as isize,
    /// Cannot allocate memory
    #[error("Cannot allocate memory")]
    NoMemory = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_NO_MEMORY as isize,
    /// Invalid argument
    #[error("Invalid argument")]
    InvalidArgument = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_INVALID_ARGUMENT as isize,
    /// Not a Btrfs filesystem
    #[error("Not a Btrfs filesystem")]
    NotBtrfs = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_NOT_BTRFS as isize,
    /// Not a Btrfs subvolume
    #[error("Not a Btrfs subvolume")]
    NotSubvolume = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_NOT_SUBVOLUME as isize,
    /// Subvolume not found
    #[error("Subvolume not found")]
    SubvolumeNotFound = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOLUME_NOT_FOUND as isize,
    /// Could not open
    #[error("Could not open")]
    OpenFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_OPEN_FAILED as isize,
    /// Could nor rmdir
    #[error("Could not rmdir")]
    RmdirFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_RMDIR_FAILED as isize,
    /// Could not unlink
    #[error("Could not unlink")]
    UnlinkFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_UNLINK_FAILED as isize,
    /// Could not stat
    #[error("Could not stat")]
    StatFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_STAT_FAILED as isize,
    /// Could not statfs
    #[error("Could not statfs")]
    StatfsFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_STATFS_FAILED as isize,
    /// Could not search B-tree
    #[error("Could not search B-tree")]
    SearchFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SEARCH_FAILED as isize,
    /// Could not lookup inode
    #[error("Could not lookup inode")]
    InoLookupFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_INO_LOOKUP_FAILED as isize,
    /// Could not get subvolume flags
    #[error("Could not get subvolume flags")]
    SubvolGetflagsFailed =
        btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOL_GETFLAGS_FAILED as isize,
    /// Could not set subvolume flags
    #[error("Could not set subvolume flags")]
    SubvolSetflagsFailed =
        btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOL_SETFLAGS_FAILED as isize,
    /// Could not create subvolume
    #[error("Could not create subvolume")]
    SubvolCreateFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOL_CREATE_FAILED as isize,
    /// Could not create snapshot
    #[error("Could not create snapshot")]
    SnapCreateFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SNAP_CREATE_FAILED as isize,
    /// Could not destroy subvolume/snapshot
    #[error("Could not destroy subvolume/snapshot")]
    SnapDestroyFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SNAP_DESTROY_FAILED as isize,
    /// Could not set default subvolume
    #[error("Could not set default subvolume")]
    DefaultSubvolFailed =
        btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_DEFAULT_SUBVOL_FAILED as isize,
    /// Could not sync filesystem
    #[error("Could not sync filesystem")]
    SyncFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SYNC_FAILED as isize,
    /// Could not start filesystem sync
    #[error("Could not start filesystem sync")]
    StartSyncFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_START_SYNC_FAILED as isize,
    /// Could not wait for filesystem sync
    #[error("Could not wait for filesystem sync")]
    WaitSyncFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_WAIT_SYNC_FAILED as isize,
    /// Could not get subvolume information with BTRFS_IOC_GET_SUBVOL_INFO
    #[error("Could not get subvolume information with BTRFS_IOC_GET_SUBVOL_INFO")]
    GetSubvolInfoFailed =
        btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_GET_SUBVOL_INFO_FAILED as isize,
    /// Could not get rootref information with BTRFS_IOC_GET_SUBVOL_ROOTREF
    #[error("Could not get rootref information with BTRFS_IOC_GET_SUBVOL_ROOTREF")]
    GetSubvolRootrefFailed =
        btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_GET_SUBVOL_ROOTREF_FAILED as isize,
    /// Could not resolve subvolume path with BTRFS_IOC_INO_LOOKUP_USER
    #[error("Could not resolve subvolume path with BTRFS_IOC_INO_LOOKUP_USER")]
    InoLookupUserFailed =
        btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_INO_LOOKUP_USER_FAILED as isize,
    /// Could not get filesystem information
    #[error("Could not get filesystem information")]
    FsInfoFailed = btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_FS_INFO_FAILED as isize,
}

impl LibError {
    /// Get the string description of a [LibError], using the [btrfs_util_strerror()] function
    /// provided by [libbtrfsutil].
    ///
    /// [LibError] provides an [std::fmt::Display] implementation provided by [thiserror]. This
    /// function should not be the preferred mechanism for obtaining the error message.
    ///
    /// [LibError]: enum.LibError.html
    /// [btrfs_util_strerror()]: ../btrfsutil_sys/fn.btrfs_util_strerror.html
    /// [std::fmt::Display]: https://doc.rust-lang.org/stable/std/fmt/trait.Display.html
    /// [thiserror]: https://docs.rs/thiserror/1.0.16/thiserror/
    /// [libbtrfsutil]: https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil
    pub fn strerror(&self) -> Result<&'static str> {
        let err_str_ptr: *const c_char;

        let errno = self.clone() as LibErrorCode;
        unsafe {
            err_str_ptr = btrfsutil_sys::btrfs_util_strerror(errno);
        }

        glue_error!(err_str_ptr.is_null(), GlueError::NullPointerReceived);

        let cstr: &CStr = unsafe { CStr::from_ptr(err_str_ptr) };
        match cstr.to_str() {
            Ok(val) => Ok(val),
            Err(e) => glue_error!(GlueError::Utf8Error(e)),
        }
    }
}

impl TryFrom<LibErrorCode> for LibError {
    type Error = BtrfsUtilError;
    fn try_from(errno: LibErrorCode) -> Result<Self> {
        glue_error!(errno > 26, GlueError::UnknownErrno(errno));
        match errno {
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_OK => Ok(LibError::Ok),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_STOP_ITERATION => {
                Ok(LibError::StopIteration)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_NO_MEMORY => Ok(LibError::NoMemory),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_INVALID_ARGUMENT => {
                Ok(LibError::InvalidArgument)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_NOT_BTRFS => Ok(LibError::NotBtrfs),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_NOT_SUBVOLUME => Ok(LibError::NotSubvolume),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOLUME_NOT_FOUND => {
                Ok(LibError::SubvolumeNotFound)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_OPEN_FAILED => Ok(LibError::OpenFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_RMDIR_FAILED => Ok(LibError::RmdirFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_UNLINK_FAILED => Ok(LibError::UnlinkFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_STAT_FAILED => Ok(LibError::StatFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_STATFS_FAILED => Ok(LibError::StatfsFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SEARCH_FAILED => Ok(LibError::SearchFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_INO_LOOKUP_FAILED => {
                Ok(LibError::InoLookupFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOL_GETFLAGS_FAILED => {
                Ok(LibError::SubvolGetflagsFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOL_SETFLAGS_FAILED => {
                Ok(LibError::SubvolSetflagsFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SUBVOL_CREATE_FAILED => {
                Ok(LibError::SubvolCreateFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SNAP_CREATE_FAILED => {
                Ok(LibError::SnapCreateFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SNAP_DESTROY_FAILED => {
                Ok(LibError::SnapDestroyFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_DEFAULT_SUBVOL_FAILED => {
                Ok(LibError::DefaultSubvolFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_SYNC_FAILED => Ok(LibError::SyncFailed),
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_START_SYNC_FAILED => {
                Ok(LibError::StartSyncFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_WAIT_SYNC_FAILED => {
                Ok(LibError::WaitSyncFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_GET_SUBVOL_INFO_FAILED => {
                Ok(LibError::GetSubvolInfoFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_GET_SUBVOL_ROOTREF_FAILED => {
                Ok(LibError::GetSubvolRootrefFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_INO_LOOKUP_USER_FAILED => {
                Ok(LibError::InoLookupUserFailed)
            }
            btrfsutil_sys::btrfs_util_error_BTRFS_UTIL_ERROR_FS_INFO_FAILED => {
                Ok(LibError::FsInfoFailed)
            }
            _ => glue_error!(GlueError::UnknownErrno(errno)),
        }
    }
}

#[cfg(feature = "enable-glue-errors")]
impl Into<BtrfsUtilError> for LibError {
    fn into(self) -> BtrfsUtilError {
        BtrfsUtilError::Lib(self)
    }
}
