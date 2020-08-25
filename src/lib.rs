//! # btrfsutil-rs
//!
//! [![Build Status](https://travis-ci.com/cezarmathe/btrfsutil-rs.svg?branch=master)](https://travis-ci.com/cezarmathe/btrfsutil-rs)
//! [![btrfsutil](https://img.shields.io/crates/v/btrfsutil)](https://crates.io/crates/btrfsutil)
//! [![docs](https://docs.rs/btrfsutil/badge.svg)](https://docs.rs/btrfsutil)
//! [![libbtrfsutil version](https://img.shields.io/badge/libbtrfsutil-1.2.0-7979F1)](https://github.com/kdave/btrfs-progs/blob/471b4cf7e3a46222531a895f90228ea164b1b857/libbtrfsutil/btrfsutil.h#L28-L30)
//!
//! Safe wrappers for [libbtrfsutil](https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil).
//!
//! ## Building
//!
//! This library links to `libbtrfsutil`, a shared library provided by installing [btrfs-progs](https://github.com/kdave/btrfs-progs) on most Linux systems.
//!
//! - Arch Linux: `# pacman -S btrfs-progs`
//! - Ubuntu: `# apt install btrfs-progs`
//!
//! ## Usage
//!
//! Please keep in mind that many of the operations this library can perform may require elevated
//! privileges(CAP_SYSTEM_ADMIN).
//!
//! ## Examples
//!
//! How to run examples with elevated privileges:
//!
//! - build with: `cargo build --examples`
//! - execute with: `sudo target/debug/examples/example_name`.
//!
//! **[Subvolume iterator info](examples/subvolume_iterator_info.rs)**
//!
//! This example requires elevated privileges.
//!
//! ```Rust
//! // This will print out informations about all subvolumes under /
//!
//! // Retrieve the subvolume for /
//! let root_subvol = Subvolume::get("/").unwrap();
//!
//! // Retrieve a subvolume iterator for /
//! let subvol_iterator: SubvolumeIterator = {
//!     let result: Result<SubvolumeIterator> = root_subvol.into();
//!     result.unwrap()
//! };
//!
//! // Iterate over the subvolumes and print out their debug information
//! for subvolume in subvol_iterator {
//!     println!("{:?}", subvolume.info().unwrap());
//! }
//! ```

#![deny(missing_docs)]

pub mod bindings {
    //! Raw bindings to [libbtrfsutil](https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil).

    #![allow(missing_docs)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    /// Id of the root subvolume in a Btrfs filesystem.
    pub const BTRFS_FS_TREE_OBJECTID: u64 = 5;
}

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod error;
#[macro_use]
mod common;
pub mod qgroup;
pub mod subvolume;

#[cfg(test)]
mod testing;

pub use error::BtrfsUtilError;

/// Result type used by this library.
pub type Result<T> = std::result::Result<T, BtrfsUtilError>;
