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

use btrfsutil_sys::btrfs_util_create_snapshot;
use btrfsutil_sys::btrfs_util_create_subvolume;
use btrfsutil_sys::btrfs_util_delete_subvolume;
use btrfsutil_sys::btrfs_util_deleted_subvolumes;
use btrfsutil_sys::btrfs_util_get_default_subvolume;
use btrfsutil_sys::btrfs_util_get_subvolume_read_only;
use btrfsutil_sys::btrfs_util_is_subvolume;
use btrfsutil_sys::btrfs_util_set_default_subvolume;
use btrfsutil_sys::btrfs_util_set_subvolume_read_only;
use btrfsutil_sys::btrfs_util_subvolume_id;
use btrfsutil_sys::btrfs_util_subvolume_path;

use libc::{c_void, free};

bitflags! {
    /// [Subvolume] delete flags.
    ///
    /// [Subvolume]:struct.Subvolume.html
    pub struct DeleteFlags: i32 {
        /// Recursive.
        const RECURSIVE = btrfsutil_sys::BTRFS_UTIL_DELETE_SUBVOLUME_RECURSIVE as i32;
    }
}
bitflags! {
    /// [Subvolume] snapshot flags.
    ///
    /// [Subvolume]:struct.Subvolume.html
    pub struct SnapshotFlags: i32 {
        /// Read-only.
        const READ_ONLY	= btrfsutil_sys::BTRFS_UTIL_CREATE_SNAPSHOT_READ_ONLY as i32;
        /// Recursive.
        const RECURSIVE = btrfsutil_sys::BTRFS_UTIL_CREATE_SNAPSHOT_RECURSIVE as i32;
    }
}

/// A Btrfs subvolume.
#[derive(Clone, Debug, PartialEq)]
pub struct Subvolume {
    id: u64,
    path: PathBuf,
}

impl Subvolume {
    /// Get a subvolume.
    ///
    /// The path must point to the root of a subvolume.
    pub fn get<'a, P>(path: P) -> Result<Self>
    where
        P: Into<&'a Path>,
    {
        Self::get_impl(path.into())
    }

    fn get_impl(path: &Path) -> Result<Self> {
        let _ = Self::is_subvolume(path)?;

        let path_cstr = common::path_to_cstr(path);
        let id: u64 = {
            let mut id: u64 = 0;
            unsafe_wrapper!({ btrfs_util_subvolume_id(path_cstr.as_ptr(), &mut id) })?;
            id
        };

        Ok(Subvolume::new(id, path.into()))
    }

    /// Get a subvolume anyway.
    ///
    /// If the path is not the root of a subvolume, attempts to use btrfs_util_subvolume_path to
    /// get it, which requires **CAP_SYS_ADMIN**.
    ///
    /// ![Requires **CAP_SYS_ADMIN**](https://img.shields.io/static/v1?label=Requires&message=CAP_SYS_ADMIN&color=informational)
    pub fn get_anyway<'a, P>(path: P) -> Result<Self>
    where
        P: Into<&'a Path>,
    {
        Self::get_anyway_impl(path.into())
    }

    fn get_anyway_impl(path: &Path) -> Result<Self> {
        if let Ok(subvol) = Self::get_impl(path) {
            return Ok(subvol);
        }

        let path_cstr = common::path_to_cstr(path);
        let id: u64 = {
            let mut id: u64 = 0;
            unsafe_wrapper!({ btrfs_util_subvolume_id(path_cstr.as_ptr(), &mut id) })?;
            id
        };

        let mut path_ret_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!({ btrfs_util_subvolume_path(path_cstr.as_ptr(), id, &mut path_ret_ptr) })?;

        let path_ret: CString = unsafe { CString::from_raw(path_ret_ptr) };

        Ok(Self::new(id, common::cstr_to_path(&path_ret)))
    }

    /// Create a new subvolume.
    pub fn create<'a, P, Q>(path: P, qgroup: Q) -> Result<Self>
    where
        P: Into<&'a Path>,
        Q: Into<Option<QgroupInherit>>,
    {
        Self::create(path.into(), qgroup.into())
    }

