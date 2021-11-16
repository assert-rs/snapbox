# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/assert-rs/trycmd/compare/v0.6.0...HEAD
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
