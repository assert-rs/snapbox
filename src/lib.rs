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
//! `.trycmd` files provide a more visually familiar way of specifying test cases.
//!
//! The basic syntax is:
//! - "`$ `" line prefix starts a new command
//! - "`> `" line prefix appends to the prior command
//! - "`? <status>`" line indicates the exit code (like `echo "? $?"`) and `<status>` can be
//!   - An exit code
//!   - `success` *(default)*, `failed`, `interrupted`, `skipped`
//!
//! The command is then split with [shlex](https://crates.io/crates/shlex), allowing quoted content
//! to allow spaces.  The first argument is the program to run which maps to `bin.name` in the
//! `.toml` file.
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
//! ##### Eliding Content
//!
//! Sometimes the output either includes:
//! - Content that changes from run-to-run (like time)
//! - Content out of scope of your tests and you want to exclude it to reduce brittleness
//!
//! To elide a section of content:
//! - `...` as its own line will match all lines until the next one.  This is equivalent of
//!   `(([^\n]*\n)*?`.
//!
//! We will preserve these with `TRYCMD=dump` and will make a best-effort at preserving them with
//! `TRYCMD=overwrite`.
//!
//! #### `*.in/`
//!
//! When present, this will automatically be picked as the CWD for the command.
//!
//! `.keep` files will be ignored but their parent directories will be created.
//!
//! #### `*.out/`
//!
//! When present, each file in this directory will be compared to generated or modified files.
//!
//! See also "Eliding Content" for `.stdout`
//!
//! `.keep` files will be ignored.
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

// Doesn't distinguish between incidental sharing vs essential sharing
#![allow(clippy::branches_sharing_code)]

pub mod cargo;
pub mod schema;

mod cases;
mod color;
mod command;
#[cfg(feature = "diff")]
pub(crate) mod diff;
pub(crate) mod elide;
mod error;
mod filesystem;
pub(crate) mod lines;
mod registry;
mod runner;
mod spec;

pub use cases::TestCases;
pub use error::Error;

pub(crate) use color::Palette;
pub(crate) use command::wait_with_input_output;
pub(crate) use filesystem::{shallow_copy, FilesystemContext, Iterate as FsIterate};
pub(crate) use registry::BinRegistry;
pub(crate) use runner::{Case, Mode, Runner};
pub(crate) use spec::RunnerSpec;
