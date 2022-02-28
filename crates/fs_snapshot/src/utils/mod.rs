mod diff;
mod lines;

#[cfg(feature = "diff")]
pub use diff::render_diff;
pub use lines::LinesWithTerminator;
