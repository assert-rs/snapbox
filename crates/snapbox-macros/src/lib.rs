#![cfg_attr(docsrs, feature(doc_auto_cfg))]
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

/// The absolute path to a binary target's executable.
///
/// The `bin_target_name` is the name of the binary
/// target, exactly as-is.
///
/// **NOTE:** This is only set when building an integration test or benchmark.
///
/// ## Example
///
/// ```rust,no_run
/// #[test]
/// fn cli_tests() {
///     trycmd::TestCases::new()
///         .default_bin_path(trycmd::cargo_bin!("bin-fixture"))
///         .case("tests/cmd/*.trycmd");
/// }
/// ```
#[macro_export]
macro_rules! cargo_bin {
    ($bin_target_name:expr) => {
        ::std::path::Path::new(env!(concat!("CARGO_BIN_EXE_", $bin_target_name)))
    };
}
