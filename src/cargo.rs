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

/// Prepare an example for testing
///
/// Unlike [`trycmd::cargo_bin!`], this does not inherit all of the current compiler settings.  It
/// will match the current target and profile but will not get feature flags.  Pass those arguments
/// to the compiler via `args`.
///
/// ## Example
///
/// ```rust,no_run
/// #[test]
/// fn cli_tests() {
///     trycmd::TestCases::new()
///         .register_bin("example-fixture", trycmd::cargo::compile_example("example-fixture", []))
///         .case("examples/cmd/*.trycmd");
/// }
/// ```
#[cfg(feature = "examples")]
pub fn compile_example<'a>(
    name: &str,
    args: impl IntoIterator<Item = &'a str>,
) -> Result<std::path::PathBuf, crate::Error> {
    eprintln!("Compiling example {}", name);
    let messages = escargot::CargoBuild::new()
        .current_target()
        .current_release()
        .example(name)
        .args(args)
        .exec()
        .map_err(|e| crate::Error::new(e.to_string()))?;
    for message in messages {
        let message = message.map_err(|e| crate::Error::new(e.to_string()))?;
        let message = message
            .decode()
            .map_err(|e| crate::Error::new(e.to_string()))?;
        eprintln!("Message: {:?}", message);
        match message {
            escargot::format::Message::CompilerMessage(msg) => {
                let level = msg.message.level;
                if level == escargot::format::diagnostic::DiagnosticLevel::Ice
                    || level == escargot::format::diagnostic::DiagnosticLevel::Error
                {
                    return Err(crate::Error::new(
                        msg.message
                            .rendered
                            .unwrap_or(msg.message.message)
                            .into_owned(),
                    ));
                }
            }
            escargot::format::Message::CompilerArtifact(artifact) => {
                if !artifact.profile.test
                    && artifact.target.crate_types == ["bin"]
                    && artifact.target.kind == ["example"]
                {
                    let path = artifact
                        .executable
                        .expect("cargo is new enough for this to be present");
                    return Ok(path.into_owned());
                }
            }
            _ => {}
        }
    }

    Err(crate::Error::new(format!(
        "Unknown error building example {}",
        name
    )))
}

/// Look up the path to a cargo-built binary within an integration test.
///
/// **NOTE:** Prefer `trycmd::cargo_bin!` as this makes assumptions about cargo
pub(crate) fn cargo_bin(name: &str) -> std::path::PathBuf {
    let file_name = format!("{}{}", name, std::env::consts::EXE_SUFFIX);
    let target_dir = target_dir();
    target_dir.join(&file_name)
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
