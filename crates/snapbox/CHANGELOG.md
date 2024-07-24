# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

### Features

- Provide `Assert::redactions`

## [0.6.14] - 2024-07-23

### Compatibility

- Deprecated `IntoData::json` in favor of `IntoData::is_json`
- Deprecated `IntoData::json_lines` in favor of `IntoData::is_jsonlines`
- Deprecated `IntoData::term_svg` in favor of `IntoData::is_termsvg`

### Features

- Added `IntoData::against`, with shortcuts for `json` and `jsonlines`, for different `actual` and `expected` formats

## [0.6.13] - 2024-07-12

## [0.6.12] - 2024-07-10

## [0.6.11] - 2024-07-10

### Fixes

- `str![]`: fail, rather than panic, on divergent overlapping overwrites

## [0.6.10] - 2024-06-10

### Features

- Expose `Assert::normalize`

## [0.6.9] - 2024-06-07

### Fixes

- Allow overwriting if `Data::is` fails

## [0.6.8] - 2024-06-07

### Features

- Add data format helpers to `IntoData`

## [0.6.7] - 2024-05-29

### Fixes

- Allow `_` in redaction placeholder

## [0.6.6] - 2024-05-29

### Fixes

- Don't stop redacting on first mismatch

## [0.6.5] - 2024-05-27

### Fixes

- Don't stop `.term.svg` redactions just because the document widths are different

## [0.6.4] - 2024-05-25

### Fixes

- `data.unordered()` should be additive and not clear other filters

## [0.6.3] - 2024-05-24

### Fixes

- Make diff headers consistent line str's showing relative paths like file-backed expected

## [0.6.2] - 2024-05-24

### Compatibility

- Deprecated `Assert::substitutions` in favor of `Assert::redact_with`

## [0.6.1] - 2024-05-24

### Fixes

- Apply redactions to `.term.svg` files

## [0.6.0] - 2024-05-23

### Compatibility

Breaking
- Removed deprecated functionality
- Removed `str![[]]` indent support
- Added trailing newline to `str![[]]`
- Made `DataFormat` future-proof
- `...` / `[..]` processing has been tweaked
- Redactions no longer normalize the value (except the new `Path` redactions)
- Change `FilterRedactions` into `NormalizedToExpected`

Other
- Deprecate `eq_` fn's in favor of `eq`

### Features

- jsonlines support
- Regex redactions
- `Path` redactions
- Json "all keys" wildcard support
- Unordered lines / json array support

### Fixes

- Handle more cases with `...`
- Be smarter about which redactions are processed first
- Redact json values on mismatch
- Filter and redact json keys

## [0.5.14] - 2024-05-23

### Compatibility

- Deprecated `filter::NormalizePaths` in favor of `filter::FilterPaths`
- Deprecated `filter::NormalizeNewlines` in favor of `filter::FilterNewlines`

## [0.5.13] - 2024-05-21

### Documentation

- Fix deprecation message

## [0.5.12] - 2024-05-17

### Compatibility

- Deprecated `Harness` in favor of the `tryfn` crate

## [0.5.11] - 2024-05-17

### Compatibility

