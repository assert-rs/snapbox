#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.trycmd").case("tests/cmd/*.toml");
    // On windows, crashes are returned as code=1
    #[cfg(target_os = "windows")]
    t.skip("tests/cmd/timeout.toml");
}