    fn create_impl(path: &Path, qgroup: Option<QgroupInherit>) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path);
        let qgroup_ptr = qgroup.map(|v| v.into()).unwrap_or(std::ptr::null_mut());

        unsafe_wrapper!({
            btrfs_util_create_subvolume(
                path_cstr.as_ptr(),
                0,
                std::ptr::null_mut(), /* make use of the async transid and wait on it later */
                qgroup_ptr,
            )
        })?;

        Self::get(path)
    }

    /// Delete a subvolume.
    pub fn delete<D>(self, flags: D) -> Result<()>
    where
        D: Into<Option<DeleteFlags>>,
    {
        Self::delete_impl(self, flags.into())
    }

    fn delete_impl(self, flags: Option<DeleteFlags>) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.path);
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

    /// Get the default subvolume.
    ///
    /// ![Requires **CAP_SYS_ADMIN**](https://img.shields.io/static/v1?label=Requires&message=CAP_SYS_ADMIN&color=informational)
    pub fn get_default<'a, P>(path: P) -> Result<Self>
    where
        P: Into<&'a Path>,
    {
        Self::get_default_impl(path.into())
    }

    fn get_default_impl(path: &Path) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path);
        let mut id: u64 = 0;

        unsafe_wrapper!({ btrfs_util_get_default_subvolume(path_cstr.as_ptr(), &mut id) })?;

        Ok(Subvolume::new(id, path.into()))
    }

    /// Set this subvolume as the default subvolume.
    ///
    /// ![Requires **CAP_SYS_ADMIN**](https://img.shields.io/static/v1?label=Requires&message=CAP_SYS_ADMIN&color=informational)
    pub fn set_default(&self) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.path);

        unsafe_wrapper!({ btrfs_util_set_default_subvolume(path_cstr.as_ptr(), self.id) })?;

        Ok(())
    }

    /// Check whether this subvolume is read-only.
    pub fn is_ro(&self) -> Result<bool> {
        let path_cstr = common::path_to_cstr(&self.path);
        let ro: bool = {
            let mut ro = false;
            unsafe_wrapper!({ btrfs_util_get_subvolume_read_only(path_cstr.as_ptr(), &mut ro) })?;
            ro
        };

        Ok(ro)
    }

    /// Set whether this subvolume is read-only or not.
    ///
    /// ![Requires **CAP_SYS_ADMIN**](https://img.shields.io/static/v1?label=Requires&message=CAP_SYS_ADMIN&color=informational)
    pub fn set_ro(&self, ro: bool) -> Result<()> {
        let path_cstr = common::path_to_cstr(&self.path);

        unsafe_wrapper!({ btrfs_util_set_subvolume_read_only(path_cstr.as_ptr(), ro) })?;

        Ok(())
    }

    /// Check if a path is a Btrfs subvolume.
    ///
    /// Returns Ok if it is a subvolume or Err if otherwise.
    pub fn is_subvolume<'a, P>(path: P) -> Result<()>
    where
        P: Into<&'a Path>,
    {
        Self::is_subvolume_impl(path.into())
    }

    fn is_subvolume_impl(path: &Path) -> Result<()> {
        let path_cstr = common::path_to_cstr(path);

        unsafe_wrapper!({ btrfs_util_is_subvolume(path_cstr.as_ptr()) })
    }

    /// Get information about this subvolume.
    pub fn info(&self) -> Result<SubvolumeInfo> {
        SubvolumeInfo::try_from(self)
    }

    /// Create a snapshot of this subvolume.
    pub fn snapshot<'a, P, F, Q>(&self, path: P, flags: F, qgroup: Q) -> Result<Self>
    where
        P: Into<&'a Path>,
        F: Into<Option<SnapshotFlags>>,
        Q: Into<Option<QgroupInherit>>,
    {
        self.snapshot_impl(path.into(), flags.into(), qgroup.into())
    }

    fn snapshot_impl(
        &self,
        path: &Path,
        flags: Option<SnapshotFlags>,
        qgroup: Option<QgroupInherit>,
    ) -> Result<Self> {
        let path_src_cstr = common::path_to_cstr(&self.path);
        let path_dest_cstr = common::path_to_cstr(path);
        let flags_val = flags.map(|v| v.bits()).unwrap_or(0);
        let qgroup_ptr = qgroup.map(|v| v.into()).unwrap_or(std::ptr::null_mut());

        unsafe_wrapper!({
            btrfs_util_create_snapshot(
                path_src_cstr.as_ptr(),
                path_dest_cstr.as_ptr(),
                flags_val,
                std::ptr::null_mut(), /* make use of the async transid and wait on it later */
                qgroup_ptr,
            )
        })?;

        Self::get(path)
    }

    /// Get the id of this subvolume.
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the path of this subvolume.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a new subvolume from an id and a path.
    ///
    /// Restricted to the crate.
    #[inline]
    pub(crate) fn new(id: u64, path: PathBuf) -> Self {
        Self {
            id,
            path: path.into(),
        }
    }
}

impl Into<u64> for &Subvolume {
    /// Returns the id of the subvolume.
    #[inline]
    fn into(self) -> u64 {
        self.id
    }
}

impl TryFrom<u64> for Subvolume {
    type Error = LibError;

    /// Attempts to get a subvolume from an id.
    ///
    /// This function will panic if it cannot retrieve the current working directory.
    ///
    /// ![Requires **CAP_SYS_ADMIN**](https://img.shields.io/static/v1?label=Requires&message=CAP_SYS_ADMIN&color=informational)
    fn try_from(src: u64) -> Result<Subvolume> {
        let path_cstr: CString = common::path_to_cstr(
            std::env::current_dir()
                .expect("Could not get the current working directory")
                .as_ref(),
        );
        let mut path_ret_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();

        unsafe_wrapper!({ btrfs_util_subvolume_path(path_cstr.as_ptr(), src, &mut path_ret_ptr) })?;

        let path_ret: CString = unsafe { CString::from_raw(path_ret_ptr) };

        Ok(Self::new(src, common::cstr_to_path(&path_ret)))
    }
}

