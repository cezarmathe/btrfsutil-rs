#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]
#![deny(missing_docs, unreachable_pub)]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod error;
#[macro_use]
mod common;
pub mod qgroup;
pub mod subvolume;
pub mod sync;

#[cfg(test)]
mod testing;

pub use error::BtrfsUtilError;

/// Result type used by this library.
pub type Result<T> = std::result::Result<T, BtrfsUtilError>;
