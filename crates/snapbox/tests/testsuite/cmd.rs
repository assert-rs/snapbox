#[test]
fn regular_stdout_split() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("snap-fixture"))
        .env("echo_cwd", "1")
        .assert()
        .success();
}

#[test]
fn large_stdout_split() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("snap-fixture"))
        .env("echo_large", "1")
        .assert()
        .success();
}

#[test]
#[cfg(feature = "cmd")]
fn regular_stdout_single() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("snap-fixture"))
        .env("echo_cwd", "1")
        .stderr_to_stdout()
        .assert()
        .success();
}

#[test]
#[cfg(feature = "cmd")]
fn large_stdout_single() {
    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin!("snap-fixture"))
        .env("echo_large", "1")
        .stderr_to_stdout()
        .assert()
        .success();
}

#[test]
#[cfg(feature = "cmd")]
#[should_panic = "`CARGO_BIN_EXE_non-existent` is unset
help: available binary names are \"snap-fixture\""]
fn cargo_bin_non_existent() {
    let _ = snapbox::cmd::cargo_bin("non-existent");
}
