//! ## Snapshot testing for a herd of CLI tests
//!
//! `trycmd` is a test harness that will enumerate test case files and run them to verify the
//! results, taking inspiration from
//! [trybuild](https://crates.io/crates/trybuild) and [cram](https://bitheap.org/cram/).
//!
//! Which tool is right:
//! - Hand-written test cases: for peculiar circumstances
//! - [assert_cmd](https://crates.io/crates/assert_cmd): Test cases follow a certain pattern but
//!   special attention is needed in how to verify the results.
//! - `trycmd`: For running a lot of blunt tests (limited test predicates)
//!   - Particular attention is given to allow the test data to be pulled into documentation, like
//!     with [mdbook](https://rust-lang.github.io/mdBook/)
//! - [cram](https://bitheap.org/cram/): For cases agnostic of any programming language
//!
//! ### Getting Started
//!
//! To create a minimal setup, create a `tests/cli_tests.rs` with
//! ```rust,no_run
//! #[test]
//! fn cli_tests() {
//!     trycmd::TestCases::new()
//!         .case("tests/cmd/*.trycmd");
//! }
//! ```
//!
//! The test can be run with `cargo test`.  This will enumerate all `.trycmd` files and run
//! them as test cases, failing if they do not pass.
//!
//! To temporarily override the results, you can do:
//! ```rust,no_run
//! #[test]
//! fn cli_tests() {
//!     trycmd::TestCases::new()
//!         .case("tests/cmd/*.trycmd")
//!         // See Issue #314
//!         .fail("tests/cmd/buggy-case.trycmd");
//! }
//! ```
//!
//! ### File Formats
//!
//! Say you have `tests/cmd/help.trycmd` (or `help.toml`), `trycmd` will look for:
//! - `tests/cmd/help.stdin`
//! - `tests/cmd/help.stdout`
//! - `tests/cmd/help.stderr`
//!
//! #### `*.trycmd`
//!
//! A `trycmd` file is just a command with arguments, with the arguments split with [shlex](https://crates.io/crates/shlex).
//!
//! The command is interpreted as `bin.name` in a `toml` file.
//!
//! #### `*.toml`
//!
//! As an alternative to `.trycmd`, he `toml` files give you a lot more control over how your command runs.
//!
//! [schema](https://github.com/assert-rs/trycmd/blob/main/schema.json):
//! - `bin.name`: The name of the binary target from `Cargo.toml` to be used to find the file path
//!
//! #### `*.stdin`
//!
//! Data to pass to `stdin`.
//! - If not present, nothing will be written to `stdin`
//! - If `binary = false` in `*.toml` (the default), newlines will be normalized.
//!
//! #### `*.stdout` and `*.stderr`
//!
//! Expected results for `stdout` or `stderr`.
//! - If not present, we'll not verify the output
//! - If `binary = false` in `*.toml` (the default), newlines will be normalized before comparing
//!
//! #### `*.in/`
//!
//! When present, this will automatically be picked as the CWD for the command
//!
//! ### Workflow
//!
//! To generate snapshots, run
//! ```bash
//! $ TRYCMD=dump cargo test --test cli_tests
//! ```
//! This will write all of the `.stdout` and `.stderr` files in a `dump/` directory.
//!
//! You can then copy over to `tests/cmd` the cases you want to test
//!
//! To update snapshots, run
//! ```bash
//! $ TRYCMD=overwrite cargo test --test cli_tests
//! ```
//! This will overwrite any existing `.stdout` and `.stderr` file in `tests/cmd`
//!
//! When iterating on a test, you can run:
//! ```bash
//! cargo test --test cli_tests -- cli_tests trycmd=name1 trycmd=name2...
//! ```
//! To filter the tests to those with `name1`, `name2`, etc in their file names.

mod cargo;
mod cases;
mod color;
mod command;
mod filesystem;
mod runner;
pub mod schema;
mod spec;

pub use cases::TestCases;

pub(crate) use cargo::cargo_bin;
pub(crate) use color::Palette;
pub(crate) use command::wait_with_input_output;
pub(crate) use filesystem::{FilesystemContext, Iterate as FsIterate};
pub(crate) use runner::{Case, Mode, Runner};
pub(crate) use schema::{Bin, CommandStatus, Env, TryCmd};
pub(crate) use spec::RunnerSpec;
