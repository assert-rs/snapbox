#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.trycmd")
        .case("tests/cmd/*.toml");
}
