mod color;
mod diff;

pub use color::Palette;
#[cfg(feature = "diff")]
pub use diff::render_diff;