- Deprecated `Assert::eq(expected, actual)` in favor of `Assert::eq_(actual, expected.raw())`
- Deprecated `Assert::matches(expected, actual)` in favor of `Assert::eq_(actual, expected)`
- Deprecated `OutputAssert::stdout_eq(expected, actual)` in favor of `OutputAssert::stdout_eq_(actual, expected.raw())`
- Deprecated `OutputAssert::stdout_matches(expected, actual)` in favor of `OutputAssert::stdout_eq_(actual, expected)`
- Deprecated `OutputAssert::stderr_eq(expected, actual)` in favor of `OutputAssert::stderr_eq_(actual, expected.raw())`
- Deprecated `OutputAssert::stderr_matches(expected, actual)` in favor of `OutputAssert::stderr_eq_(actual, expected)`
- Deprecated `data::Data::normalize` in favor of `filter::Filter::filter`
- Deprecated `data::Normalize::normalize` in favor of `filter::Filter::filter`
- Deprecated `data::NormalizeNewlines` in favor of `filter::NormalizeNewlines`
- Deprecated `data::NormalizePaths` in favor of `filter::NormalizePaths`
- Deprecated `data::NormalizeMatches` in favor of `filter::FilterRedactions`
- Deprecated `data::Substitutions` in favor of `filter::Redactions`
- Deprecated `path` feature / mod in favor of `dir` feature / mod
- Deprecated `path::PathFixture` in favor of `dir::DirRoot`
- Deprecated `Error` in favor of `assert::Error`
- Deprecated `Result` in favor of `assert::Result`
- Deprecated `Action` in favor of `assert::Action`
- Deprecated `DEFAULT_ACTION_ENV` in favor of `assert::DEFAULT_ACTION_ENV`
- Deprecated `utils::normalize_lines` in favor of `filter::normalize_lines`
- Deprecated `utils::normalize_paths` in favor of `filter::normalize_paths`
- Deprecated `utils::normalize_text`

### Features

- Add `Assert::eq_(actual, expected)`
- Add `OutputAssert::stdout_eq_(actual, expected)`
- Add `OutputAssert::stderr_eq_(actual, expected)`
- Add `assert_data_eq!(actual, expected)`
- Add `prelude` for easier access to `IntoData`, `IntoJson`, and `ToDebug`
- Add `IntoData` for easier data conversions
- Add `IntoJson` for easier data conversions
- Add `Data::raw` to disable filters on expected values
- Add `Harness::with_assert` for greater customization
- Add `Assert::try_eq` for non-panicking asserts
- Add `Assert::selected_action` to see whats chosen

## [0.5.10] - 2024-05-15

## [0.5.9] - 2024-03-13

### Fixes

- Don't strip on `str![].indent(false)` to work with lines with indentation

## [0.5.8] - 2024-03-01

### Fixes

- Make auto-path snapshots more deterministic

## [0.5.7] - 2024-02-27

### Fixes

- Respect `Assert::normalize_paths(false)` when running `subset_matches`

## [0.5.6] - 2024-02-22

### Fixes

- *(term-svg)* Reduce noise in SVG snapshots

## [0.5.5] - 2024-02-22

## [0.5.4] - 2024-02-22

## [0.5.3] - 2024-02-22

### Fixes

- *(term-svg)* Correctly coerce "actual" to this type on when the snapshot doesn't yet exist
- *(term-svg)* Ignore irrelevant details when checking for failuress
- *(term-svg)* Don't show irrelevant details in the diff

## [0.5.2] - 2024-02-19

### Features

- Add `term-svg` feature for snapshotting of terminal styling by ending files with `.term.svg`

## [0.5.1] - 2024-02-15

### Fixes

- Fixed regression where you couldn't overwrite a snapshot that doesn't exist yet

## [0.5.0] - 2024-02-14

### Breaking Change

- `_path` assertions have been replaced by regular assertions accepting and updating `Data::read_from` / `file![]`
- Renamed `Data::try_coerce` to `Data::coerce_to`
- Normalization types and `DataFormat` were moved to the `data` mod

### Features

- `str![]` macro for inline snapshots
  - `coerce_to` for specifying the structure of the snapshot
- `file!["<path-relative-to-rs>"]` macro for test-relative snapshots
  - `_` placeholder can be used for auto-generated file names
  - `: Json` (see `DataFormat`) type ascriptions for specifying the structure of the snapshot
- `ToDebug` extension trait for easier snapshot testing of debug output
- Help catch swapped parameters to assertions by panicking if the "actual" has a source to update but not the "expected"
- Allow inserting multiple values for the same variable, to handle multiple forms of the same value (like UNC paths vs regular paths)
- `path::current_rs!()` macro for an absolute path of `std::fs::file!`
- `path::current_dir!()` macro to get the absolute path to `std::fs::file!`
- `path::cargo_rustc_current_dir()` macro as a polyfill for the unstable `CARGO_RUSTC_CURRENT_DIR`

