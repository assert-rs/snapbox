mod lines;

pub use lines::LinesWithTerminator;

#[doc(inline)]
pub use crate::cargo_rustc_current_dir;
#[doc(inline)]
pub use crate::current_dir;
#[doc(inline)]
pub use crate::current_rs;

#[deprecated(since = "0.5.11", note = "Replaced with `filter::normalize_lines`")]
pub fn normalize_lines(data: &str) -> String {
    crate::filter::normalize_lines(data)
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::normalize_paths`")]
pub fn normalize_paths(data: &str) -> String {
    crate::filter::normalize_paths(data)
}

/// "Smart" text normalization
///
/// This includes
/// - Line endings
/// - Path separators
#[deprecated(
    since = "0.5.11",
    note = "Replaced with `filter::normalize_paths(filter::normalize_lines(...))`"
)]
pub fn normalize_text(data: &str) -> String {
    #[allow(deprecated)]
    normalize_paths(&normalize_lines(data))
}
