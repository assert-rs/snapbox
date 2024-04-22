//! # Snapshot testing toolbox
//!
//! > When you have to treat your tests like pets, instead of [cattle][trycmd]
//!
//! `snapbox` is a snapshot-testing toolbox that is ready to use for verifying output from
//! - Function return values
//! - CLI stdout/stderr
//! - Filesystem changes
//!
//! It is also flexible enough to build your own test harness like [trycmd](https://crates.io/crates/trycmd).
//!
//! ## Which tool is right
//!
//! - [cram](https://bitheap.org/cram/): End-to-end CLI snapshotting agnostic of any programming language
//! - [trycmd](https://crates.io/crates/trycmd): For running a lot of blunt tests (limited test predicates)
//!   - Particular attention is given to allow the test data to be pulled into documentation, like
//!     with [mdbook](https://rust-lang.github.io/mdBook/)
//! - `snapbox`: When you want something like `trycmd` in one off
//!   cases or you need to customize `trycmd`s behavior.
//! - [assert_cmd](https://crates.io/crates/assert_cmd) +
//!   [assert_fs](https://crates.io/crates/assert_fs): Test cases follow a certain pattern but
//!   special attention is needed in how to verify the results.
//! - Hand-written test cases: for peculiar circumstances
//!
//! ## Getting Started
//!
//! Testing Functions:
//! - [`assert_eq`][crate::assert_eq()] and [`assert_matches`] for reusing diffing / pattern matching for non-snapshot testing
//! - [`harness::Harness`] for discovering test inputs and asserting against snapshot files:
//!
//! Testing Commands:
//! - [`cmd::Command`]: Process spawning for testing of non-interactive commands
//! - [`cmd::OutputAssert`]: Assert the state of a [`Command`][cmd::Command]'s
//!   [`Output`][std::process::Output].
//!
//! Testing Filesystem Interactions:
//! - [`path::PathFixture`]: Working directory for tests
//! - [`Assert`]: Diff a directory against files present in a pattern directory
//!
//! You can also build your own version of these with the lower-level building blocks these are
//! made of.
//!
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!
//! # Examples
//!
//! [`assert_matches`]
//! ```rust
//! snapbox::assert_matches("Hello [..] people!", "Hello many people!");
//! ```
//!
//! [`Assert`]
//! ```rust,no_run
//! let actual = "...";
//! snapbox::Assert::new()
//!     .action_env("SNAPSHOTS")
//!     .matches(snapbox::file!["help_output_is_clean.txt"], actual);
//! ```
//!
//! [`harness::Harness`]
#![cfg_attr(not(feature = "harness"), doc = " ```rust,ignore")]
#![cfg_attr(feature = "harness", doc = " ```rust,no_run")]
//! snapbox::harness::Harness::new(
//!     "tests/fixtures/invalid",
//!     setup,
//!     test,
//! )
//! .select(["tests/cases/*.in"])
//! .action_env("SNAPSHOTS")
//! .test();
//!
//! fn setup(input_path: std::path::PathBuf) -> snapbox::harness::Case {
//!     let name = input_path.file_name().unwrap().to_str().unwrap().to_owned();
//!     let expected = input_path.with_extension("out");
//!     snapbox::harness::Case {
//!         name,
//!         fixture: input_path,
//!         expected,
//!     }
//! }
//!
//! fn test(input_path: &std::path::Path) -> Result<usize, Box<dyn std::error::Error>> {
//!     let raw = std::fs::read_to_string(input_path)?;
//!     let num = raw.parse::<usize>()?;
//!
//!     let actual = num + 10;
//!
//!     Ok(actual)
//! }
//! ```
//!
//! [trycmd]: https://docs.rs/trycmd

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod macros;

pub mod assert;
pub mod cmd;
pub mod data;
pub mod dir;
pub mod filter;
pub mod path;
pub mod report;
pub mod utils;

#[cfg(feature = "harness")]
pub mod harness;

#[deprecated(since = "0.5.11", note = "Replaced with `assert::Assert`")]
pub use assert::Action;
pub use assert::Assert;
pub use data::Data;
pub use data::ToDebug;
pub use filter::RedactedValue;
pub use filter::Redactions;
#[doc(hidden)]
pub use snapbox_macros::debug;

