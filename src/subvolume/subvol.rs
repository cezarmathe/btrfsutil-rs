use crate::common;
use crate::error::*;
use crate::qgroup::QgroupInherit;
use crate::subvolume::SubvolumeInfo;
use crate::subvolume::SubvolumeIterator;
use crate::subvolume::SubvolumePath;
use crate::Result;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;
use std::path::PathBuf;

use btrfsutil_sys::btrfs_util_create_snapshot;
use btrfsutil_sys::btrfs_util_create_subvolume;
use btrfsutil_sys::btrfs_util_delete_subvolume;
use btrfsutil_sys::btrfs_util_deleted_subvolumes;
use btrfsutil_sys::btrfs_util_get_default_subvolume;
use btrfsutil_sys::btrfs_util_get_subvolume_read_only;
use btrfsutil_sys::btrfs_util_is_subvolume;
use btrfsutil_sys::btrfs_util_qgroup_inherit;
use btrfsutil_sys::btrfs_util_set_default_subvolume;
use btrfsutil_sys::btrfs_util_set_subvolume_read_only;
use btrfsutil_sys::btrfs_util_subvolume_id;
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

/// A Btrfs subvolume.
///
/// This struct contains the id of this subvolume and a path to it.
#[derive(Clone, Debug)]
pub struct Subvolume {
    id: u64,
    path: SubvolumePath,
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

