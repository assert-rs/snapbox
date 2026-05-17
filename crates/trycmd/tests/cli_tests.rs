#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    t.case("tests/cmd/*.trycmd").case("tests/cmd/*.toml");
    #[cfg(not(feature = "schema"))]
    {
        t.skip("tests/cmd/schema.toml");
    }
    #[cfg(not(feature = "filesystem"))]
    {
        t.skip("tests/cmd/diff_subset.toml");
        t.skip("tests/cmd/sandbox.toml");
        t.skip("tests/cmd/multistep.trycmd");
        t.skip("tests/cmd/normalize.toml");
        t.skip("tests/cmd/symlink.toml");
        t.skip("tests/cmd/subst-cwd.toml");
    }
    #[cfg(target_os = "windows")]
    {
        // On windows, crashes are returned as code=1
        t.skip("tests/cmd/timeout.toml");
    }
    t.extend_vars([("[EXAMPLE]", "example")]).unwrap();
    t.register_bin("ignored-bin", trycmd::schema::Bin::Ignore);
}

#[test]
fn default_current_dir() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cmd/default-current-dir.in");
    trycmd::TestCases::new()
        .case("tests/cmd/default-current-dir.toml")
        .default_current_dir(&fixture)
        .run();
}