#[deprecated(since = "0.5.11", note = "Replaced with `Redactions`")]
pub type Substitutions = filter::Redactions;

#[deprecated(since = "0.5.11", note = "Replaced with `assert::DEFAULT_ACTION_ENV`")]
pub const DEFAULT_ACTION_ENV: &str = assert::DEFAULT_ACTION_ENV;

#[deprecated(since = "0.5.11", note = "Replaced with `assert::Result`")]
pub type Result<T, E = assert::Error> = std::result::Result<T, E>;
#[deprecated(since = "0.5.11", note = "Replaced with `assert::Error`")]
pub type Error = assert::Error;

/// Easier access to common traits
pub mod prelude {
    pub use crate::ToDebug;
}

/// Check if a value is the same as an expected value
///
/// When the content is text, newlines are normalized.
///
/// ```rust
/// # use snapbox::assert_eq;
/// let output = "something";
/// let expected = "something";
/// assert_eq(expected, output);
/// ```
///
/// Can combine this with [`file!`]
/// ```rust,no_run
/// # use snapbox::assert_eq;
/// # use snapbox::file;
/// let actual = "something";
/// assert_eq(file!["output.txt"], actual);
/// ```
#[track_caller]
pub fn assert_eq(expected: impl Into<crate::Data>, actual: impl Into<crate::Data>) {
    Assert::new()
        .action_env(assert::DEFAULT_ACTION_ENV)
        .eq_(actual, expected.into().raw());
}

/// Check if a value matches a pattern
///
/// Pattern syntax:
/// - `...` is a line-wildcard when on a line by itself
/// - `[..]` is a character-wildcard when inside a line
/// - `[EXE]` matches `.exe` on Windows
///
/// Normalization:
/// - Newlines
/// - `\` to `/`
///
/// ```rust
/// # use snapbox::assert_matches;
/// let output = "something";
/// let expected = "so[..]g";
/// assert_matches(expected, output);
/// ```
///
/// Can combine this with [`file!`]
/// ```rust,no_run
/// # use snapbox::assert_matches;
/// # use snapbox::file;
/// let actual = "something";
/// assert_matches(file!["output.txt"], actual);
/// ```
#[track_caller]
pub fn assert_matches(pattern: impl Into<crate::Data>, actual: impl Into<crate::Data>) {
    Assert::new()
        .action_env(assert::DEFAULT_ACTION_ENV)
        .eq_(actual, pattern);
}

/// Check if a path matches the content of another path, recursively
///
/// When the content is text, newlines are normalized.
///
/// ```rust,no_run
/// let output_root = "...";
/// let expected_root = "tests/snapshots/output.txt";
/// snapbox::assert_subset_eq(expected_root, output_root);
/// ```
#[cfg(feature = "dir")]
#[track_caller]
pub fn assert_subset_eq(
    expected_root: impl Into<std::path::PathBuf>,
    actual_root: impl Into<std::path::PathBuf>,
) {
    Assert::new()
        .action_env(assert::DEFAULT_ACTION_ENV)
        .subset_eq(expected_root, actual_root);
}

/// Check if a path matches the pattern of another path, recursively
///
/// Pattern syntax:
/// - `...` is a line-wildcard when on a line by itself
/// - `[..]` is a character-wildcard when inside a line
/// - `[EXE]` matches `.exe` on Windows
///
/// Normalization:
/// - Newlines
/// - `\` to `/`
///
/// ```rust,no_run
/// let output_root = "...";
/// let expected_root = "tests/snapshots/output.txt";
/// snapbox::assert_subset_matches(expected_root, output_root);
/// ```
#[cfg(feature = "dir")]
#[track_caller]
pub fn assert_subset_matches(
    pattern_root: impl Into<std::path::PathBuf>,
    actual_root: impl Into<std::path::PathBuf>,
) {
    Assert::new()
        .action_env(assert::DEFAULT_ACTION_ENV)
        .subset_matches(pattern_root, actual_root);
}
