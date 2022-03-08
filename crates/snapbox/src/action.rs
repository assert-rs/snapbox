pub const DEFAULT_ACTION_ENV: &str = "SNAPSHOTS";

/// Test action, see [`Assert`][crate::Assert]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl Action {
    pub fn with_env_var(var: impl AsRef<std::ffi::OsStr>) -> Option<Self> {
        let var = var.as_ref();
        let value = std::env::var_os(var)?;
        Self::with_env_value(value)
    }

    pub fn with_env_value(value: impl AsRef<std::ffi::OsStr>) -> Option<Self> {
        let value = value.as_ref();
        match value.to_str()? {
            "skip" => Some(Action::Skip),
            "ignore" => Some(Action::Ignore),
            "verify" => Some(Action::Verify),
            "overwrite" => Some(Action::Overwrite),
            _ => None,
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::Verify
    }
}
