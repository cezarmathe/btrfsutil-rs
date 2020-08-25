use btrfsutil::subvolume::*;
use btrfsutil::Result;

use std::path::Path;

fn main() {
    let root_subvol = Subvolume::from_path(Path::new("/")).unwrap();

    let subvol_iterator: SubvolumeIterator = {
        let result: Result<SubvolumeIterator> = root_subvol.into();
        result.unwrap()
    };

    for subvolume in subvol_iterator {
        println!("{:?}", subvolume.info().unwrap());
    }
}
