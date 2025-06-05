//! # Snapshot testing for a herd of CLI tests
//!
//! > Treat your tests like cattle, instead of [pets](https://docs.rs/snapbox)
//!
//! `trycmd` is a test harness that will enumerate test case files and run them to verify the
//! results, taking inspiration from
//! [trybuild](https://crates.io/crates/trybuild) and [cram](https://bitheap.org/cram/).
//!
//! ## Which tool is right
//!
//! - [cram](https://bitheap.org/cram/): End-to-end CLI snapshotting agnostic of any programming language
//! - `trycmd`: For running a lot of blunt tests (limited test predicates)
//!   - Particular attention is given to allow the test data to be pulled into documentation, like
//!     with [mdbook](https://rust-lang.github.io/mdBook/)
//! - [snapbox](https://crates.io/crates/snapbox): When you want something like `trycmd` in one off
//!   cases or you need to customize `trycmd`s behavior.
//! - [assert_cmd](https://crates.io/crates/assert_cmd) +
//!   [assert_fs](https://crates.io/crates/assert_fs): Test cases follow a certain pattern but
//!   special attention is needed in how to verify the results.
//! - Hand-written test cases: for peculiar circumstances
//!
//! ## Getting Started
//!
//! To create a minimal setup, create a `tests/cli_tests.rs` with
//! ```rust,no_run
//! #[test]
//! fn cli_tests() {
//!     trycmd::TestCases::new()
//!         .case("tests/cmd/*.toml")
//!         .case("README.md");
//! }
//! ```
//! and write out your test cases in your `.toml` files along with examples in your `README.md`.
//!
//! Run this with `cargo test` like normal.  [`TestCases`] will enumerate all test case files and
//! run the contained commands, verifying they run as expected.
//!
//! To temporarily override the results, you can do:
//! ```rust,no_run
//! #[test]
//! fn cli_tests() {
//!     trycmd::TestCases::new()
//!         .case("tests/cmd/*.toml")
//!         .case("README.md")
//!         // See Issue #314
//!         .fail("tests/cmd/buggy-case.toml");
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
//! To debug what `trycmd` is doing, run `cargo test -F trycmd/debug`.
//!
//! ## File Formats
//!
//! For `tests/cmd/help.trycmd`, `trycmd` will look for:
//! - `tests/cmd/help.in/`
//! - `tests/cmd/help.out/`
//!
//! Say you have `tests/cmd/help.toml`, `trycmd` will look for:
//! - `tests/cmd/help.stdin`
//! - `tests/cmd/help.stdout`
//! - `tests/cmd/help.stderr`
//! - `tests/cmd/help.in/`
//! - `tests/cmd/help.out/`
//!
//! ### `*.trycmd`
//!
//! `*.trycmd` / `*.md` files are literate test cases good for:
//! - Markdown-compatible syntax for directly rendering them
//! - Terminal-like appearance for extracting subsections into documentation
//! - Reducing the proliferation of files
//! - Running multiple commands within the same temp dir (if a `*.out/` directory is present)
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
//!
//! With a `[[bin]]` like:
//! ```rust,ignore
//! fn main() {
//!     println!("Hello world");
//! }
//! ```
//!
//! You can verify a code block like:
//! ~~~md
//! ```console
//! $ my-cmd
//! Hello world
//!
//! ```
//! ~~~
//!
//! For a more complete example, see:
//! <https://github.com/assert-rs/trycmd/tree/main/examples/demo_trycmd>.
//!
//! ### `*.toml`
//!
//! As an alternative to `.trycmd`, the `toml` are good for:
//! - Precise control over current dir, stdin/stdout/stderr (including binary support)
//! - 1-to-1 with dumped results
//! - `TRYCMD=overwrite` support
//!
//! [See full schema](https://github.com/assert-rs/snapbox/blob/main/crates/trycmd/schema.json):
//! Basic parameters:
//! - `bin.name`: The name of the binary target from `Cargo.toml` to be used to find the file path
//! - `args`: the arguments (including flags and option) passed to the binary
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
//!   `\n(([^\n]*\n)*)?`.
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
//! Tests are assumed to not modify files in `*.in/` unless an `*.out/` is provided or
//! `fs.sandbox = true` is set in the `.toml` file.
//!
//! ### `*.out/`
//!
//! When present, each file in this directory will be compared to generated or modified files.
//!
//! See also "Eliding Content" for `.stdout`
//!
//! `.keep` files will be ignored.
//!
//! Note: This implies `fs.sandbox = true`.
//!
//! ## Examples
//!
//! - Simple cargo binary: [trycmd's integration tests](https://github.com/assert-rs/snapbox/blob/main/crates/trycmd/tests/cli_tests.rs)
//! - Simple example: [trycmd's integration tests](https://github.com/assert-rs/snapbox/blob/main/crates/trycmd/tests/example_tests.rs)
//! - [typos](https://github.com/crate-ci/typos) (source code spell checker)
//! - [clap](https://github.com/clap-rs/clap/) (CLI parser) to test examples
//!
//! ## Related crates
//!
//! For testing command line programs.
//! - [escargot][escargot] for more control over configuring the crate's binary.
//! - [duct][duct] for orchestrating multiple processes.
//!   - or [commandspec] for easier writing of commands
//! - [`assert_cmd`][assert_cmd] for test cases that are individual pets, rather than herd of cattle
//! - [`assert_fs`][assert_fs] for filesystem fixtures and assertions.
//!   - or [tempfile][tempfile] for scratchpad directories.
//! - [rexpect][rexpect] for testing interactive programs.
//! - [dir-diff][dir-diff] for testing file side-effects.
//!
//! For snapshot testing:
//! - [insta](https://crates.io/crates/insta)
//! - [fn-fixture](https://crates.io/crates/fn-fixture)
//! - [runt](https://crates.io/crates/runt)
//!   - [turnt](https://github.com/cucapra/turnt)
//!   - [cram](https://bitheap.org/cram/)
//! - [term-transcript](https://crates.io/crates/term-transcript): CLI snapshot testing, including colors
//!
//! [escargot]: http://docs.rs/escargot
//! [rexpect]: https://crates.io/crates/rexpect
//! [dir-diff]: https://crates.io/crates/dir-diff
//! [tempfile]: https://crates.io/crates/tempfile
//! [duct]: https://crates.io/crates/duct
//! [assert_fs]: https://crates.io/crates/assert_fs
//! [assert_cmd]: https://crates.io/crates/assert_cmd
//! [commandspec]: https://crates.io/crates/commandspec

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

pub mod cargo;
pub mod schema;

mod cases;
mod registry;
mod runner;
mod spec;

pub use cases::TestCases;
pub use snapbox::assert::Error;

pub(crate) use registry::BinRegistry;
pub(crate) use runner::{Case, Mode, Runner};
pub(crate) use spec::RunnerSpec;

pub(crate) use snapbox::Data;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
