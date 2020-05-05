use btrfsutil::subvolume::*;
use btrfsutil::Result;

fn main() {
    let root_subvol = Subvolume::get("/").unwrap();

    let subvol_iterator: SubvolumeIterator = {
        let result: Result<SubvolumeIterator> = root_subvol.into();
        result.unwrap()
    };

    for subvolume in subvol_iterator {
        println!("{:?}", subvolume.info().unwrap());
    }
}