    /// Delete a subvolume.
    pub fn delete(self, flags: Option<DeleteFlags>) -> Result<()> {
        let path_cstr = common::path_to_cstr(self.path()?)?;
        let flags_val = if_let_some!(flags, val, val.bits(), 0);

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_delete_subvolume(path_cstr.as_ptr(), flags_val);
        });

        Ok(())
    }

    /// Get a list of subvolumes which have been deleted but not yet cleaned up.
    pub fn deleted<T: Into<PathBuf>>(path: Option<T>) -> Result<Vec<Subvolume>> {
        let path_cstr = common::optional_into_path_to_cstr(path)?;
        let mut ids_ptr: *mut u64 = std::ptr::null_mut();
        let mut ids_count: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode =
                btrfs_util_deleted_subvolumes(path_cstr.as_ptr(), &mut ids_ptr, &mut ids_count);
        });

        if ids_count == 0 {
            return Ok(Vec::new());
        }

        glue_error!(ids_ptr.is_null(), GlueError::NullPointerReceived);

        let subvolume_ids: Vec<u64> =
            unsafe { std::slice::from_raw_parts(ids_ptr, ids_count as usize).to_owned() };

        let subvolumes: Vec<Subvolume> = {
            let mut subvolumes: Vec<Subvolume> = Vec::with_capacity(ids_count as usize);
            for item in subvolume_ids {
                subvolumes.push(Subvolume(item));
            }
            subvolumes
        };

        Ok(subvolumes)
    }

    /// Get the default subvolume
    pub fn get_default<T: Into<PathBuf>>(path: Option<T>) -> Result<Self> {
        let path_cstr = common::optional_into_path_to_cstr(path)?;
        let mut id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_get_default_subvolume(path_cstr.as_ptr(), &mut id);
        });

        Ok(Subvolume(id))
    }

    /// Set this subvolume as the default subvolume.
    pub fn set_default(&self) -> Result<()> {
        let path_cstr = common::into_path_to_cstr("/")?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_set_default_subvolume(path_cstr.as_ptr(), self.0);
        });

        Ok(())
    }

    /// Check whether this subvolume is read-only.
    pub fn is_ro(&self) -> Result<bool> {
        let path_cstr = common::path_to_cstr(self.path()?)?;
        let mut ro: bool = false;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_get_subvolume_read_only(path_cstr.as_ptr(), &mut ro);
        });

        Ok(ro)
    }

    /// Set whether this subvolume is read-only or not.
    pub fn set_ro(&self, ro: bool) -> Result<()> {
        let path_cstr = common::path_to_cstr(self.path()?)?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_set_subvolume_read_only(path_cstr.as_ptr(), ro);
        });

        Ok(())
    }

    #[cfg(any(
        feature = "subvol-path-no-confirm",
        feature = "subvol-path-try-confirm"
    ))]
    /// Get the subvolume for a certain path.
    pub fn get<T: Into<PathBuf>>(path: T) -> Result<Self> {
        let path_cstr = common::into_path_to_cstr(path)?;
        let subvol_id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_id(path_cstr.as_ptr(), &mut subvol_id);
        });

        let subvolume = Self{
            id: subvol_id,
            path: SubvolumePath::NotConfirmed(path.into()),
        };
        if cfg!(feature = "subvol-path-try-confirm") {
            let _ = subvolume.try_confirm_path();
        }

        Ok(subvolume)
    }

    #[cfg(any(feature = "subvol-path-relaxed", feature = "subvol-path-strict"))]
    /// Get the subvolume for a certain path.
    pub fn get<T: Into<PathBuf>>(path: T) -> Result<Self> {
        let path_cstr = common::into_path_to_cstr(path)?;
        let subvol_id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_id(path_cstr.as_ptr(), &mut subvol_id);
        });

        let subvolume = Self{
            id: subvol_id,
            path: path.into(),
        };
        if cfg!(feature = "subvol-path-strict") {
            subvolume.try_confirm_path()?;
        }

        Ok(subvolume)
    }

    /// Check if a path is a Btrfs subvolume.
    pub fn is_subvolume<T: Into<PathBuf>>(path: T) -> Result<()> {
        let path_cstr = common::into_path_to_cstr(path)?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_is_subvolume(path_cstr.as_ptr());
        });

        Ok(())
    }

    /// Get information about this subvolume.
    pub fn info(&self) -> Result<SubvolumeInfo> {
        SubvolumeInfo::try_from(self)
    }

    #[cfg(any(
        feature = "subvol-path-no-confirm",
        feature = "subvol-path-try-confirm"
    ))]
    /// Try to confirm the path to this Subvolume
    pub fn try_confirm_path(&self) -> Result<()> {
        if let SubvolumePath::Confirmed(_) = self.path {
            return Ok(());
        }
        let path_cstr = common::into_path_to_cstr("/")?;
        let mut str_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_path(path_cstr.as_ptr(), self.0, &mut str_ptr);
        });

        glue_error!(str_ptr.is_null(), GlueError::NullPointerReceived);

        let cstr: CString = unsafe { CString::from_raw(str_ptr) };
        match cstr.to_str() {
            Ok(val) => {
                self.path = SubvolumePath::Confirmed(PathBuf::from(format!("/{}", val)));
                Ok(())
            }
            Err(e) => glue_error!(GlueError::Utf8Error(e)),
        }
    }

    #[cfg(any(feature = "subvol-path-relaxed", feature = "subvol-path-strict"))]
    /// Try to confirm the path to this Subvolume
    pub fn try_confirm_path(&self) -> Result<()> {
        let path_cstr = common::into_path_to_cstr("/")?;
        let mut str_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_path(path_cstr.as_ptr(), self.0, &mut str_ptr);
        });

        glue_error!(str_ptr.is_null(), GlueError::NullPointerReceived);

        let cstr: CString = unsafe { CString::from_raw(str_ptr) };
        match cstr.to_str() {
            Ok(val) => {
                self.path = PathBuf::from(format!("/{}", val));
                Ok(())
            }
            Err(e) => glue_error!(GlueError::Utf8Error(e)),
        }
    }

    /// Create a snapshot of this subvolume.
    pub fn snapshot<T: Into<PathBuf> + Clone>(
        &self,
        path: T,
        flags: Option<SnapshotFlags>,
        qgroup: Option<QgroupInherit>,
    ) -> Result<Self> {
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

    /// Get the id of this subvolume.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Create a new Subvolume from an id.
    ///
    /// Restricted to the crate.
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }
}

impl TryInto<SubvolumeIterator> for Subvolume {
    type Error = BtrfsUtilError;
    fn try_into(self) -> Result<SubvolumeIterator> {
        SubvolumeIterator::create(self, None)
    }
}
