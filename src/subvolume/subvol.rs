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
use std::ffi::CStr;
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

use libc::{c_void, free};

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
#[derive(Clone, Debug, PartialEq)]
pub struct Subvolume {
    id: u64,
    fs_root: PathBuf,
}

impl Subvolume {
    /// Create a new subvolume.
    pub fn create(path: &Path, qgroup: Option<QgroupInherit>) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path)?;
        let qgroup_ptr = qgroup.map(|v| v.into()).unwrap_or(std::ptr::null_mut());

        unsafe_wrapper!({
            btrfs_util_create_subvolume(path_cstr.as_ptr(), 0, std::ptr::null_mut(), qgroup_ptr)
        })?;

        Self::from_path(path)
    }

    /// Delete a subvolume.
    pub fn delete(self, flags: Option<DeleteFlags>) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.abs_path()?)?;
        let flags_val = flags.map(|v| v.bits()).unwrap_or(0);

        unsafe_wrapper!({ btrfs_util_delete_subvolume(path_cstr.as_ptr(), flags_val) })?;

        Ok(())
    }

    /// Get a list of subvolumes which have been deleted but not yet cleaned up.
    pub fn deleted(fs_root: &Path) -> Result<Vec<Subvolume>> {
        let path_cstr = common::path_to_cstr(fs_root)?;
        let mut ids_ptr: *mut u64 = std::ptr::null_mut();
        let mut ids_count: u64 = 0;

        unsafe_wrapper!({
            btrfs_util_deleted_subvolumes(path_cstr.as_ptr(), &mut ids_ptr, &mut ids_count)
        })?;

        if ids_count == 0 {
            return Ok(Vec::new());
        }

        glue_error!(ids_ptr.is_null(), GlueError::NullPointerReceived);

        let subvolume_ids: Vec<u64> = unsafe {
            let v = std::slice::from_raw_parts(ids_ptr, ids_count as usize).to_owned();
            free(ids_ptr as *mut c_void);
            v
        };

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

        unsafe_wrapper!({ btrfs_util_get_default_subvolume(path_cstr.as_ptr(), &mut id) })?;

        Ok(Subvolume::new(id, path))
    }

    /// Set this subvolume as the default subvolume.
    pub fn set_default(&self) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.fs_root)?;

        unsafe_wrapper!({ btrfs_util_set_default_subvolume(path_cstr.as_ptr(), self.id()) })?;

        Ok(())
    }

    /// Check whether this subvolume is read-only.
    pub fn is_ro(&self) -> Result<bool> {
        let path_cstr = common::path_to_cstr(&self.abs_path()?)?;
        let mut ro: bool = false;

        unsafe_wrapper!({ btrfs_util_get_subvolume_read_only(path_cstr.as_ptr(), &mut ro) })?;

        Ok(ro)
    }

    /// Set whether this subvolume is read-only or not.
    pub fn set_ro(&self, ro: bool) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.abs_path()?)?;

        unsafe_wrapper!({ btrfs_util_set_subvolume_read_only(path_cstr.as_ptr(), ro) })?;

        Ok(())
    }

    /// Get the subvolume for a certain path.
    pub fn from_path(path: &Path) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path)?;
        let id: *mut u64 = &mut 0;

        unsafe_wrapper!({ btrfs_util_subvolume_id(path_cstr.as_ptr(), id) })?;

        glue_error!(id.is_null(), GlueError::NullPointerReceived);

        let id = unsafe { *id };

        let fs_root = Self::query_fs_root(path, id)?;
        Ok(Subvolume::new(id, &fs_root))
    }

    /// Check if a path is a Btrfs subvolume.
    pub fn is_subvolume(path: &Path) -> Result<bool> {
        let path_cstr = common::path_to_cstr(path)?;

        Ok(unsafe_wrapper!({ btrfs_util_is_subvolume(path_cstr.as_ptr()) }).is_ok())
    }

    /// Get information about this subvolume.
    pub fn info(&self) -> Result<SubvolumeInfo> {
        SubvolumeInfo::try_from(self)
    }

    /// Get the path of this subvolume relative to the filesystem root.
    fn subvol_path(path: &Path, id: u64) -> Result<PathBuf> {
        let path_cstr = common::path_to_cstr(path)?;
        let mut str_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!({ btrfs_util_subvolume_path(path_cstr.as_ptr(), id, &mut str_ptr) })?;

        glue_error!(str_ptr.is_null(), GlueError::NullPointerReceived);

        let cstr = unsafe { CStr::from_ptr(str_ptr) };
        let result = match cstr.to_str() {
            Ok(val) => Ok(PathBuf::from(val)),
            Err(e) => glue_error!(GlueError::Utf8Error(e)),
        };
        unsafe { free(str_ptr as *mut c_void) };
        result
    }

    /// Get the path of the filesystem's root mount point.
    fn query_fs_root(path: &Path, id: u64) -> Result<PathBuf> {
        let mut path_buf = path.to_owned();
        let mut subvol_path = Self::subvol_path(path, id)?;

        // Given path may include regular directories after subvolume. Discard
        // these until path ends with subvolume path.
        while !path_buf.ends_with(&subvol_path) {
            assert_eq!(true, path_buf.pop())
        }

        // Discard subvolume path segments to get filesystem root
        while subvol_path.pop() {
            path_buf.pop();
        }

        // What's left is the filesystem's root mount point.
        Ok(path_buf)
    }

    /// Get the path of this subvolume relative to the filesystem root.
    pub fn rel_path(&self) -> Result<PathBuf> {
        Self::subvol_path(self.fs_root(), self.id())
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
        let path_src_cstr = common::path_to_cstr(&self.abs_path()?)?;
        let path_dest_cstr = common::path_to_cstr(path)?;
        let flags_val = flags.map(|v| v.bits()).unwrap_or(0);
        let qgroup_ptr = qgroup.map(|v| v.into()).unwrap_or(std::ptr::null_mut());

        unsafe_wrapper!({
            btrfs_util_create_snapshot(
                path_src_cstr.as_ptr(),
                path_dest_cstr.as_ptr(),
                flags_val,
                std::ptr::null_mut(), // should be changed in the future for async support
                qgroup_ptr,
            )
        })?;

        Ok(Self::from_path(path)?)
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
