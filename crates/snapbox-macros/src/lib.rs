#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[cfg(feature = "color")]
pub use anstream::eprint;
#[cfg(feature = "color")]
pub use anstream::eprintln;
#[cfg(not(feature = "color"))]
pub use std::eprint;
#[cfg(not(feature = "color"))]
pub use std::eprintln;

/// Feature-flag controlled additional test debug information
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        #![allow(unexpected_cfgs)]  // HACK: until we upgrade the minimum anstream
        $crate::eprint!("[{:>w$}] \t", module_path!(), w = 28);
        $crate::eprintln!($($arg)*);
    })
}

/// Feature-flag controlled additional test debug information
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
