# Trycmd demo

This crate demos the `.trycmd` file format. In fact, the file you're reading right now is a trycmd
test, registered in `tests/trycmd.rs`.

Let's test our simple hello world binary (found in `src/main.rs`):

```console
$ simple World
Hello World!

$ simple Ferris
Hello Ferris!

```

The format looks for code blocks with the `console` or `trycmd` language:

~~~md
```console
$ command ...
```
~~~

You can also test for command failures and pass in environment variables:

```console
$ simple
? 1
Must supply exactly one argument.

$ GOODBYE=true simple World
Goodbye World!

```

Sometimes, your test might include output that is generated at runtime. When that's the case, you
can
use variables to replace those values. In our `tests/trycmd.rs`, we've defined a
variable `[REPLACEMENT]` such that whenever the value `runtime-value` appears, it will be
replaced with `[REPLACEMENT]`:

```console
$ simple "blah blah runtime-value blah"
Hello blah blah [REPLACEMENT] blah!

$ simple "blah blah runtime-value blah"
Hello blah blah runtime-value blah!

```

Note that the tests can still contain `runtime-value`: using `[REPLACEMENT]` is purely for
your convenience.
