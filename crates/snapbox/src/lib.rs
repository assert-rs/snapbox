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
//! - [`assert_eq`][crate::assert_eq] and [`assert_matches`] for reusing diffing / pattern matching for non-snapshot testing
//! - [`assert_eq`][crate::assert_eq_path] and [`assert_matches_path`] for one-off assertions with the snapshot stored in a file
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
//! snapbox::assert_matches("Hello many people!", "Hello [..] people!");
//! ```
//!
//! [`Assert`]
//! ```rust,no_run
//! let actual = "...";
//! snapbox::Assert::new()
//!     .action_env("SNAPSHOT_ACTION")
//!     .matches_path(actual, "tests/fixtures/help_output_is_clean.txt");
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
//! .action_env("SNAPSHOT_ACTION")
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
//! fn test(input_path: &std::path::Path) -> Result<usize, Box<std::error::Error>> {
//!     let raw = std::fs::read_to_string(input_path)?;
//!     let num = raw.parse::<usize>()?;
//!
//!     let expected = num + 10;
//!
//!     Ok(expected)
//! }
//! ```
//!
//! [trycmd]: https://docs.rs/trycmd

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod action;
mod assert;
mod data;
mod error;
mod substitutions;

pub mod cmd;
pub mod path;
pub mod report;
pub mod utils;

#[cfg(feature = "harness")]
pub mod harness;

pub use action::Action;
pub use assert::Assert;
pub use data::Data;
pub use error::Error;
pub use snapbox_macros::debug;
pub use substitutions::Substitutions;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Check if a value is the same as an expected value
///
/// When the content is text, newlines are normalized.
///
/// ```rust
/// let output = "something";
/// snapbox::assert_eq(output, "something");
/// ```
#[track_caller]
pub fn assert_eq(actual: impl Into<crate::Data>, expected: impl Into<crate::Data>) {
    Assert::new().eq(actual, expected);
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
/// let output = "something";
/// snapbox::assert_matches(output, "so[..]g");
/// ```
#[track_caller]
pub fn assert_matches(actual: impl Into<crate::Data>, pattern: impl Into<crate::Data>) {
    Assert::new().matches(actual, pattern);
}

/// Check if a value matches the content of a file
///
/// When the content is text, newlines are normalized.
///
/// ```rust,no_run
/// let output = "something";
/// snapbox::assert_eq_path(output, "tests/snapshots/output.txt");
/// ```
#[track_caller]
pub fn assert_eq_path(actual: impl Into<crate::Data>, expected_path: impl AsRef<std::path::Path>) {
    Assert::new().eq_path(actual, expected_path);
}

/// Check if a value matches the pattern in a file
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
/// let output = "something";
/// snapbox::assert_matches_path(output, "tests/snapshots/output.txt");
/// ```
#[track_caller]
pub fn assert_matches_path(
    actual: impl Into<crate::Data>,
    pattern_path: impl AsRef<std::path::Path>,
) {
    Assert::new().matches_path(actual, pattern_path);
}
