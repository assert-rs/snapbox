//! # Snapshot testing toolbox
//!
//! > When you have to treat your tests like pets, instead of [cattle][trycmd]
//!
//! `snapbox` is for when:
//! - You need a lot of customization around an individual test
//! - You need to build your own test harness instead of using something like [`trycmd`][trycmd]
//!
//! In-memory:
//! - [`assert_eq`][crate::assert_eq] and [`assert_matches`] for reusing diffing / pattern matching for non-snapshot testing
//! - [`file_assert`] for one-off assertions with the snapshot stored in a file
//! - [`harness::Harness`] for discovering test inputs and asserting against snapshot files:
//!
//! Filesystem:
//! - [`path::PathFixture`]
//! - [`path::path_assert()`]
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
//! [`file_assert`]
//! ```rust,no_run
//! let actual = "...";
//! snapbox::file_assert()
//!     .action_env("SNAPSHOT_ACTION")
//!     .matches(actual, "tests/fixtures/help_output_is_clean.txt");
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

mod action;
mod data;
mod error;
mod file;
mod substitutions;

pub mod cmd;
pub mod path;
pub mod report;
pub mod utils;

#[cfg(feature = "harness")]
pub mod harness;

pub use action::Action;
pub use data::assert_eq;
pub use data::assert_matches;
pub use data::Data;
pub use error::Error;
pub use file::file_assert;
pub use file::FileAssert;
pub use path::path_assert;
pub use path::PathAssert;
pub use snapbox_macros::debug;
pub use substitutions::Substitutions;

pub type Result<T, E = Error> = std::result::Result<T, E>;
