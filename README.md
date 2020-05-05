# btrfsutil-rs

[![Build Status](https://travis-ci.com/cezarmathe/btrfsutil-rs.svg?branch=master)](https://travis-ci.com/cezarmathe/btrfsutil-rs)
[![btrfsutil](https://img.shields.io/crates/v/btrfsutil)](https://crates.io/crates/btrfsutil)
[![docs](https://docs.rs/btrfsutil/badge.svg)](https://docs.rs/btrfsutil)
[![libbtrfsutil version](https://img.shields.io/badge/libbtrfsutil-1.2.0-7979F1)](https://github.com/kdave/btrfs-progs/blob/471b4cf7e3a46222531a895f90228ea164b1b857/libbtrfsutil/btrfsutil.h#L28-L30)

Safe wrappers for [libbtrfsutil](https://github.com/kdave/btrfs-progs/tree/master/libbtrfsutil).

## Building

This library links to `libbtrfsutil`, a shared library provided by installing [btrfs-progs](https://github.com/kdave/btrfs-progs) on most Linux systems.

- Arch Linux: `# pacman -S btrfs-progs`
- Ubuntu: `# apt install btrfs-progs`

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
btrfsutil = "0.1.0"
```

For further details, please refer to the [documentation](https://docs.rs/btrfsutil).

Also, please keep in mind that many of the operations this library can perform may require elevated
privileges(CAP_SYSTEM_ADMIN).

## Examples

How to run examples with elevated privileges:

- build with: `cargo build --examples`
- execute with: `sudo target/debug/examples/example_name`.

**[Subvolume iterator info](examples/subvolume_iterator_info.rs)**

This example requires elevated privileges.

```Rust
// This will print out informations about all subvolumes under /

// Retrieve the subvolume for /
let root_subvol = Subvolume::get("/").unwrap();

// Retrieve a subvolume iterator for /
let subvol_iterator: SubvolumeIterator = {
    let result: Result<SubvolumeIterator> = root_subvol.into();
    result.unwrap()
};

// Iterate over the subvolumes and print out their debug information
for subvolume in subvol_iterator {
    println!("{:?}", subvolume.info().unwrap());
}
```

## License

MIT
