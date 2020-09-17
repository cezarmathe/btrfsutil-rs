use crate::common;
use crate::error::LibError;
use crate::error::LibErrorCode;
use crate::subvolume::Subvolume;
use crate::BtrfsUtilError;
use crate::Result;

use std::convert::TryFrom;
use std::path::PathBuf;

use btrfsutil_sys::btrfs_util_subvolume_info;

use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use chrono::Timelike;

use uuid::Uuid;

/// Information about a Btrfs subvolume.
///
/// Contains everything from [btrfs_util_subvolume_info] plus the path of the subvolume.
///
/// [btrfs_util_subvolume_info]: https://docs.rs/btrfsutil-sys/1.2.1/btrfsutil_sys/struct.btrfs_util_subvolume_info.html
#[derive(Clone, Debug, PartialEq)]
pub struct SubvolumeInfo {
    /// ID of this subvolume, unique across the filesystem.
    pub id: u64,
    /// The path of the subvolume.
    pub path: PathBuf,
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
    pub ctime: DateTime<Local>,
    /// Time when this subvolume was created.
    pub otime: DateTime<Local>,
    /// Not well-defined, usually zero unless it was set otherwise. See the note on
    /// [received_uuid](#structfield.received_uuid).
    pub stime: Option<DateTime<Local>>,
    /// Time when this subvolume was received, or zero if this subvolume was not received. See the
    /// [received_uuid](#structfield.received_uuid).
    pub rtime: Option<DateTime<Local>>,
}

impl Into<Subvolume> for &SubvolumeInfo {
    fn into(self) -> Subvolume {
        Subvolume::new(self.id, self.path.clone())
    }
}

impl TryFrom<&Subvolume> for SubvolumeInfo {
    type Error = BtrfsUtilError;

    fn try_from(src: &Subvolume) -> Result<Self> {
        let path_cstr = common::path_to_cstr(src.path());
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
                ctime: btrfsutil_sys::timespec {
                    tv_nsec: 0 as btrfsutil_sys::__time_t,
                    tv_sec: 0 as btrfsutil_sys::__syscall_slong_t,
                },
                otime: btrfsutil_sys::timespec {
                    tv_nsec: 0 as btrfsutil_sys::__time_t,
                    tv_sec: 0 as btrfsutil_sys::__syscall_slong_t,
                },
                stime: btrfsutil_sys::timespec {
                    tv_nsec: 0 as btrfsutil_sys::__time_t,
                    tv_sec: 0 as btrfsutil_sys::__syscall_slong_t,
                },
                rtime: btrfsutil_sys::timespec {
                    tv_nsec: 0 as btrfsutil_sys::__time_t,
                    tv_sec: 0 as btrfsutil_sys::__syscall_slong_t,
                },
            }));

        unsafe_wrapper!({
            btrfs_util_subvolume_info(path_cstr.as_ptr(), src.id(), btrfs_subvolume_info_ptr)
        })?;

        let info: Box<btrfs_util_subvolume_info> =
            unsafe { Box::from_raw(btrfs_subvolume_info_ptr) };

        // process the retrieved info struct
        let uuid: Uuid = Uuid::from_slice(&info.uuid).expect("Failed to get uuid from C");
        let parent_uuid_val: Uuid =
            Uuid::from_slice(&info.parent_uuid).expect("Failed to get parent uuid from C");
        let received_uuid_val: Uuid =
            Uuid::from_slice(&info.received_uuid).expect("Failed to get received uuid from C");
        let ctime: DateTime<Local> = Local.timestamp(info.ctime.tv_sec, info.ctime.tv_nsec as u32);
        let otime: DateTime<Local> = Local.timestamp(info.otime.tv_sec, info.otime.tv_nsec as u32);
        let stime_val: DateTime<Local> =
            Local.timestamp(info.stime.tv_sec, info.stime.tv_nsec as u32);
        let rtime_val: DateTime<Local> =
            Local.timestamp(info.rtime.tv_sec, info.rtime.tv_nsec as u32);
        let parent_id: Option<u64> = if info.parent_id == 0 {
            None
        } else {
            Some(info.parent_id)
        };
        let dir_id: Option<u64> = if info.dir_id == 0 {
            None
        } else {
            Some(info.dir_id)
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
        let stransid: Option<u64> = if info.stransid == 0 {
            None
        } else {
            Some(info.stransid)
        };
        let rtransid: Option<u64> = if info.rtransid == 0 {
            None
        } else {
            Some(info.rtransid)
        };
        let stime: Option<DateTime<Local>> =
            if stime_val.nanosecond() == 0 && stime_val.second() == 0 {
                None
            } else {
                Some(stime_val)
            };
        let rtime: Option<DateTime<Local>> =
            if rtime_val.nanosecond() == 0 && rtime_val.second() == 0 {
                None
            } else {
                Some(rtime_val)
            };

        Ok(Self {
            id: info.id,
            path: src.path().to_path_buf(),
            parent_id,
            dir_id,
            flags: info.flags,
            uuid,
            parent_uuid,
            received_uuid,
            generation: info.generation,
            ctransid: info.ctransid,
            otransid: info.otransid,
            stransid,
            rtransid,
            ctime,
            otime,
            stime,
            rtime,
        })
    }
}
