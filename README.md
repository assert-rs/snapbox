# trycmd

> Snapshot testing for a herd of CLI tests

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/trycmd.svg)
[![Crates Status](https://img.shields.io/crates/v/trycmd.svg)](https://crates.io/crates/trycmd)

`trycmd` aims to simplify the process for running a large collection of
end-to-end CLI test cases, taking inspiration from
[trybuild](https://crates.io/crates/trybuild).

## Example

Here's a trivial example:

```rust,no_run
#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.trycmd");
}
```

See the [docs](http://docs.rs/trycmd) for more.

## Users

- [typos](https://github.com/crate-ci/typos) (source code spell checker)
  - See [port from `assert_cmd`](https://github.com/crate-ci/typos/compare/a8ae8a5..cdfdc4084c928423211c6a80acbd24dbed7108f6)

## Relevant crates

For testing command line programs.
* [`assert_cmd`][assert_cmd] for test cases that are individual pets, rather than herd of cattle
* [escargot][escargot] for more control over configuring the crate's binary.
* [duct][duct] for orchestrating multiple processes.
  * or [commandspec] for easier writing of commands
* [rexpect][rexpect] for testing interactive programs.
* [`assert_fs`][assert_fs] for filesystem fixtures and assertions.
  * or [tempfile][tempfile] for scratchpad directories.
* [dir-diff][dir-diff] for testing file side-effects.

For snapshot testing:
- [insta](https://crates.io/crates/insta)
- [fn-fixture](https://crates.io/crates/fn-fixture)
- [runt](https://crates.io/crates/runt)
  - [turnt](https://github.com/cucapra/turnt)
  - [cram](https://bitheap.org/cram/)

[escargot]: http://docs.rs/escargot
[rexpect]: https://crates.io/crates/rexpect
[dir-diff]: https://crates.io/crates/dir-diff
[tempfile]: https://crates.io/crates/tempfile
[duct]: https://crates.io/crates/duct
[assert_fs]: https://crates.io/crates/assert_fs
[assert_cmd]: https://crates.io/crates/assert_cmd
[commandspec]: https://crates.io/crates/commandspec

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Crates.io]: https://crates.io/crates/trycmd
[Documentation]: https://docs.rs/trycmd
