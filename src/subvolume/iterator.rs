use crate::common;
use crate::error::LibError;
use crate::subvolume::Subvolume;
use crate::Result;

use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CString;
use std::path::Path;

use btrfsutil_sys::btrfs_util_create_subvolume_iterator;
use btrfsutil_sys::btrfs_util_destroy_subvolume_iterator;
use btrfsutil_sys::btrfs_util_subvolume_iterator;
use btrfsutil_sys::btrfs_util_subvolume_iterator_next;

bitflags! {
    /// Subvolume iterator options
    pub struct SubvolumeIteratorFlags: i32 {
        /// Post order
        const POST_ORDER = btrfsutil_sys::BTRFS_UTIL_SUBVOLUME_ITERATOR_POST_ORDER as i32;
    }
}

/// A subvolume iterator.
pub struct SubvolumeIterator(*mut btrfs_util_subvolume_iterator);

impl SubvolumeIterator {
    /// Create a new subvolume iterator.
    pub fn new<'a, P, F>(path: P, flags: F) -> Result<Self>
    where
        P: Into<&'a Path>,
        F: Into<Option<SubvolumeIteratorFlags>>,
    {
        Self::new_impl(path.into(), flags.into())
    }

    fn new_impl(path: &Path, flags: Option<SubvolumeIteratorFlags>) -> Result<Self> {
        let path_cstr = common::path_to_cstr(path);
        let flags_val = if let Some(val) = flags { val.bits() } else { 0 };

        let raw_iterator_ptr: *mut btrfs_util_subvolume_iterator = {
            let mut raw_iterator_ptr: *mut btrfs_util_subvolume_iterator = std::ptr::null_mut();
            unsafe_wrapper!({
                btrfs_util_create_subvolume_iterator(
                    path_cstr.as_ptr(),
                    0, // read below
                    flags_val,
                    &mut raw_iterator_ptr,
                )
            })?;
            // using 0 instead of an id is intentional
            // https://github.com/kdave/btrfs-progs/blob/11acf45eea6dd81e891564967051e2bb10bd25f7/libbtrfsutil/subvolume.c#L971
            // if we specify an id then libbtrfsutil will use elevated privileges to search for
            // subvolumes
            // if we don't, then it will use elevated privileges only if the current user is root
            raw_iterator_ptr
        };

        Ok(Self(raw_iterator_ptr))
    }
}

impl Iterator for SubvolumeIterator {
    type Item = Result<Subvolume>;

    fn next(&mut self) -> Option<Result<Subvolume>> {
        let mut cstr_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut id: u64 = 0;

        if let Err(e) =
            unsafe_wrapper!({ btrfs_util_subvolume_iterator_next(self.0, &mut cstr_ptr, &mut id) })
        {
            if e == LibError::StopIteration {
                None
            } else {
                Err(e).into()
            }
        } else if !cstr_ptr.is_null() {
            let path = common::cstr_to_path(unsafe { CString::from_raw(cstr_ptr).as_ref() });
            Subvolume::get(path.as_path()).into()
        } else if id != 0 {
            Subvolume::try_from(id).into()
        } else {
            panic!("subvolume iterator returned both a null path")
        }
    }
}

impl Drop for SubvolumeIterator {
    fn drop(&mut self) {
        unsafe {
            btrfs_util_destroy_subvolume_iterator(self.0);
        }
    }
}

impl TryFrom<&Subvolume> for SubvolumeIterator {
    type Error = LibError;

    /// Same as SubvolumeIterator::new with no flags.
    #[inline]
    fn try_from(src: &Subvolume) -> Result<SubvolumeIterator> {
        SubvolumeIterator::new_impl(src.path(), None)
    }
}

impl TryInto<Vec<Subvolume>> for SubvolumeIterator {
    type Error = LibError;

    /// Same as SubvolumeIterator.`collect::<Result<Vec<Subvolume>>>`.
    #[inline]
    fn try_into(self) -> Result<Vec<Subvolume>> {
        self.collect::<Result<Vec<Subvolume>>>()
    }
}
