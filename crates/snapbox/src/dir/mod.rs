//! Initialize working directories and assert on how they've changed

mod diff;
mod fixture;
mod ops;
#[cfg(test)]
mod tests;

pub use diff::FileType;
pub use diff::PathDiff;
pub use fixture::PathFixture;
#[cfg(feature = "dir")]
pub use ops::copy_template;
pub use ops::resolve_dir;
pub use ops::strip_trailing_slash;
#[cfg(feature = "dir")]
pub use ops::Walk;

#[cfg(feature = "dir")]
pub(crate) use ops::canonicalize;
pub(crate) use ops::display_relpath;
pub(crate) use ops::shallow_copy;
