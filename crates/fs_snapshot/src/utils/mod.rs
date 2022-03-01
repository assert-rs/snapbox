mod lines;

pub use lines::LinesWithTerminator;

pub fn normalize_text(data: &str) -> String {
    normalize_line_endings::normalized(data.chars())
        // Also help out with Windows paths
        .map(|c| if c == '\\' { '/' } else { c })
        .collect()
}
