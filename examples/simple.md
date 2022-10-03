With a trycmd case for your README file:

```rust,no_run
#[test]
fn readme() {
    // Paths are relative to the crate root
    trycmd::TestCases::new().case("README.md");
}
```

You can now write a test for it:

```console
$ simple World
Hello World!

$ simple Ferris
Hello Ferris!

$ simple
? 1
Must supply exactly one argument.

```

That's right, the file you're reading right now is a trycmd test!

It uses the trycmd format to run code blocks with the `console` or `trycmd` language:

~~~md
```console
$ command ...
```
~~~

> Note: since this demo code lives in `examples/simple.md`, the actual `simple` binary is
> in `examples/simple.rs` and the trycmd case lives in `tests/example_tests.rs`.
