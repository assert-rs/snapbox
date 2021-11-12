//! Interact with `cargo`

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

#[cfg(feature = "examples")]
pub use examples::{compile_example, compile_examples};

#[cfg(feature = "examples")]
pub(crate) mod examples {
    /// Prepare an example for testing
    ///
    /// Unlike [`cargo_bin!`][crate::cargo_bin!], this does not inherit all of the current compiler settings.  It
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
    pub fn compile_example<'a>(
        target_name: &str,
        args: impl IntoIterator<Item = &'a str>,
    ) -> crate::schema::Bin {
        compile_example_path(target_name, args).into()
    }

    fn compile_example_path<'a>(
        target_name: &str,
        args: impl IntoIterator<Item = &'a str>,
    ) -> Result<crate::schema::Bin, crate::Error> {
        debug!("Compiling example {}", target_name);
        let messages = escargot::CargoBuild::new()
            .current_target()
            .current_release()
            .example(target_name)
            .args(args)
            .exec()
            .map_err(|e| crate::Error::new(e.to_string()))?;
        for message in messages {
            let message = message.map_err(|e| crate::Error::new(e.to_string()))?;
            let message = message
                .decode()
                .map_err(|e| crate::Error::new(e.to_string()))?;
            debug!("Message: {:?}", message);
            if let Some(bin) = decode_example_message(&message) {
                let (name, bin) = bin?;
                assert_eq!(target_name, name);
                return Ok(bin);
            }
        }

        Err(crate::Error::new(format!(
            "Unknown error building example {}",
            target_name
        )))
    }

    /// Prepare all examples for testing
    ///
    /// Unlike [`cargo_bin!`][crate::cargo_bin!], this does not inherit all of the current compiler settings.  It
    /// will match the current target and profile but will not get feature flags.  Pass those arguments
    /// to the compiler via `args`.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// #[test]
    /// fn cli_tests() {
    ///     trycmd::TestCases::new()
    ///         .register_bins(trycmd::cargo::compile_examples([]).unwrap())
    ///         .case("examples/cmd/*.trycmd");
    /// }
    /// ```
    pub fn compile_examples<'a>(
        args: impl IntoIterator<Item = &'a str>,
    ) -> Result<impl Iterator<Item = (String, crate::schema::Bin)>, crate::Error> {
        debug!("Compiling examples");
        let mut examples = std::collections::BTreeMap::new();

        let messages = escargot::CargoBuild::new()
            .current_target()
            .current_release()
            .examples()
            .args(args)
            .exec()
            .map_err(|e| crate::Error::new(e.to_string()))?;
        for message in messages {
            let message = message.map_err(|e| crate::Error::new(e.to_string()))?;
            let message = message
                .decode()
                .map_err(|e| crate::Error::new(e.to_string()))?;
            debug!("Message: {:?}", message);
            if let Some(bin) = decode_example_message(&message) {
                let (name, bin) = bin?;
                examples.insert(name.to_owned(), bin);
            }
        }

        Ok(examples.into_iter())
    }

    fn decode_example_message<'m>(
        message: &'m escargot::format::Message,
    ) -> Option<Result<(&'m str, crate::schema::Bin), crate::Error>> {
        match message {
            escargot::format::Message::CompilerMessage(msg) => {
                let level = msg.message.level;
                if level == escargot::format::diagnostic::DiagnosticLevel::Ice
                    || level == escargot::format::diagnostic::DiagnosticLevel::Error
                {
                    let output = msg
                        .message
                        .rendered
                        .as_deref()
                        .unwrap_or_else(|| msg.message.message.as_ref())
                        .to_owned();
                    if is_example_target(&msg.target) {
                        let bin = crate::schema::Bin::Error(crate::Error::new(output));
                        Some(Ok((msg.target.name.as_ref(), bin)))
                    } else {
                        Some(Err(crate::Error::new(output)))
                    }
                } else {
                    None
                }
            }
            escargot::format::Message::CompilerArtifact(artifact) => {
                if !artifact.profile.test && is_example_target(&artifact.target) {
                    let path = artifact
                        .executable
                        .clone()
                        .expect("cargo is new enough for this to be present");
                    let bin = crate::schema::Bin::Path(path.into_owned());
                    Some(Ok((artifact.target.name.as_ref(), bin)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_example_target(target: &escargot::format::Target) -> bool {
        target.crate_types == ["bin"] && target.kind == ["example"]
    }
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
