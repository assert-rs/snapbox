# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.13.5] - 2022-08-01

### Fixes

- Make diffs viewable with large output by eliding large sections of unchanged content

## [0.13.4] - 2022-05-02

### Fixes

- Allow 3+ backtick code fences to escape code fences in stdout

## [0.13.3] - 2022-03-21

### Documentation

- Link to examples from docs.rs

## [0.13.2] - 2022-03-08

### Fixes

- Have standard gutter divider in Diffs

## [0.13.1] - 2022-03-07

### Fixes

- Don't swap actual/expected with file diffs

## [0.13.0] - 2022-03-07

### Breaking Changes

None known, just a significant internal release

### Features

- Diffs now show full context, with highlighting of changes within lines and a marker for no newline at end of file

## [0.12.2] - 2022-01-27

### Fixes

- Auto-detect binary files and don't diff them

## [0.12.1] - 2022-01-21

### Features

- Allow `stdin`, `stdout`, and `stderr` in toml file

## [0.12.0] - 2022-01-14

### Breaking Change

- Normalize `\\` to `/` to help with paths on Windows

### Fixes

- Normalize `\\` to `/` to help with paths on Windows

## [0.11.1] - 2022-01-14

### Fixes

Substitutions
- Ensure `[CWD]` captures `/private` on macOS when sandboxing

## [0.11.0] - 2022-01-14

### Breaking Change

- Any of the below fixes could break

### Fixes

Substitutions
- Ensure consistent choice between `[ROOT]` and `[CWD]` when they are the same
- Prefer `[CWD]` to `[ROOT]` without sandbox
- Consistently exclude trailing `/` with `[CWD]` / `[ROOT]`
- Improve sandbox-unsupported error

## [0.10.0] - 2022-01-13

### Breaking Change

Config
- Re-defined `fs.base` root to the `.toml`'s directory, like `fs.cwd`

### Fixes

Config
- Support `.in` directories that were symlinks on Linux but Windows checked out as files.
- Re-defined `fs.base` root to the `.toml`'s directory, like `fs.cwd`

## [0.9.1] - 2022-01-11

## [0.9.0] - 2022-01-05

### Breaking Change

- `md`: `bash` and `sh` info strings are now ignored, switch to `console`
- `md`: A blank line is needed after each command's output

### Fixes

- Use more appropriate `console` info string for `md`
- Allow testing commands without trailing newline in `md` files

## [0.8.3] - 2021-12-16

### Fixes

- Dependencies requirements reflect minimum versions needed

## [0.8.2] - 2021-11-30

### Fixes

- Only skip after a failure when they share mutable state (the file system).

## [0.8.1] - 2021-11-23

### Fixes

- Fix variable substitutions

## [0.8.0] - 2021-11-23

### Breaking Change

- Instead of hard erroring on unknown bins, we ignore

### Fixes

- Always apply substitutions (we were missing one spot)
- Don't fail because a substitution evaluates to ""
- Ignore tests with unknown bins to work better with feature flags

## [0.7.0] - 2021-11-16

### Breaking Change

- `...` for character matching (and not line matching) has switched to `[..]`.

### Features

- Variable substitutions for easier maintenance
  - User defined with `TestCases::insert_var`
  - Built-in with `[..]`, `[EXE]`, `[ROOT]`, and `[CWD]`.

## [0.6.0] - 2021-11-12

### Features

- Inline eliding

## [0.5.1] - 2021-11-12

### Fixes

- Test case showed a success despite it failing (summary reported it failed)

## [0.5.0] - 2021-11-12

### Features

- Exposed additional information through the `debug` feature flag

## [0.4.2] - 2021-11-11

### Fixes

- Reduce example-build output by relegating it to the new `debug` feature flag

## [0.4.1] - 2021-11-11

### Fixes

- `TRYCMD=overwrite` support for `*.trycmd` files
- `TRYCMD=dump` creates a file per step in `*.trycmd`, rather than overwriting the file from a prior step

