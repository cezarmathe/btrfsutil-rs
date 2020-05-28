//! Btrfs subvolumes

#[macro_use]
mod iterator;
mod privileged;
mod subvol_info;

pub use iterator::*;
pub use privileged::*;
pub use subvol_info::*;

use crate::common;
use crate::error::GlueError;
use crate::qgroup::QgroupInherit;
use crate::Result;

use std::ffi::CString;
use std::path::Path;
use std::path::PathBuf;

use btrfsutil_sys::btrfs_util_create_snapshot;
use btrfsutil_sys::btrfs_util_create_subvolume;
use btrfsutil_sys::btrfs_util_get_subvolume_read_only;
use btrfsutil_sys::btrfs_util_is_subvolume;
use btrfsutil_sys::btrfs_util_qgroup_inherit;
use btrfsutil_sys::btrfs_util_subvolume_path;

bitflags! {
    /// Subvolume delete flags.
    pub struct DeleteFlags: i32 {
        /// Recursive.
        const RECURSIVE = btrfsutil_sys::BTRFS_UTIL_DELETE_SUBVOLUME_RECURSIVE as i32;
    }
}
bitflags! {
    /// Subvolume snapshot flags.
    pub struct SnapshotFlags: i32 {
        /// Read-only.
        const READ_ONLY	= btrfsutil_sys::BTRFS_UTIL_CREATE_SNAPSHOT_READ_ONLY as i32;
        /// Recursive.
        const RECURSIVE = btrfsutil_sys::BTRFS_UTIL_CREATE_SNAPSHOT_RECURSIVE as i32;
    }
}

/// The representation of a Btrfs subvolume
pub struct Subvolume {
    id: u64,
    path: PathBuf,
}

impl Subvolume {
    /// Create a new subvolume.
    pub fn create<T: Into<PathBuf> + Clone>(
        path: T,
        qgroup: Option<QgroupInherit>,
    ) -> Result<Self> {
        let path_cstr = common::into_path_to_cstr(path.clone())?;
        let qgroup_ptr: *mut btrfs_util_qgroup_inherit =
            if_let_some!(qgroup, val, val.into(), std::ptr::null_mut());

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_create_subvolume(
                path_cstr.as_ptr(),
                0,
                std::ptr::null_mut(),
                qgroup_ptr,
            );
        });

        Self::get(path)
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_ro(&self) -> Result<bool> {
        let path_cstr = common::path_to_cstr(self.path()?)?;
        let mut ro: bool = false;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_get_subvolume_read_only(path_cstr.as_ptr(), &mut ro);
        });

        Ok(ro)
    }

    /// Check if a path is a Btrfs subvolume.
    pub fn is_subvolume<T: Into<PathBuf>>(path: T) -> Result<()> {
        let path_cstr = common::into_path_to_cstr(path)?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_is_subvolume(path_cstr.as_ptr());
        });

        Ok(())
    }

    /// Get the path of this subvolume relative to the filesystem root.
    pub fn path(&self) -> Result<PathBuf> {
        let path_cstr = common::into_path_to_cstr("/")?;
        let mut str_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_path(path_cstr.as_ptr(), self.id, &mut str_ptr);
        });

        glue_error!(str_ptr.is_null(), GlueError::NullPointerReceived);

        let cstr: CString = unsafe { CString::from_raw(str_ptr) };
        match cstr.to_str() {
            Ok(val) => Ok(PathBuf::from(format!("/{}", val))),
            Err(e) => glue_error!(GlueError::Utf8Error(e)),
        }
    }

    pub fn path_ref(&self) -> &Path {
        self.path.as_path()
    }

    /// Create a snapshot of this subvolume.
    fn snapshot<T: Into<PathBuf> + Clone>(
        &self,
        path: T,
        flags: Option<SnapshotFlags>,
        qgroup: Option<QgroupInherit>,
    ) -> Result<Subvolume> {
        let path_src_cstr = common::path_to_cstr(self.path()?)?;
        let path_dest_cstr = common::into_path_to_cstr(path.clone())?;
        let flags_val = if_let_some!(flags, val, val.bits(), 0);
        let qgroup_ptr: *mut btrfs_util_qgroup_inherit =
            if_let_some!(qgroup, val, val.into(), std::ptr::null_mut());

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_create_snapshot(
                path_src_cstr.as_ptr(),
                path_dest_cstr.as_ptr(),
                flags_val,
                std::ptr::null_mut(), // should be changed in the future for async support
                qgroup_ptr,
            );
        });

        Ok(Self::get(path)?)
    }
}
