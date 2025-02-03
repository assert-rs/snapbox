# trycmd

> Treat your tests like cattle, instead of [pets](https://docs.rs/snapbox)

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/trycmd.svg)
[![Crates Status](https://img.shields.io/crates/v/trycmd.svg)][Crates.io]

`trycmd` is a test harness that will enumerate test case files and run them to verify the
results, taking inspiration from
[trybuild](https://crates.io/crates/trybuild) and [cram](https://bitheap.org/cram/).

## Example

To create a minimal setup, create a `tests/cli_tests.rs` with
```rust,no_run
#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.toml")
        .case("README.md");
}
```
and write out your test cases in `.toml` files along with examples in your `README.md`.

Run this with `cargo test` like normal.  `TestCases` will enumerate all test case files and
run the contained commands, verifying they run as expected.

See the [docs](http://docs.rs/trycmd) for more.

## Users

- [typos](https://github.com/crate-ci/typos) (source code spell checker)
  - See [port from `assert_cmd`](https://github.com/crate-ci/typos/compare/a8ae8a5..cdfdc4084c928423211c6a80acbd24dbed7108f6)
- [cargo-edit](https://github.com/killercup/cargo-edit) (`Cargo.toml` editor)
- [clap](https://github.com/clap-rs/clap/) (CLI parser) to test examples

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms or
conditions.

[Crates.io]: https://crates.io/crates/trycmd
[Documentation]: https://docs.rs/trycmd
