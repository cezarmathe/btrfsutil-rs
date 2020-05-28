pub fn check_env() {
    let err_msg = || {
        println!("***WARNING***");
        println!("Testing `btrfsutil` can result in permanent system damage.");
        println!("Only test `btrfsutil` in disposable environments.");
        panic!("TESTING IS NOT ALLOWED");
    };
    if std::env::var("BTRFSUTIL_TESTENV").is_err() {
        err_msg();
    } else if std::env::var("BTRFSUTIL_TESTENV").unwrap_or_default().is_empty() {
        err_msg();
    }
}

pub fn get_fs_root() -> String {
    format!("/mnt/{}", std::env::var("BTRFSUTIL_TESTENV").unwrap())
}
