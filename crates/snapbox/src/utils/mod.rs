mod lines;

pub use lines::LinesWithTerminator;

/// Normalize line endings
pub fn normalize_lines(data: &str) -> String {
    normalize_line_endings::normalized(data.chars()).collect()
}

/// "Smart" text normalization
///
/// This includes
/// - Line endings
/// - Path separators
pub fn normalize_text(data: &str) -> String {
    normalize_line_endings::normalized(data.chars())
        // Also help out with Windows paths
        .map(|c| if c == '\\' { '/' } else { c })
        .collect()
}
