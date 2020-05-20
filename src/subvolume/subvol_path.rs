use std::path::PathBuf;

#[cfg(any(feature = "subvol-path-relaxed", feature = "subvol-path-strict"))]
type SubvolumePath = PathBuf;

#[cfg(any(
    feature = "subvol-path-no-confirm",
    feature = "subvol-path-try-confirm"
))]
#[derive(Clone, Debug, Eq, PartialEq)]
/// Subvolume path that tells the difference between a confirmed and an unconfirmed path.
pub enum SubvolumePath {
    /// An unconfirmed path.
    NotConfirmed(PathBuf),
    /// An confirmed path.
    Confirmed(PathBuf),
}
