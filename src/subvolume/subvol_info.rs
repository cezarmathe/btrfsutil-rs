use crate::bindings;
use crate::common;
use crate::error::GlueError;
use crate::error::LibError;
use crate::error::LibErrorCode;
use crate::subvolume::Subvolume;
use crate::BtrfsUtilError;
use crate::Result;

use std::convert::Into;
use std::convert::TryFrom;

use bindings::btrfs_util_subvolume_info;

use chrono::NaiveDateTime;
use chrono::Timelike;
use uuid::Uuid;

/// Information about a Btrfs subvolume.
///
/// Analogous to [btrfs_util_subvolume_info](../bindings/struct.btrfs_util_subvolume_info.html).
#[derive(Clone, Debug)]
pub struct SubvolumeInfo {
    /// ID of this subvolume, unique across the filesystem.
    pub id: u64,
    /// ID of the subvolume which contains this subvolume, or zero for the root subvolume
    /// ([BTRFS_FS_TREE_OBJECTID]) or orphaned subvolumes (i.e., subvolumes which have been
    /// deleted but not yet cleaned up).
    ///
    /// [BTRFS_FS_TREE_OBJECTID]: https://github.com/kdave/btrfs-progs/blob/471b4cf7e3a46222531a895f90228ea164b1b857/libbtrfsutil/btrfs_tree.h#L34
    pub parent_id: Option<u64>,
    /// Inode number of the directory containing this subvolume in the parent subvolume, or zero
    /// for the root subvolume ([BTRFS_FS_TREE_OBJECTID]) or orphaned subvolumes.
    ///
    /// [BTRFS_FS_TREE_OBJECTID]: https://github.com/kdave/btrfs-progs/blob/471b4cf7e3a46222531a895f90228ea164b1b857/libbtrfsutil/btrfs_tree.h#L34
    pub dir_id: Option<u64>,
    /// On-disk root item flags.
    pub flags: u64,
    /// UUID of this subvolume.
    pub uuid: Uuid,
    /// UUID of the subvolume this subvolume is a snapshot of, or all zeroes if this subvolume is
    /// not a snapshot.
    pub parent_uuid: Option<Uuid>,
    /// UUID of the subvolume this subvolume was received from, or all zeroes if this subvolume was
    /// not received. Note that this field, [stransid](#structfield.stransid),
    /// [rtransid](#structfield.rtransid), [stime](#structfield.stime), and
    /// [rtime](#structfield.rtime) are set manually by userspace after a subvolume is received.
    pub received_uuid: Option<Uuid>,
    /// Transaction ID of the subvolume root.
    pub generation: u64,
    /// Transaction ID when an inode in this subvolume was last changed.
    pub ctransid: u64,
    /// Transaction ID when this subvolume was created.
    pub otransid: u64,
    /// Transaction ID of the sent subvolume this subvolume was received from, or zero if this
    /// subvolume was not received. See the note on [received_uuid](#structfield.received_uuid).
    pub stransid: Option<u64>,
    /// Transaction ID when this subvolume was received, or zero if this subvolume was not
    /// received. See the note on [received_uuid](#structfield.received_uuid).
    pub rtransid: Option<u64>,
    /// Time when an inode in this subvolume was last changed.
    pub ctime: NaiveDateTime,
    /// Time when this subvolume was created.
    pub otime: NaiveDateTime,
    /// Not well-defined, usually zero unless it was set otherwise. See the note on
    /// [received_uuid](#structfield.received_uuid).
    pub stime: Option<NaiveDateTime>,
    /// Time when this subvolume was received, or zero if this subvolume was not received. See the
    /// [received_uuid](#structfield.received_uuid).
    pub rtime: Option<NaiveDateTime>,
}

impl TryFrom<&Subvolume> for SubvolumeInfo {
    type Error = BtrfsUtilError;