## [0.4.17] - 2024-02-08

## [0.4.16] - 2024-01-12

### Internal

- Update dependency

## [0.4.15] - 2023-12-08

### Fixes

- Actually match standard diff colors

## [0.4.14] - 2023-10-02

### Fixes

- Match standard diff colors

## [0.4.13] - 2023-09-28

### Internal

- Update `anstream`

## [0.4.12] - 2023-08-24

### Compatibility

- MSRV bumped to 1.70.0

### Performance

- Improved build-times by dropping the `is-terminal` dependency

### Features

- Wildcard support for structured data

## [0.4.11] - 2023-04-13

### Internal

- Dependency update

## [0.4.10] - 2023-03-16

### Internal

- Dependency update

## [0.4.9] - 2023-03-14

### Compatibility

- Update MSRV to 1.64.0
- Deprecated `color-auto` feature, its now automatic

## Fixes

- Improved terminal detection for color

## [0.4.8] - 2023-02-28

### Internal

- Update dependencies

## [0.4.7] - 2023-02-20

### Internal

- Reduce dependencies

## [0.4.6] - 2023-02-19

### Features

- Added `examples` feature to expose example-compilation functions in the `cmd` module

## [0.4.5] - 2023-02-15

### Internal

- Dependency updates

## [0.4.4] - 2023-01-04

## [0.4.3] - 2022-11-26

## [0.4.2] - 2022-11-24

## [0.4.1] - 2022-11-04

### Fixes

- Report signal that terminated a command

## [0.4.0] - 2022-09-23

### Features

- Initial json support

### Fixes

- Hide optional dependencies

## [0.3.3] - 2022-08-15

### Fixes

- Don't hang when merging stderr with stdout on large output (#121)

## [0.3.2] - 2022-08-15

### Breaking Changes

- `Assert::eq` and `PathDiff::subset_eq_iter` normalize newlines but not paths (broken in 0.3.0) 

## [0.3.1] - 2022-08-03

### Features

- Added `Assert::normalize_paths` to allow disabling path normalization

## [0.3.0] - 2022-08-01

### Breaking Changes

- `Data::read_from` now takes a desired data format rather than a bool between text/binary
- `Data::try_text` was replaced with `Data::try_coerce`
- `Data::as_bytes` was replaced with `Data::to_bytes`
- `Assert::eq` (and everything built on it) normalizes paths but not `PathDiff::subset_eq_iter` (bug)

### Fixes

- Make diffs viewable with large output by eliding large sections of unchanged content

## [0.2.10] - 2022-05-02

### Fixes

- Link in documentation

## [0.2.9] - 2022-03-09

## [0.2.8] - 2022-03-08

### Fixes

- In Diffs, emphasize role over name

## [0.2.7] - 2022-03-08

### Features

- Configure asserts on the `Command` itself

### Fixes

- When manually setting `Action`, overwrite the env var

## [0.2.6] - 2022-03-08

### Fixes

- Show relpath in diff header where possible

## [0.2.5] - 2022-03-08

### Fixes

- Have standard gutter divider in Diffs
- Improve command assertion output

## [0.2.4] - 2022-03-08

### Fixes

- Create target directory when needed

## [0.2.3] - 2022-03-08

### Features

- Simple path assert

## [0.2.2] - 2022-03-08

### Features

- Defaulted the action env to `SNAPSHOTS`
- Made path function more accepting of inputs

## [0.2.1] - 2022-03-07

### Fixes

- Remove need for doing `<VAR>=overwrite` twice due to lack of normalization on first call

## [0.2.0] - 2022-03-07

### Breaking Changes

- Name changed from `fs_snapshot`

### Features

- More flexible return types
- Diffs now show full context, with highlighting of changes within lines and a marker for no newline at end of file
- Everything needed to implement `trycmd` is now included

## [0.1.2] - 2022-01-11

## [0.1.1] - 2021-12-28

### Fixes

- Working no-default-features

## [0.1.0] - 2021-12-28

