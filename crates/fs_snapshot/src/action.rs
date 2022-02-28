/// Test action
pub enum Action {
    /// Do not run the test
    Skip,
    /// Ignore test failures
    Ignore,
    /// Fail on mismatch
    Verify,
    /// Overwrite on mismatch
    Overwrite,
}
