#[test]
#[cfg(feature = "examples")]
fn example_tests() {
    let t = trycmd::TestCases::new();
    t.register_bin(
        "example-fixture",
        trycmd::cargo::compile_example("example-fixture", []),
    );
    t.case("examples/cmd/*.trycmd").case("examples/cmd/*.toml");
}