    fn try_from(src: &Subvolume) -> Result<Self> {
        let path_cstr = common::path_to_cstr(src.fs_root())?;
        let btrfs_subvolume_info_ptr: *mut btrfs_util_subvolume_info =
            Box::into_raw(Box::from(btrfs_util_subvolume_info {
                id: 0,
                parent_id: 0,
                dir_id: 0,
                flags: 0,
                uuid: [0; 16],
                parent_uuid: [0; 16],
                received_uuid: [0; 16],
                generation: 0,
                ctransid: 0,
                otransid: 0,
                stransid: 0,
                rtransid: 0,
                ctime: bindings::timespec {
                    tv_nsec: 0 as bindings::__time_t,
                    tv_sec: 0 as bindings::__syscall_slong_t,
                },
                otime: bindings::timespec {
                    tv_nsec: 0 as bindings::__time_t,
                    tv_sec: 0 as bindings::__syscall_slong_t,
                },
                stime: bindings::timespec {
                    tv_nsec: 0 as bindings::__time_t,
                    tv_sec: 0 as bindings::__syscall_slong_t,
                },
                rtime: bindings::timespec {
                    tv_nsec: 0 as bindings::__time_t,
                    tv_sec: 0 as bindings::__syscall_slong_t,
                },
            }));

        unsafe_wrapper!({
            btrfs_util_subvolume_info(path_cstr.as_ptr(), src.id(), btrfs_subvolume_info_ptr)
        })?;

        glue_error!(
            btrfs_subvolume_info_ptr.is_null(),
            GlueError::NullPointerReceived
        );

        SubvolumeInfo::try_from(unsafe { Box::from_raw(btrfs_subvolume_info_ptr) })
    }
}

macro_rules! handle_uuid {
    ($src: expr) => {
        match Uuid::from_slice($src) {
            Ok(val) => val,
            Err(e) => glue_error!(GlueError::UuidError(e)),
        }
    };
}

macro_rules! handle_timespec {
    ($src: expr) => {
        match NaiveDateTime::from_timestamp_opt($src.tv_sec, $src.tv_nsec as u32) {
            Some(val) => val,
            None => glue_error!(GlueError::BadTimespec(format!("{:?}", $src))),
        }
    };
}

impl TryFrom<Box<btrfs_util_subvolume_info>> for SubvolumeInfo {
    type Error = BtrfsUtilError;

    fn try_from(src: Box<btrfs_util_subvolume_info>) -> Result<Self> {
        let uuid: Uuid = handle_uuid!(&src.uuid);
        let parent_uuid_val: Uuid = handle_uuid!(&src.parent_uuid);
        let received_uuid_val: Uuid = handle_uuid!(&src.received_uuid);
        let ctime: NaiveDateTime = handle_timespec!(src.ctime);
        let otime: NaiveDateTime = handle_timespec!(src.otime);
        let stime_val: NaiveDateTime = handle_timespec!(src.stime);
        let rtime_val: NaiveDateTime = handle_timespec!(src.rtime);

        let parent_id: Option<u64> = if src.parent_id == 0 {
            None
        } else {
            Some(src.parent_id)
        };

        let dir_id: Option<u64> = if src.dir_id == 0 {
            None
        } else {
            Some(src.dir_id)
        };

        let parent_uuid: Option<Uuid> = if parent_uuid_val.is_nil() {
            None
        } else {
            Some(parent_uuid_val)
        };

        let received_uuid: Option<Uuid> = if received_uuid_val.is_nil() {
            None
        } else {
            Some(received_uuid_val)
        };

        let stransid: Option<u64> = if src.stransid == 0 {
            None
        } else {
            Some(src.stransid)
        };

        let rtransid: Option<u64> = if src.rtransid == 0 {
            None
        } else {
            Some(src.rtransid)
        };

        let stime: Option<NaiveDateTime> = if stime_val.nanosecond() == 0 && stime_val.second() == 0
        {
            None
        } else {
            Some(stime_val)
        };

        let rtime: Option<NaiveDateTime> = if rtime_val.nanosecond() == 0 && rtime_val.second() == 0
        {
            None
        } else {
            Some(rtime_val)
        };

        Ok(Self {
            id: src.id,
            parent_id,
            dir_id,
            flags: src.flags,
            uuid,
            parent_uuid,
            received_uuid,
            generation: src.generation,
            ctransid: src.ctransid,
            otransid: src.otransid,
            stransid,
            rtransid,
            ctime,
            otime,
            stime,
            rtime,
        })
    }
}
