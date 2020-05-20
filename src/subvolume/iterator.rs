use crate::common;
use crate::error::GlueError;
use crate::error::LibError;
use crate::error::LibErrorCode;
use crate::subvolume::Subvolume;
use crate::Result;

use std::convert::TryFrom;

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

/// Wrapper around the raw subvolume iterator
struct RawIterator(*mut btrfs_util_subvolume_iterator);

impl RawIterator {
    fn next(&self) -> Result<Subvolume> {
        let mut str_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
        let mut id: u64 = 0;

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_subvolume_iterator_next(self.0, &mut str_ptr, &mut id);
        });

        glue_error!(str_ptr.is_null(), GlueError::NullPointerReceived);
        glue_error!(
            id < btrfsutil_sys::BTRFS_FS_TREE_OBJECTID,
            GlueError::BadId(id)
        );

        Ok(Subvolume::new(id))
    }
}

impl Drop for RawIterator {
    fn drop(&mut self) {
        unsafe {
            btrfs_util_destroy_subvolume_iterator(self.0);
        }
    }
}

/// A Subvolume iterator.
pub struct SubvolumeIterator(Vec<Subvolume>);

impl SubvolumeIterator {
    /// Create a new subvolume iterator.
    #[allow(clippy::identity_conversion)]
    pub fn create(subvolume: Subvolume, flags: Option<SubvolumeIteratorFlags>) -> Result<Self> {
        let path_cstr = common::path_to_cstr(subvolume.path()?)?;
        let flags_val = if let Some(val) = flags { val.bits() } else { 0 };
        let mut iterator_ptr: *mut btrfs_util_subvolume_iterator = std::ptr::null_mut();

        unsafe_wrapper!(errcode, {
            errcode = btrfs_util_create_subvolume_iterator(
                path_cstr.as_ptr(),
                subvolume.id(),
                flags_val,
                &mut iterator_ptr,
            );
        });

        glue_error!(iterator_ptr.is_null(), GlueError::NullPointerReceived);

        let items: Vec<Subvolume> = {
            let mut items = Vec::new();
            let raw_iterator = RawIterator(iterator_ptr);
            loop {
                match raw_iterator.next() {
                    Ok(val) => items.push(val),
                    Err(e) => {
                        if e == LibError::StopIteration.into() {
                            break;
                        } else {
                            return Result::Err(e);
                        }
                    }
                }
            }
            items
        };

        Ok(Self(items))
    }
}

impl IntoIterator for SubvolumeIterator {
    type Item = Subvolume;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
