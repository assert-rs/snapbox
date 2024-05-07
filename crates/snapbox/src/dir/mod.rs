//! Initialize working directories and assert on how they've changed

mod diff;
#[cfg(feature = "dir")]
#[allow(clippy::module_inception)]
mod dir;
mod ops;
mod root;
mod source;
#[cfg(test)]
mod tests;

pub use diff::FileType;
pub use diff::PathDiff;
#[cfg(feature = "dir")]
pub use dir::Dir;
#[cfg(feature = "dir")]
pub use dir::DirEntry;
#[cfg(feature = "dir")]
pub use dir::InMemoryDir;
#[cfg(feature = "dir")]
pub use dir::InMemoryDirIter;
#[cfg(feature = "dir")]
pub use dir::PathIter;
pub use ops::resolve_dir;
pub use ops::strip_trailing_slash;
pub use root::DirRoot;
pub use source::DirSource;

#[cfg(feature = "dir")]
pub(crate) use ops::canonicalize;
pub(crate) use ops::shallow_copy;
#[cfg(feature = "dir")]
pub(crate) use ops::Walk;
