//! Normalize `actual` or `expected` data
//!
//! This can be done for
//! - Making snapshots consistent across platforms or conditional compilation
//! - Focusing snapshots on the characteristics of the data being tested

mod redactions;

pub use redactions::RedactedValue;
pub use redactions::Redactions;
#[cfg(feature = "regex")]
pub use regex::Regex;

/// Normalize line endings
pub fn normalize_lines(data: &str) -> String {
    normalize_lines_chars(data.chars()).collect()
}

fn normalize_lines_chars(data: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    normalize_line_endings::normalized(data)
}

/// Normalize path separators
///
/// [`std::path::MAIN_SEPARATOR`] can vary by platform, so make it consistent
///
/// Note: this cannot distinguish between when a character is being used as a path separator or not
/// and can "normalize" unrelated data
pub fn normalize_paths(data: &str) -> String {
    normalize_paths_chars(data.chars()).collect()
}

fn normalize_paths_chars(data: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    data.map(|c| if c == '\\' { '/' } else { c })
}

/// "Smart" text normalization
///
/// This includes
/// - Line endings
/// - Path separators
pub fn normalize_text(data: &str) -> String {
    normalize_paths_chars(normalize_lines_chars(data.chars())).collect()
}
