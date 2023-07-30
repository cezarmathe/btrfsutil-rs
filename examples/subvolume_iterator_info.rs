use btrfsutil::subvolume::*;

use std::path::Path;

fn main() {
    let root_path = std::env::var("SUBVOLUME_PATH").unwrap_or_else(|_| "/mnt/btrfs".to_owned());
    let root_subvol = Subvolume::try_from(Path::new(&root_path)).unwrap();

    let subvol_iterator = SubvolumeIterator::try_from(&root_subvol).unwrap();

    for subvolume in subvol_iterator {
        println!("{:?}", subvolume.unwrap().info().unwrap());
    }
}
