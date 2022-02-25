mod color;
#[cfg(feature = "diff")]
mod diff;
mod harness;
#[cfg(feature = "diff")]
mod lines;

pub use harness::*;
