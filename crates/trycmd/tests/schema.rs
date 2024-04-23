#[test]
#[cfg(feature = "schema")]
fn dump_schema() {
    let bin_path = snapbox::cmd::cargo_bin!("trycmd-schema");
    snapbox::cmd::Command::new(bin_path)
        .assert()
        .success()
        .stdout_eq(snapbox::file!["../schema.json"]);
}
