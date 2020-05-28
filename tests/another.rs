mod common;

use common::*;

#[test]
fn test_common() {
    check_env();

    assert_eq!(0, btrfsutil::lol())
}
