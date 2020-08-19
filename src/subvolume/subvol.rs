use crate::bindings;
use crate::common;
use crate::error::GlueError;
use crate::error::LibError;
use crate::error::LibErrorCode;
use crate::qgroup::QgroupInherit;
use crate::subvolume::SubvolumeInfo;
use crate::subvolume::SubvolumeIterator;
use crate::Result;

use std::convert::TryFrom;
use std::ffi::CString;
use std::path::{Path, PathBuf};

use bindings::btrfs_util_create_snapshot;
use bindings::btrfs_util_create_subvolume;
use bindings::btrfs_util_delete_subvolume;
use bindings::btrfs_util_deleted_subvolumes;
use bindings::btrfs_util_get_default_subvolume;
use bindings::btrfs_util_get_subvolume_read_only;
use bindings::btrfs_util_is_subvolume;
use bindings::btrfs_util_set_default_subvolume;
use bindings::btrfs_util_set_subvolume_read_only;
use bindings::btrfs_util_subvolume_id;
use bindings::btrfs_util_subvolume_path;

bitflags! {
    /// Subvolume delete flags.
    pub struct DeleteFlags: i32 {
        /// Recursive.
        const RECURSIVE = bindings::BTRFS_UTIL_DELETE_SUBVOLUME_RECURSIVE as i32;
    }
}
bitflags! {
    /// Subvolume snapshot flags.
    pub struct SnapshotFlags: i32 {
        /// Read-only.
        const READ_ONLY	= bindings::BTRFS_UTIL_CREATE_SNAPSHOT_READ_ONLY as i32;
        /// Recursive.
        const RECURSIVE = bindings::BTRFS_UTIL_CREATE_SNAPSHOT_RECURSIVE as i32;
    }
}

/// A Btrfs subvolume.
///
/// Internally, this contains just the id of the subvolume.
#[derive(Clone, Debug)]
pub struct Subvolume {
    id: u64,
    fs_root: PathBuf,
}

impl Subvolume {
    /// Create a new subvolume.
    pub fn create(path: &Path, qgroup: Option<QgroupInherit>) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path)?;
        let qgroup_ptr = qgroup.map(|v| v.into()).unwrap_or(std::ptr::null_mut());

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
        let path_cstr = common::path_to_cstr(self.fs_root())?;
        let flags_val = flags.map(|v| v.bits()).unwrap_or(0);

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_delete_subvolume(path_cstr.as_ptr(), flags_val);
        });

        Ok(())
    }

    /// Get a list of subvolumes which have been deleted but not yet cleaned up.
    pub fn deleted(fs_root: &Path) -> Result<Vec<Subvolume>> {
        let path_cstr = common::path_to_cstr(fs_root)?;
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
            for id in subvolume_ids {
                subvolumes.push(Subvolume::new(id, fs_root));
            }
            subvolumes
        };

        Ok(subvolumes)
    }

    /// Get the default subvolume
    pub fn get_default(path: &Path) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path)?;
        let mut id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_get_default_subvolume(path_cstr.as_ptr(), &mut id);
        });

        Ok(Subvolume::new(id, path))
    }

    /// Set this subvolume as the default subvolume.
    pub fn set_default(&self) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.fs_root)?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_set_default_subvolume(path_cstr.as_ptr(), self.id());
        });

        Ok(())
    }

    /// Check whether this subvolume is read-only.
    pub fn is_ro(&self) -> Result<bool> {
        let path_cstr = common::path_to_cstr(self.fs_root())?;
        let mut ro: bool = false;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_get_subvolume_read_only(path_cstr.as_ptr(), &mut ro);
        });

        Ok(ro)
    }

    /// Set whether this subvolume is read-only or not.
    pub fn set_ro(&self, ro: bool) -> Result<()> {
        let path_cstr = common::path_to_cstr(self.fs_root())?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_set_subvolume_read_only(path_cstr.as_ptr(), ro);
        });

        Ok(())
    }

    /// Get the subvolume for a certain path.
    pub fn get(path: &Path) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path)?;
        let id: *mut u64 = &mut 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_id(path_cstr.as_ptr(), id);
        });

        glue_error!(id.is_null(), GlueError::NullPointerReceived);

        let subvol_id: u64 = unsafe { *id };
        Ok(Subvolume::new(subvol_id, path))
    }

    /// Check if a path is a Btrfs subvolume.
    pub fn is_subvolume(path: &Path) -> Result<()> {
        let path_cstr = common::path_to_cstr(path)?;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_is_subvolume(path_cstr.as_ptr());
        });

        Ok(())
    }

    /// Get information about this subvolume.
    pub fn info(&self) -> Result<SubvolumeInfo> {
        SubvolumeInfo::try_from(self)
    }

    /// Get the path of this subvolume relative to the filesystem root.
    pub fn rel_path(&self) -> Result<PathBuf> {
        let path_cstr = common::path_to_cstr(&self.fs_root)?;
        let mut str_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_path(path_cstr.as_ptr(), self.id(), &mut str_ptr);
        });

        glue_error!(str_ptr.is_null(), GlueError::NullPointerReceived);

        let cstr: CString = unsafe { CString::from_raw(str_ptr) };
        match cstr.to_str() {
            Ok(val) => Ok(PathBuf::from(val)),
            Err(e) => glue_error!(GlueError::Utf8Error(e)),
        }
    }

    /// Get the absolute path of this subvolume
    pub fn abs_path(&self) -> Result<PathBuf> {
        let mut subpath: PathBuf = self.fs_root.to_owned();
        subpath.push(self.rel_path()?);
        Ok(subpath)
    }

    /// Create a snapshot of this subvolume.
    pub fn snapshot(
        &self,
        path: &Path,
        flags: Option<SnapshotFlags>,
        qgroup: Option<QgroupInherit>,
    ) -> Result<Self> {
        let path_src_cstr = common::path_to_cstr(self.fs_root())?;
        let path_dest_cstr = common::path_to_cstr(path.clone())?;
        let flags_val = flags.map(|v| v.bits()).unwrap_or(0);
        let qgroup_ptr = qgroup.map(|v| v.into()).unwrap_or(std::ptr::null_mut());

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

    /// Get the filesystem root of this subvolume.
    pub fn fs_root(&self) -> &Path {
        &self.fs_root
    }

    /// Create a new Subvolume from an id.
    ///
    /// Restricted to the crate.
    pub(crate) fn new(id: u64, fs_root: &Path) -> Self {
        Self {
            id,
            fs_root: fs_root.to_owned(),
        }
    }
}

impl Into<Result<SubvolumeIterator>> for Subvolume {
    fn into(self) -> Result<SubvolumeIterator> {
        SubvolumeIterator::create(self, None)
    }
}
