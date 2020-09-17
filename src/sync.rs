//! Module related to syncing a btrfs filesystem.

use crate::common;
use crate::Result;

use std::path::Path;

use btrfsutil_sys::btrfs_util_start_sync;
use btrfsutil_sys::btrfs_util_wait_sync;

/// Start syncing on a btrfs filesystem.
pub fn sync<'a, P>(path: P) -> Result<()>
where
    P: Into<&'a Path>,
{
    sync_impl(path.into())
}

fn sync_impl(path: &Path) -> Result<()> {
    let path_cstr = common::path_to_cstr(path);

    let async_transid: u64 = {
        let mut async_transid: u64 = 0;
        unsafe_wrapper!({ btrfs_util_start_sync(path_cstr.as_ptr(), &mut async_transid) })?;
        async_transid
    };

    unsafe_wrapper!({ btrfs_util_wait_sync(path_cstr.as_ptr(), async_transid) })?;

    Ok(())
}