impl Into<PathBuf> for &Subvolume {
    /// Returns the path of the subvolume.
    #[inline]
    fn into(self) -> PathBuf {
        self.path
    }
}

impl TryFrom<&Path> for Subvolume {
    type Error = LibError;

    /// Attempts to get a subvolume from a path.
    #[inline]
    fn try_from(src: &Path) -> Result<Subvolume> {
        Subvolume::get_impl(src)
    }
}

impl TryFrom<PathBuf> for Subvolume {
    type Error = LibError;

    /// Attempts to get a subvolume from a path.
    #[inline]
    fn try_from(src: PathBuf) -> Result<Subvolume> {
        Subvolume::get_impl(src.as_ref())
    }
}

impl Into<Result<SubvolumeIterator>> for Subvolume {
    fn into(self) -> Result<SubvolumeIterator> {
        SubvolumeIterator::create(self, None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fs::{create_dir_all, OpenOptions};
    use std::path::Path;

    use nix::mount::{mount, MsFlags};

    use crate::testing::{btrfs_create_fs, test_with_spec};
    use btrfsutil_sys::BTRFS_FS_TREE_OBJECTID;

    fn test_btrfs_subvol(paths: &[&Path]) {
        // Create btrfs filesystem on loopback device
        btrfs_create_fs(paths[0]).unwrap();

        // Create mount point and mount
        let mount_pt = Path::new("/tmp/btrfsutil/mnt");
        create_dir_all(mount_pt).unwrap();
        mount(
            Some(paths[0]),
            mount_pt,
            Some("btrfs"),
            MsFlags::empty(),
            None as Option<&str>,
        )
        .unwrap();

        let root_subvol = Subvolume::from_path(mount_pt).unwrap();
        assert_eq!(root_subvol.id(), BTRFS_FS_TREE_OBJECTID);

        let mut new_sv_path = mount_pt.to_owned();
        new_sv_path.push("subvol1");
        let sv1 = Subvolume::create(&new_sv_path, None).unwrap();

        // Test rel_path()
        let sv1_rel_path = sv1.rel_path().unwrap();
        assert_eq!(&sv1_rel_path, Path::new("subvol1"));

        // Test abs_path()
        let sv1_abs_path = sv1.abs_path().unwrap();
        assert_eq!(&sv1_abs_path, &new_sv_path);

        // Test get_default
        let default_sv = Subvolume::get_default(mount_pt).unwrap();
        assert_eq!(default_sv, root_subvol);

        // Test set_default
        sv1.set_default().unwrap();
        let new_default_sv = Subvolume::get_default(mount_pt).unwrap();
        assert_eq!(sv1, new_default_sv);
        assert_eq!(new_default_sv.abs_path().unwrap(), new_sv_path);

        // Restore root as default
        root_subvol.set_default().unwrap();

        let info = root_subvol.info().unwrap();
        assert_eq!(info.id, BTRFS_FS_TREE_OBJECTID);
        assert_eq!(info.parent_id, None);
        assert_eq!(info.dir_id, None);
        assert_eq!(info.parent_uuid, None);
        assert_eq!(info.received_uuid, None);

        // Test cannot write to readonly subvolume
        assert_eq!(false, sv1.is_ro().unwrap());
        sv1.set_ro(true).unwrap();
        let mut file_path = sv1_abs_path.clone();
        file_path.push("file.txt");
        assert!(OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .is_err());

        // Can now create a file
        sv1.set_ro(false).unwrap();
        assert!(OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .is_ok());

        // Test is_subvolume
        assert_eq!(true, Subvolume::is_subvolume(mount_pt).unwrap());
        assert_eq!(true, Subvolume::is_subvolume(&new_sv_path).unwrap());
        // Existing non-btrfs path
        assert_eq!(false, Subvolume::is_subvolume(Path::new("/tmp")).unwrap());
        // Nonexistent path
        assert_eq!(
            false,
            Subvolume::is_subvolume(Path::new("/foobar")).unwrap()
        );

        let mut dir_path = sv1_abs_path.clone();
        dir_path.push("dir1");
        create_dir_all(&dir_path).unwrap();
        // A directory within a subvolume is not a subvolume
        assert_eq!(false, Subvolume::is_subvolume(&dir_path).unwrap());

        // Test making a snapshot
        let mut snap_path = mount_pt.to_owned();
        snap_path.push("snap1");
        let snap_sv1 = sv1.snapshot(&snap_path, None, None).unwrap();
        let mut snap_file_path = snap_path;
        snap_file_path.push("file.txt");

        // File from orig also in snap
        assert!(OpenOptions::new().read(true).open(&snap_file_path).is_ok());

        // Test subvol deletion
        let snap_id = snap_sv1.info().unwrap().id;
        snap_sv1.delete(None).unwrap();

        let deleted = Subvolume::deleted(mount_pt).unwrap();
        assert_eq!(1, deleted.len());
        assert_eq!(snap_id, deleted[0].id());
    }

    #[test]
    fn loop_test_btrfs_subvol() {
        test_with_spec(1, test_btrfs_subvol);
    }
}