### Regressions

- `TRYCMD=overwrite` will report corrected test cases as failures

## [0.4.0] - 2021-11-10

### Breaking Changes

- `*.trycmd` syntax now requires fenced code blocks for test cases
- `*.trycmd` syntax requires specific ordering between `$`, `>`, and `?`
- `*.trycmd` files no longer support `*.stdout` and `*.stderr` files

### Features

- `*.trycmd` files
  - Markdown-compatible syntax and `*.md` extension support
  - Support multiple test cases
  - Support setting env variables inline
  - Support inline stdout with stderr redirected to stdout
- Ignore `.keep` files in `*.in` and `*.out`
- Support redirecting stderr to stdout

### Fixes

- Cleaned up tenses
- On failure, direct people to `TRYCMD` env variable

## [0.3.1] - 2021-11-08

### Fixes

- docs.rs to include the full API

## [0.3.0] - 2021-11-08

### Features

- `cargo_bin!` macro for looking up a bin correctly
  - Use it with either `TestCases::default_bin_path` or `TestCases::register_bin`
- `cargo::compile_example` and `cargo::compile_examples` to do snapshot testing of examples!
  - Use it with `TestCases::register_bin`

## [0.2.2] - 2021-11-08

### Features

- Allow `cmd.toml` to contain `args = "arg1 'arg with space'"`

## [0.2.1] - 2021-11-06

### Features

- Show text failures as diffs

## [0.2.0] - 2021-11-06

### Breaking Changes

- We will now interpret `...` in files as wildcards
- The TOML `cwd` key is now `fs.cwd`

### Features

- `...\n` in `*.stdout`, `*.stderr`, or `*.out/*` will match multiple lines (non-greedy) 
- Infer the CWD from a `*.in/`
- Sandbox in a tempdir tests that mutate the CWD
- `*.out/` for specifying files to verify in the sandbox

### Fixes

- Show all failures for a command, not just the first

## [0.1.1] - 2021-11-05

### Fixes

- Tweaks to output

## [0.1.0] - 2021-11-05

<!-- next-url -->
[Unreleased]: https://github.com/assert-rs/trycmd/compare/v0.13.5...HEAD
[0.13.5]: https://github.com/assert-rs/trycmd/compare/v0.13.4...v0.13.5
[0.13.4]: https://github.com/assert-rs/trycmd/compare/v0.13.3...v0.13.4
[0.13.3]: https://github.com/assert-rs/trycmd/compare/v0.13.2...v0.13.3
[0.13.2]: https://github.com/assert-rs/trycmd/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/assert-rs/trycmd/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/assert-rs/trycmd/compare/v0.12.2...v0.13.0
[0.12.2]: https://github.com/assert-rs/trycmd/compare/v0.12.1...v0.12.2
[0.12.1]: https://github.com/assert-rs/trycmd/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/assert-rs/trycmd/compare/v0.11.1...v0.12.0
[0.11.1]: https://github.com/assert-rs/trycmd/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/assert-rs/trycmd/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/assert-rs/trycmd/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/assert-rs/trycmd/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/assert-rs/trycmd/compare/v0.8.3...v0.9.0
[0.8.3]: https://github.com/assert-rs/trycmd/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/assert-rs/trycmd/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/assert-rs/trycmd/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/assert-rs/trycmd/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/assert-rs/trycmd/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/assert-rs/trycmd/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/assert-rs/trycmd/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/assert-rs/trycmd/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/assert-rs/trycmd/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/assert-rs/trycmd/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/assert-rs/trycmd/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/assert-rs/trycmd/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/assert-rs/trycmd/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/assert-rs/trycmd/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/assert-rs/trycmd/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/assert-rs/trycmd/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/assert-rs/assert_cmd/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/assert-rs/assert_cmd/compare/5ed8849...v0.1.0
