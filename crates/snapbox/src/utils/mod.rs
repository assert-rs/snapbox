mod lines;

pub use lines::LinesWithTerminator;

/// Normalize line endings
pub fn normalize_lines(data: &str) -> String {
    normalize_lines_chars(data.chars()).collect()
}

fn normalize_lines_chars(data: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    normalize_line_endings::normalized(data)
}

/// Normalize path separators
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
