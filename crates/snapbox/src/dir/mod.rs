//! Initialize working directories and assert on how they've changed

mod diff;
mod ops;
mod root;
#[cfg(test)]
mod tests;

pub use diff::FileType;
pub use diff::PathDiff;
#[cfg(feature = "dir")]
pub use ops::copy_template;
pub use ops::resolve_dir;
pub use ops::strip_trailing_slash;
#[cfg(feature = "dir")]
pub use ops::Walk;
pub use root::PathFixture;

#[cfg(feature = "dir")]
pub(crate) use ops::canonicalize;
pub(crate) use ops::shallow_copy;
