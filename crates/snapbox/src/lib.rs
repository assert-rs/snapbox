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
//!   - See also [scrut](https://github.com/facebookincubator/scrut)
//! - [trycmd](https://crates.io/crates/trycmd): For running a lot of blunt tests (limited test predicates)
//!   - Particular attention is given to allow the test data to be pulled into documentation, like
//!     with [mdbook](https://rust-lang.github.io/mdBook/)
//! - [tryfn](https://crates.io/crates/tryfn): For running a lot of simple input/output tests
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
//! - [`assert_data_eq!`] for quick and dirty snapshotting
//!
//! Testing Commands:
//! - [`cmd::Command`]: Process spawning for testing of non-interactive commands
//! - [`cmd::OutputAssert`]: Assert the state of a [`Command`][cmd::Command]'s
//!   [`Output`][std::process::Output].
//!
//! Testing Filesystem Interactions:
//! - [`dir::DirRoot`]: Working directory for tests
//! - [`Assert`]: Diff a directory against files present in a pattern directory
//!
//! You can also build your own version of these with the lower-level building blocks these are
//! made of.
//!
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!
//! # Examples
//!
//! [`assert_data_eq!`]
//! ```rust
//! snapbox::assert_data_eq!("Hello many people!", "Hello [..] people!");
//! ```
//!
//! [`Assert`]
//! ```rust,no_run
//! let actual = "...";
//! snapbox::Assert::new()
//!     .action_env("SNAPSHOTS")
//!     .eq(actual, snapbox::file!["help_output_is_clean.txt"]);
//! ```
//!
//! [trycmd]: https://docs.rs/trycmd

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod macros;

pub mod assert;
pub mod cmd;
pub mod data;
pub mod dir;
pub mod filter;
pub mod report;
pub mod utils;

pub use assert::Assert;
pub use data::Data;
pub use data::IntoData;
#[cfg(feature = "json")]
pub use data::IntoJson;
pub use data::ToDebug;
pub use filter::RedactedValue;
pub use filter::Redactions;
#[doc(hidden)]
pub use snapbox_macros::debug;

/// Easier access to common traits
pub mod prelude {
    pub use crate::IntoData;
    #[cfg(feature = "json")]
    pub use crate::IntoJson;
    pub use crate::ToDebug;
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

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
