# btrfsutil-rs

![Pre-release Checks](https://github.com/cezarmathe/btrfsutil-rs/actions/workflows/check.yml/badge.svg?branch=master)
[![btrfsutil](https://img.shields.io/crates/v/btrfsutil)](https://crates.io/crates/btrfsutil)
[![docs](https://docs.rs/btrfsutil/badge.svg)](https://docs.rs/btrfsutil)
[![libbtrfsutil version](https://img.shields.io/badge/libbtrfsutil-1.2.0-7979F1)](https://github.com/kdave/btrfs-progs/blob/471b4cf7e3a46222531a895f90228ea164b1b857/libbtrfsutil/btrfsutil.h#L28-L30)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Safe wrappers for [libbtrfsutil](https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil).

## Building

This library links to `libbtrfsutil`, a shared library provided by installing [btrfs-progs](https://github.com/kdave/btrfs-progs) on most Linux systems.

- Arch Linux: `pacman -S btrfs-progs`
- Ubuntu: `apt install btrfs-progs`

## Usage

Add the latest version to your project with:

```shell
cargo add btrfsutil
```

For further details, please refer to the [documentation](https://docs.rs/btrfsutil).

Also, please keep in mind that many of the operations this library can perform may require elevated
privileges(`CAP_SYSTEM_ADMIN`).

## Examples

Examples require elevated privileges. Environment variables can be used to run examples with `sudo`, like so:

```shell
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' cargo run --example subvolume_iterator_info
```
