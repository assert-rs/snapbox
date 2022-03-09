# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/assert-rs/trycmd/compare/snapbox-v0.2.9...HEAD
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
