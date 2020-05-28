use crate::common;
use crate::error::*;
use crate::subvolume::DeleteFlags;
use crate::subvolume::Subvolume;
use crate::subvolume::SubvolumeInfo;
use crate::subvolume::SubvolumeIterator;
use crate::Result;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::path::PathBuf;

use btrfsutil_sys::btrfs_util_delete_subvolume;
use btrfsutil_sys::btrfs_util_deleted_subvolumes;
use btrfsutil_sys::btrfs_util_get_default_subvolume;
use btrfsutil_sys::btrfs_util_set_default_subvolume;
use btrfsutil_sys::btrfs_util_set_subvolume_read_only;
use btrfsutil_sys::btrfs_util_subvolume_id;

/// The functionality provided by a PrivilegedSubvolume
pub trait PrivilegedSubvolume {
    /// Delete a subvolume.
    fn delete(self, flags: Option<DeleteFlags>) -> Result<()>;

    /// Get a list of subvolumes which have been deleted but not yet cleaned up.
    fn deleted<T: Into<PathBuf>>(path: Option<T>) -> Result<Vec<Subvolume>>;

    /// Get the default subvolume
    fn get_default<T: Into<PathBuf>>(path: Option<T>) -> Result<Subvolume>;

    /// Set this subvolume as the default subvolume.
    fn set_default(&self) -> Result<()>;

    /// Set whether this subvolume is read-only or not.
    fn set_ro(&self, ro: bool) -> Result<()>;

    /// Get the subvolume for a certain path.
    fn get<T: Into<PathBuf>>(path: T) -> Result<Subvolume>;

    /// Get information about this subvolume.
    fn info(&self) -> Result<SubvolumeInfo>;
}

impl PrivilegedSubvolume for Subvolume {
    fn delete(self, flags: Option<DeleteFlags>) -> Result<()> {
        let path_cstr = common::path_to_cstr(self.path()?)?;
        let flags_val = if_let_some!(flags, val, val.bits(), 0);

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_delete_subvolume(path_cstr.as_ptr(), flags_val);
        });

        Ok(())
    }

    fn deleted<T: Into<PathBuf>>(path: Option<T>) -> Result<Vec<Subvolume>> {
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

    fn get_default<T: Into<PathBuf>>(path: Option<T>) -> Result<Self> {
        let path_cstr = common::optional_into_path_to_cstr(path)?;
        let mut id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_get_default_subvolume(path_cstr.as_ptr(), &mut id);
        });

        Ok(Subvolume(id))
    }

    fn set_default(&self) -> Result<()> {
        let path_cstr = common::into_path_to_cstr("/")?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_set_default_subvolume(path_cstr.as_ptr(), self.0);
        });

        Ok(())
    }



    /// Set whether this subvolume is read-only or not.
    fn set_ro(&self, ro: bool) -> Result<()> {
        let path_cstr = common::path_to_cstr(self.path()?)?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_set_subvolume_read_only(path_cstr.as_ptr(), ro);
        });

        Ok(())
    }

    /// Get the subvolume for a certain path.
    fn get<T: Into<PathBuf>>(path: T) -> Result<Self> {
        let path_cstr = common::into_path_to_cstr(path)?;
        let subvol_id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_id(path_cstr.as_ptr(), &mut subvol_id);
        });

        let subvolume = Self{
            id: subvol_id,
            path: subvolume::path()?,
        };

        Ok(subvolume)
    }

    /// Get information about this subvolume.
    fn info(&self) -> Result<SubvolumeInfo> {
        SubvolumeInfo::try_from(self)
    }
}

impl TryInto<SubvolumeIterator> for dyn PrivilegedSubvolume {
    type Error = BtrfsUtilError;
    fn try_into(self) -> Result<SubvolumeIterator> {
        SubvolumeIterator::create(self, None)
    }
}
