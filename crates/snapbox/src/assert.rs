use crate::Action;

/// Snapshot assertion against a file's contents
pub fn file_assert() -> FileAssert {
    Default::default()
}

/// Snapshot assertion against a file's contents
///
/// See [`assert()`]
pub struct FileAssert {
    action: Action,
    substitutions: crate::Substitutions,
    palette: crate::report::Palette,
}

/// # Assertions
impl FileAssert {
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

    fn matches_inner(&self, actual: crate::Data, pattern_path: &std::path::Path) {
        match self.action {
            Action::Skip => {}
            Action::Ignore => match self.try_verify(&actual, pattern_path) {
                Ok(()) => {}
                Err(err) => {
                    use std::io::Write;

                    let _ = writeln!(
                        std::io::stderr(),
                        "{}: {}",
                        self.palette.warn("Ignoring failure"),
                        err
                    );
                }
            },
            Action::Verify => match self.try_verify(&actual, pattern_path) {
                Ok(()) => {}
                Err(err) => {
                    panic!("{}: {}", self.palette.error("Match failed"), err);
                }
            },
            Action::Overwrite => match self.try_verify(&actual, pattern_path) {
                Ok(()) => {}
                Err(err) => {
                    use std::io::Write;

                    let _ = writeln!(
                        std::io::stderr(),
                        "{}: {}",
                        self.palette.warn("Overwriting failed match"),
                        err
                    );
                    actual.write_to(pattern_path).unwrap();
                }
            },
        }
    }

    fn try_verify(
        &self,
        actual: &crate::Data,
        expected_path: &std::path::Path,
    ) -> crate::Result<()> {
        let expected = crate::Data::read_from(expected_path, Some(false))?
            .map_text(crate::utils::normalize_lines);

        if *actual != expected {
            let mut buf = String::new();
            crate::report::write_diff(
                &mut buf,
                &expected,
                &actual,
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
}

impl Default for FileAssert {
    fn default() -> Self {
        let mut substitutions = crate::Substitutions::new();
        substitutions
            .insert("[EXE]", std::env::consts::EXE_SUFFIX)
            .unwrap();
        Self {
            action: Action::Verify,
            substitutions,
            palette: crate::report::Palette::auto(),
        }
    }
}
