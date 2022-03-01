mod color;
mod diff;

pub use color::Palette;
pub use color::Styled;
#[cfg(feature = "diff")]
pub use diff::render_diff;
