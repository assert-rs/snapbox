//! Initialize working directories and assert on how they've changed

#[doc(inline)]
pub use crate::cargo_rustc_current_dir;
#[doc(inline)]
pub use crate::current_dir;
#[doc(inline)]
pub use crate::current_rs;

/// Working directory for tests
#[deprecated(since = "0.5.11", note = "Replaced with dir::DirRoot")]
pub type PathFixture = crate::dir::DirRoot;

pub use crate::dir::FileType;
pub use crate::dir::PathDiff;

/// Recursively walk a path
///
/// Note: Ignores `.keep` files
#[deprecated(since = "0.5.11", note = "Replaced with dir::Walk")]
#[cfg(feature = "dir")]
pub type Walk = crate::dir::Walk;

/// Copy a template into a [`PathFixture`]
///
/// Note: Generally you'll use [`PathFixture::with_template`] instead.
///
/// Note: Ignores `.keep` files
#[deprecated(since = "0.5.11", note = "Replaced with dir::copy_template")]
#[cfg(feature = "dir")]
pub fn copy_template(
    source: impl AsRef<std::path::Path>,
    dest: impl AsRef<std::path::Path>,
) -> crate::assert::Result<()> {
    crate::dir::copy_template(source, dest)
}

#[deprecated(since = "0.5.11", note = "Replaced with dir::resolve_dir")]
pub fn resolve_dir(
    path: impl AsRef<std::path::Path>,
) -> Result<std::path::PathBuf, std::io::Error> {
    crate::dir::resolve_dir(path)
}

#[deprecated(since = "0.5.11", note = "Replaced with dir::strip_trailing_slash")]
pub fn strip_trailing_slash(path: &std::path::Path) -> &std::path::Path {
    crate::dir::strip_trailing_slash(path)
}
