//! Utilities to report test results to users

mod color;
mod diff;

pub use color::Palette;
pub(crate) use color::Style;
pub use color::Styled;
pub use diff::write_diff;
