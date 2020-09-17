//! Btrfs quota groups

use crate::error::*;
use crate::Result;

use std::convert::TryFrom;

use btrfsutil_sys::btrfs_util_create_qgroup_inherit;
use btrfsutil_sys::btrfs_util_destroy_qgroup_inherit;
use btrfsutil_sys::btrfs_util_qgroup_inherit;
use btrfsutil_sys::btrfs_util_qgroup_inherit_add_group;
use btrfsutil_sys::btrfs_util_qgroup_inherit_get_groups;

/// Qgroup inheritance specifier.
///
/// Wrapper around [btrfs_util_qgroup_inherit].
///
/// [btrfs_util_qgroup_inherit]: ../bindings/struct.btrfs_util_qgroup_inherit.html
#[derive(Clone, Debug)]
pub struct QgroupInherit(*mut btrfs_util_qgroup_inherit);

impl QgroupInherit {
    /// Create a quota group inheritance specifier.
    pub fn create() -> Result<Self> {
        let mut qgroup_ptr: *mut btrfs_util_qgroup_inherit = std::ptr::null_mut();

        unsafe_wrapper!({ btrfs_util_create_qgroup_inherit(0, &mut qgroup_ptr) })?;

        glue_error!(qgroup_ptr.is_null(), GlueError::NullPointerReceived);

        Ok(Self(qgroup_ptr))
    }

    /// Add inheritance from a qgroup to a qgroup inheritance specifier.
    pub fn add(&mut self, qgroup_id: u64) -> Result<()> {
        let qgroup_ptr_initial: *mut btrfs_util_qgroup_inherit = self.as_ptr();
        let mut qgroup_ptr: *mut btrfs_util_qgroup_inherit = self.as_ptr();

        unsafe_wrapper!({ btrfs_util_qgroup_inherit_add_group(&mut qgroup_ptr, qgroup_id) })?;

        glue_error!(qgroup_ptr.is_null(), GlueError::NullPointerReceived);

        if qgroup_ptr != qgroup_ptr_initial {
            self.0 = qgroup_ptr;
        }

        Ok(())
    }

    /// Get the qgroup ids contained by this inheritance specifier.
    pub fn get_groups(&self) -> Result<Vec<u64>> {
        let qgroup_ptr: *const btrfs_util_qgroup_inherit = self.as_ptr();
        let mut qgroup_ids_ptr: *const u64 = std::ptr::null();
        let mut qgroup_ids_count: u64 = 0;

        unsafe {
            btrfs_util_qgroup_inherit_get_groups(
                qgroup_ptr,
                &mut qgroup_ids_ptr,
                &mut qgroup_ids_count,
            );
        }

        if qgroup_ids_count == 0 {
            return Ok(Vec::new());
        }

        glue_error!(qgroup_ids_ptr.is_null(), GlueError::NullPointerReceived);

        let ids: Vec<u64> = unsafe {
            std::slice::from_raw_parts(qgroup_ids_ptr, qgroup_ids_count as usize).to_owned()
        };
        Ok(ids)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut btrfs_util_qgroup_inherit {
        self.0
    }
}

impl Drop for QgroupInherit {
    fn drop(&mut self) {
        unsafe {
            btrfs_util_destroy_qgroup_inherit(self.0);
        }
    }
}
