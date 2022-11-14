#[test]
fn trycmd() {
    trycmd::TestCases::new()
        .case("README.md")
        .insert_var("[REPLACEMENT]", "runtime-value")
        .unwrap();
}
