//! # Snapshot testing for a herd of CLI tests
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
//! ## Getting Started
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
//! ## Workflow
//!
//! To generate snapshots, run
//! ```console
//! $ TRYCMD=dump cargo test --test cli_tests
//! ```
//! This will write all of the `.stdout` and `.stderr` files in a `dump/` directory.
//!
//! You can then copy over to `tests/cmd` the cases you want to test
//!
//! To update snapshots, run
//! ```console
//! $ TRYCMD=overwrite cargo test --test cli_tests
//! ```
//! This will overwrite any existing `.stdout` and `.stderr` file in `tests/cmd`
//!
//! To filter the tests to those with `name1`, `name2`, etc in their file names, you can run:
//! ```console
//! cargo test --test cli_tests -- cli_tests trycmd=name1 trycmd=name2...
//! ```
//!
//! To debug what `trycmd` is doing, add the feature flag `debug`.
//!
//! ## File Formats
//!
//! Say you have `tests/cmd/help.trycmd`, `trycmd` will look for:
//! - `tests/cmd/help.in/`
//! - `tests/cmd/help.out/`
//!
//! For `tests/cmd/help.toml`, `trycmd` will look for:
//! - `tests/cmd/help.stdin`
//! - `tests/cmd/help.stdout`
//! - `tests/cmd/help.stderr`
//! - `tests/cmd/help.in/`
//! - `tests/cmd/help.out/`
//!
//! ### `*.trycmd`
//!
//! `*.trycmd` files are literate test cases good for:
//! - Markdown-compatible syntax for directly rendering them
//! - Terminal-like appearance for extracting subsections into documentation
//! - Reducing the proliferation of files
//! - Running multiple commands within the same temp dir
//!
//! The syntax is:
//! - Test cases live inside of ` ``` ` fenced code blocks
//!   - Everything out of them is ignored
//!   - Blocks with info strings with an unsupported language (not `trycmd`, `console`) or the
//!     `ignore` attribute are ignored
//! - "`$ `" line prefix starts a new command
//! - "`> `" line prefix appends to the prior command
//! - "`? <status>`" line indicates the exit code (like `echo "? $?"`) and `<status>` can be
//!   - An exit code
//!   - `success` *(default)*, `failed`, `interrupted`, `skipped`
//!  - All following lines are treated as stdout + stderr
//!
//! The command is then split with [shlex](https://crates.io/crates/shlex), allowing quoted content
//! to allow spaces.  The first argument is the program to run which maps to `bin.name` in the
//! `.toml` file.
//!
//! Example:
//! ~~~md
//! With the following code:
//! ```rust
//! println!("{}", message);
//! ```
//!
//! You get the following:
//! ```
//! $ my-cmd --print 'Hello World'
//! Hello
//! ```
//! ~~~
//!
//! ### `*.toml`
//!
//! As an alternative to `.trycmd`, the `toml` are good for:
//! - Precise control over current dir, stdin/stdout/stderr (including binary support)
//! - 1-to-1 with dumped results
//! - `TRYCMD=overwrite` support
//!
//! [schema](https://github.com/assert-rs/trycmd/blob/main/schema.json):
//! - `bin.name`: The name of the binary target from `Cargo.toml` to be used to find the file path
//!
//! #### `*.stdin`
//!
//! Data to pass to `stdin`.
//! - If not present, nothing will be written to `stdin`
//! - If `binary = false` in `*.toml` (the default), newlines and path separators will be normalized.
//!
//! #### `*.stdout` and `*.stderr`
//!
//! Expected results for `stdout` or `stderr`.
//! - If not present, we'll not verify the output
//! - If `binary = false` in `*.toml` (the default), newlines and path separators will be normalized before comparing
//!
//! **Eliding Content**
//!
//! Sometimes the output either includes:
//! - Content that changes from run-to-run (like time)
//! - Content out of scope of your tests and you want to exclude it to reduce brittleness
//!
//! To elide a section of content:
//! - `...` as its own line: match all lines until the next one.  This is equivalent of
//!   `\n(([^\n]*\n)*?`.
//! - `[..]` as part of a line: match any characters.  This is equivalent of `[^\n]*?`.
//! - `[EXE]` as part of the line: On Windows, matches `.exe`, ignored otherwise
//! - `[ROOT]` as part of the line: The root directory for where the test is running
//! - `[CWD]` as part of the line: The current working directory within the root
//! - `[YOUR_NAME_HERE]` as part of the line: See [`TestCases::insert_var`]
//!
//! We will preserve these with `TRYCMD=dump` and will make a best-effort at preserving them with
//! `TRYCMD=overwrite`.
//!
//! ### `*.in/`
//!
//! When present, this will automatically be picked as the CWD for the command.
//!
//! `.keep` files will be ignored but their parent directories will be created.
//!
//! ### `*.out/`
//!
//! When present, each file in this directory will be compared to generated or modified files.
//!
//! See also "Eliding Content" for `.stdout`
//!
//! `.keep` files will be ignored.

// Doesn't distinguish between incidental sharing vs essential sharing
#![allow(clippy::branches_sharing_code)]

#[macro_use]
mod macros;

pub mod cargo;
pub mod schema;

#[cfg(feature = "diff")]
pub(crate) mod diff;
pub(crate) mod elide;
pub(crate) mod lines;

mod cases;
mod color;
mod command;
mod error;
mod filesystem;
mod registry;
mod runner;
mod spec;

pub use cases::TestCases;
pub use error::Error;

pub(crate) use color::Palette;
pub(crate) use command::wait_with_input_output;
pub(crate) use filesystem::{shallow_copy, File, FilesystemContext, Iterate as FsIterate};
pub(crate) use registry::BinRegistry;
pub(crate) use runner::{Case, Mode, Runner};
pub(crate) use spec::RunnerSpec;