<!-- next-url -->
[Unreleased]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.14...HEAD
[0.6.14]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.13...snapbox-v0.6.14
[0.6.13]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.12...snapbox-v0.6.13
[0.6.12]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.11...snapbox-v0.6.12
[0.6.11]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.10...snapbox-v0.6.11
[0.6.10]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.9...snapbox-v0.6.10
[0.6.9]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.8...snapbox-v0.6.9
[0.6.8]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.7...snapbox-v0.6.8
[0.6.7]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.6...snapbox-v0.6.7
[0.6.6]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.5...snapbox-v0.6.6
[0.6.5]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.4...snapbox-v0.6.5
[0.6.4]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.3...snapbox-v0.6.4
[0.6.3]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.2...snapbox-v0.6.3
[0.6.2]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.1...snapbox-v0.6.2
[0.6.1]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.6.0...snapbox-v0.6.1
[0.6.0]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.14...snapbox-v0.6.0
[0.5.14]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.13...snapbox-v0.5.14
[0.5.13]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.12...snapbox-v0.5.13
[0.5.12]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.11...snapbox-v0.5.12
[0.5.11]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.10...snapbox-v0.5.11
[0.5.10]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.9...snapbox-v0.5.10
[0.5.9]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.8...snapbox-v0.5.9
[0.5.8]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.7...snapbox-v0.5.8
[0.5.7]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.6...snapbox-v0.5.7
[0.5.6]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.5...snapbox-v0.5.6
[0.5.5]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.4...snapbox-v0.5.5
[0.5.4]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.3...snapbox-v0.5.4
[0.5.3]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.2...snapbox-v0.5.3
[0.5.2]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.1...snapbox-v0.5.2
[0.5.1]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.5.0...snapbox-v0.5.1
[0.5.0]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.17...snapbox-v0.5.0
[0.4.17]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.16...snapbox-v0.4.17
[0.4.16]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.15...snapbox-v0.4.16
[0.4.15]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.14...snapbox-v0.4.15
[0.4.14]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.13...snapbox-v0.4.14
[0.4.13]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.12...snapbox-v0.4.13
[0.4.12]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.11...snapbox-v0.4.12
[0.4.11]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.10...snapbox-v0.4.11
[0.4.10]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.9...snapbox-v0.4.10
[0.4.9]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.8...snapbox-v0.4.9
[0.4.8]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.7...snapbox-v0.4.8
[0.4.7]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.6...snapbox-v0.4.7
[0.4.6]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.5...snapbox-v0.4.6
[0.4.5]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.4...snapbox-v0.4.5
[0.4.4]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.3...snapbox-v0.4.4
[0.4.3]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.2...snapbox-v0.4.3
[0.4.2]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.1...snapbox-v0.4.2
[0.4.1]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.4.0...snapbox-v0.4.1
[0.4.0]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.3.3...snapbox-v0.4.0
[0.3.3]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.3.2...snapbox-v0.3.3
[0.3.2]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.3.1...snapbox-v0.3.2
[0.3.1]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.3.0...snapbox-v0.3.1
[0.3.0]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.10...snapbox-v0.3.0
[0.2.10]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.9...snapbox-v0.2.10
[0.2.9]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.8...snapbox-v0.2.9
[0.2.8]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.7...snapbox-v0.2.8
[0.2.7]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.6...snapbox-v0.2.7
[0.2.6]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.5...snapbox-v0.2.6
[0.2.5]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.4...snapbox-v0.2.5
[0.2.4]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.3...snapbox-v0.2.4
[0.2.3]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.2...snapbox-v0.2.3
[0.2.2]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.1...snapbox-v0.2.2
[0.2.1]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.0...snapbox-v0.2.1
[0.2.0]: https://github.com/assert-rs/trycmd/compare/72729043c3570a7447c311f498e163d844d49d99...snapbox-v0.2.0
[0.1.2]: https://github.com/assert-rs/fs_snapshot/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/assert-rs/fs_snapshot/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/assert-rs/fs_snapshot/compare/111b5143c55922f2f7a2b7791840a899f35ad5ba...v0.1.0
