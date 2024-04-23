//! Interact with `cargo`

#[doc(inline)]
pub use snapbox::cmd::cargo_bin;

/// Prepare an example for testing
///
/// Unlike `cargo_bin!`, this does not inherit all of the current compiler settings.  It
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
    target_name: &str,
    args: impl IntoIterator<Item = &'a str>,
) -> crate::schema::Bin {
    snapbox::cmd::compile_example(target_name, args).into()
}

/// Prepare all examples for testing
///
/// Unlike `cargo_bin!`, this does not inherit all of the current compiler settings.  It
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
#[cfg(feature = "examples")]
pub fn compile_examples<'a>(
    args: impl IntoIterator<Item = &'a str>,
) -> Result<impl Iterator<Item = (String, crate::schema::Bin)>, crate::Error> {
    snapbox::cmd::compile_examples(args).map(|i| i.map(|(name, path)| (name, path.into())))
}
