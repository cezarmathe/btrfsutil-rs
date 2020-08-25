// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    fs::File,
    io,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use nix::mount::{umount2, MntFlags};

/// Execute command while collecting stdout & stderr.
fn execute_cmd(cmd: &mut Command) -> io::Result<()> {
    match cmd.output() {
        Err(err) => {
            eprintln!("cmd: {:?}, error '{}'", cmd, err.to_string());
            Err(err)
        }

        Ok(result) => {
            if result.status.success() {
                Ok(())
            } else {
                let std_out_txt = String::from_utf8_lossy(&result.stdout);
                let std_err_txt = String::from_utf8_lossy(&result.stderr);
                eprintln!(
                    "cmd: {:?} stdout: {} stderr: {}",
                    cmd, std_out_txt, std_err_txt
                );
                Ok(())
            }
        }
    }
}

/// Generate an XFS FS, does not specify UUID as that's not supported on version in Travis
pub fn btrfs_create_fs(devnode: &Path) -> io::Result<()> {
    execute_cmd(Command::new("mkfs.btrfs").arg("-f").arg("-q").arg(&devnode))
}

/// Unmount any filesystems that contain TEST_ID in the mount point.
/// Return immediately on the first unmount failure.
fn test_fs_unmount() -> io::Result<()> {
    || -> io::Result<()> {
        let mut mount_data = String::new();
        File::open("/proc/self/mountinfo")?.read_to_string(&mut mount_data)?;
        let parser = libmount::mountinfo::Parser::new(mount_data.as_bytes());

        for mount_point in parser
            .filter_map(|x| x.ok())
            .filter_map(|m| m.mount_point.into_owned().into_string().ok())
            .filter(|mp| mp.contains("/tmp/btrfsutil/"))
        {
            umount2(&PathBuf::from(mount_point), MntFlags::MNT_DETACH).map_err(|e| {
                eprintln!("Could not umount2: {}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;
        }
        Ok(())
    }()
}

pub fn clean_up() -> io::Result<()> {
    test_fs_unmount()
}
