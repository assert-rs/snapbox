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

/// Look up the path to a cargo-built binary within an integration test.
///
/// **NOTE:** Prefer `trycmd::cargo_bin!` as this makes assumptions about cargo
pub(crate) fn cargo_bin<S: AsRef<str>>(name: S) -> std::path::PathBuf {
    cargo_bin_str(name.as_ref())
}

fn cargo_bin_str(name: &str) -> std::path::PathBuf {
    target_dir().join(format!("{}{}", name, std::env::consts::EXE_SUFFIX))
}

// Adapted from
// https://github.com/rust-lang/cargo/blob/485670b3983b52289a2f353d589c57fae2f60f82/tests/testsuite/support/mod.rs#L507
fn target_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap()
}
