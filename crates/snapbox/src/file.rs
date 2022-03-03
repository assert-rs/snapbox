use crate::Action;

/// Snapshot assertion against a file's contents
///
/// Useful for one-off assertions with the snapshot stored in a file
///
/// # Examples
///
/// ```rust,no_run
/// let actual = "...";
/// snapbox::file_assert()
///     .action_env("SNAPSHOT_ACTION")
///     .matches(actual, "tests/fixtures/help_output_is_clean.txt");
/// ```
pub fn file_assert() -> FileAssert {
    Default::default()
}

/// Snapshot assertion against a file's contents
///
/// See [`file_assert()`]
pub struct FileAssert {
    action: Action,
    substitutions: crate::Substitutions,
    palette: crate::report::Palette,
    binary: Option<bool>,
}

/// # Assertions
impl FileAssert {
    /// Check if a value matches the content of a file
    ///
    /// When the content is text, newlines are normalized.
    #[track_caller]
    pub fn eq(&self, actual: impl Into<crate::Data>, pattern_path: impl AsRef<std::path::Path>) {
        let actual = actual.into();
        let pattern_path = pattern_path.as_ref();
        self.eq_inner(actual, pattern_path);
    }

    #[track_caller]
    fn eq_inner(&self, mut actual: crate::Data, pattern_path: &std::path::Path) {
        match self.action {
            Action::Skip => {
                return;
            }
            Action::Ignore | Action::Verify | Action::Overwrite => {}
        }

        let expected = crate::Data::read_from(pattern_path, self.binary)
            .map(|d| d.map_text(crate::utils::normalize_lines));
        if expected.as_ref().ok().and_then(|d| d.as_str()).is_some() {
            actual = actual.try_text().map_text(crate::utils::normalize_lines);
        }

        let result = expected.and_then(|e| self.try_verify(&actual, &e, pattern_path));
        if let Err(err) = result {
            match self.action {
                Action::Skip => unreachable!("Bailed out earlier"),
                Action::Ignore => {
                    use std::io::Write;

                    let _ = writeln!(
                        std::io::stderr(),
                        "{}: {}",
                        self.palette.warn("Ignoring eq failure"),
                        err
                    );
                }
                Action::Verify => {
                    panic!("{}: {}", self.palette.error("Not eq"), err);
                }
                Action::Overwrite => {
                    use std::io::Write;

                    let _ = writeln!(
                        std::io::stderr(),
                        "{}: {}",
                        self.palette.warn("Overwriting failed eq check"),
                        err
                    );
                    actual.write_to(pattern_path).unwrap();
                }
            }
        }
    }

    /// Check if a value matches the pattern in a file
    ///
    /// Pattern syntax:
    /// - `...` is a line-wildcard when on a line by itself
    /// - `[..]` is a character-wildcard when inside a line
    /// - `[EXE]` matches `.exe` on Windows (override with [`FileAssert::substitutions`])
    ///
    /// Normalization:
    /// - Newlines
    /// - `\` to `/`
    #[track_caller]
    pub fn matches(
        &self,
        actual: impl Into<crate::Data>,
        pattern_path: impl AsRef<std::path::Path>,
    ) {
        let actual = actual.into();
        let pattern_path = pattern_path.as_ref();
        self.matches_inner(actual, pattern_path);
    }

    #[track_caller]
    fn matches_inner(&self, mut actual: crate::Data, pattern_path: &std::path::Path) {
        match self.action {
            Action::Skip => {
                return;
            }
            Action::Ignore | Action::Verify | Action::Overwrite => {}
        }

        let expected = crate::Data::read_from(pattern_path, self.binary)
            .map(|d| d.map_text(crate::utils::normalize_lines));
        if let Some(expected) = expected.as_ref().ok().and_then(|d| d.as_str()) {
            actual = actual
                .try_text()
                .map_text(crate::utils::normalize_text)
                .map_text(|t| self.substitutions.normalize(t, expected));
        }

        let result = expected.and_then(|e| self.try_verify(&actual, &e, pattern_path));
        if let Err(err) = result {
            match self.action {
                Action::Skip => unreachable!("Bailed out earlier"),
                Action::Ignore => {
                    use std::io::Write;

                    let _ = writeln!(
                        std::io::stderr(),
                        "{}: {}",
                        self.palette.warn("Ignoring match failure"),
                        err
                    );
                }
                Action::Verify => {
                    panic!("{}: {}", self.palette.error("Match failed"), err);
                }
                Action::Overwrite => {
                    use std::io::Write;

                    let _ = writeln!(
                        std::io::stderr(),
                        "{}: {}",
                        self.palette.warn("Overwriting failed match"),
                        err
                    );
                    actual.write_to(pattern_path).unwrap();
                }
            }
        }
    }

    fn try_verify(
        &self,
        actual: &crate::Data,
        expected: &crate::Data,
        expected_path: &std::path::Path,
    ) -> crate::Result<()> {
        if actual != expected {
            let mut buf = String::new();
            crate::report::write_diff(
                &mut buf,
                expected,
                actual,
                &expected_path.display(),
                &expected_path.display(),
                self.palette,
            )
            .map_err(|e| e.to_string())?;
            Err(buf.into())
        } else {
            Ok(())
        }
    }
}

/// # Customize Behavior
impl FileAssert {
    /// Override the color palette
    pub fn palette(mut self, palette: crate::report::Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Read the failure action from an environment variable
    pub fn action_env(mut self, var_name: &str) -> Self {
        let action = Action::with_env_var(var_name);
        self.action = action.unwrap_or(self.action);
        self
    }

    /// Override the failure action
    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    /// Override the default [`Substitutions`][crate::Substitutions]
    pub fn substitutions(mut self, substitutions: crate::Substitutions) -> Self {
        self.substitutions = substitutions;
        self
    }

    /// Specify whether the content should be treated as binary or not
    ///
    /// The default is to auto-detect
    pub fn binary(mut self, yes: bool) -> Self {
        self.binary = Some(yes);
        self
    }
}

impl Default for FileAssert {
    fn default() -> Self {
        Self {
            action: Action::Verify,
            substitutions: crate::Substitutions::with_exe(),
            palette: crate::report::Palette::auto(),
            binary: None,
        }
    }
}
