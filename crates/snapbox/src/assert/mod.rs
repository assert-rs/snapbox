mod action;
mod error;

#[cfg(feature = "color")]
use anstream::panic;
#[cfg(feature = "color")]
use anstream::stderr;
#[cfg(not(feature = "color"))]
use std::io::stderr;

use crate::filter::{Filter as _, FilterNewlines, FilterPaths, FilterRedactions};

pub use action::Action;
pub use action::DEFAULT_ACTION_ENV;
pub use error::Error;
pub use error::Result;

/// Snapshot assertion against a file's contents
///
/// Useful for one-off assertions with the snapshot stored in a file
///
/// # Examples
///
/// ```rust,no_run
/// # use snapbox::Assert;
/// # use snapbox::file;
/// let actual = "something";
/// Assert::new().matches(file!["output.txt"], actual);
/// ```
#[derive(Clone, Debug)]
pub struct Assert {
    pub(crate) action: Action,
    action_var: Option<String>,
    normalize_paths: bool,
    substitutions: crate::Redactions,
    pub(crate) palette: crate::report::Palette,
}

/// # Assertions
impl Assert {
    pub fn new() -> Self {
        Default::default()
    }

    /// Check if a value is the same as an expected value
    ///
    /// When the content is text, newlines are normalized.
    ///
    /// ```rust
    /// # use snapbox::Assert;
    /// let actual = "something";
    /// let expected = "something";
    /// Assert::new().eq(expected, actual);
    /// ```
    ///
    /// Can combine this with [`file!`][crate::file]
    /// ```rust,no_run
    /// # use snapbox::Assert;
    /// # use snapbox::file;
    /// let actual = "something";
    /// Assert::new().eq(file!["output.txt"], actual);
    /// ```
    #[track_caller]
    pub fn eq(&self, expected: impl Into<crate::Data>, actual: impl Into<crate::Data>) {
        let expected = expected.into();
        let actual = actual.into();
        if let Err(err) = self.try_eq(expected.raw(), actual, Some(&"In-memory")) {
            err.panic();
        }
    }

    /// Check if a value matches a pattern
    ///
    /// Pattern syntax:
    /// - `...` is a line-wildcard when on a line by itself
    /// - `[..]` is a character-wildcard when inside a line
    /// - `[EXE]` matches `.exe` on Windows
    ///
    /// Normalization:
    /// - Newlines
    /// - `\` to `/`
    ///
    /// ```rust
    /// # use snapbox::Assert;
    /// let actual = "something";
    /// let expected = "so[..]g";
    /// Assert::new().matches(expected, actual);
    /// ```
    ///
    /// Can combine this with [`file!`][crate::file]
    /// ```rust,no_run
    /// # use snapbox::Assert;
    /// # use snapbox::file;
    /// let actual = "something";
    /// Assert::new().matches(file!["output.txt"], actual);
    /// ```
    #[track_caller]
    pub fn matches(&self, pattern: impl Into<crate::Data>, actual: impl Into<crate::Data>) {
        let pattern = pattern.into();
        let actual = actual.into();
        if let Err(err) = self.try_eq(pattern, actual, Some(&"In-memory")) {
            err.panic();
        }
    }

    pub(crate) fn try_eq(
        &self,
        expected: crate::Data,
        actual: crate::Data,
        actual_name: Option<&dyn std::fmt::Display>,
    ) -> Result<()> {
        if expected.source().is_none() && actual.source().is_some() {
            panic!("received `(actual, expected)`, expected `(expected, actual)`");
        }
        match self.action {
            Action::Skip => {
                return Ok(());
            }
            Action::Ignore | Action::Verify | Action::Overwrite => {}
        }

        let (expected, actual) = self.normalize(expected, actual);

        self.do_action(expected, actual, actual_name)
    }

    pub(crate) fn normalize(
        &self,
        mut expected: crate::Data,
        mut actual: crate::Data,
    ) -> (crate::Data, crate::Data) {
        if expected.filters.is_newlines_set() {
            expected = FilterNewlines.filter(expected);
        }

        // On `expected` being an error, make a best guess
        let format = expected.intended_format();
        actual = actual.coerce_to(format);

        if self.normalize_paths && expected.filters.is_paths_set() {
            actual = FilterPaths.filter(actual);
        }
        if expected.filters.is_newlines_set() {
            actual = FilterNewlines.filter(actual);
        }
        if expected.filters.is_redaction_set() {
            actual = FilterRedactions::new(&self.substitutions, &expected).filter(actual);
        }

        (expected, actual)
    }

    pub(crate) fn do_action(
        &self,
        expected: crate::Data,
        actual: crate::Data,
        actual_name: Option<&dyn std::fmt::Display>,
    ) -> Result<()> {
        let result = self.try_verify(&expected, &actual, actual_name);
        let Err(err) = result else {
            return Ok(());
        };
        match self.action {
            Action::Skip => unreachable!("Bailed out earlier"),
            Action::Ignore => {
                use std::io::Write;

                let _ = writeln!(
                    stderr(),
                    "{}: {}",
                    self.palette.warn("Ignoring failure"),
                    err
                );
                Ok(())
            }
            Action::Verify => {
                let message = if expected.source().is_none() {
                    crate::report::Styled::new(String::new(), Default::default())
                } else if let Some(action_var) = self.action_var.as_deref() {
                    self.palette
                        .hint(format!("Update with {}=overwrite", action_var))
                } else {
                    crate::report::Styled::new(String::new(), Default::default())
                };
                Err(Error::new(format_args!("{err}{message}")))
            }
            Action::Overwrite => {
                use std::io::Write;

                if let Some(source) = expected.source() {
                    let _ = writeln!(stderr(), "{}: {}", self.palette.warn("Fixing"), err);
                    actual.write_to(source).unwrap();
                    Ok(())
                } else {
                    Err(Error::new(format_args!("{err}")))
                }
            }
        }
    }

    pub(crate) fn try_verify(
        &self,
        expected: &crate::Data,
        actual: &crate::Data,
        actual_name: Option<&dyn std::fmt::Display>,
    ) -> crate::assert::Result<()> {
        if expected != actual {
            let mut buf = String::new();
            crate::report::write_diff(
                &mut buf,
                expected,
                actual,
                expected.source().map(|s| s as &dyn std::fmt::Display),
                actual_name,
                self.palette,
            )
            .map_err(|e| e.to_string())?;
            Err(buf.into())
        } else {
            Ok(())
        }
    }
}

/// # Directory Assertions
#[cfg(feature = "dir")]
impl Assert {
    #[track_caller]
    pub fn subset_eq(
        &self,
        expected_root: impl Into<std::path::PathBuf>,
        actual_root: impl Into<std::path::PathBuf>,
    ) {
        let expected_root = expected_root.into();
        let actual_root = actual_root.into();
        self.subset_eq_inner(expected_root, actual_root)
    }

    #[track_caller]
    fn subset_eq_inner(&self, expected_root: std::path::PathBuf, actual_root: std::path::PathBuf) {
        match self.action {
            Action::Skip => {
                return;
            }
            Action::Ignore | Action::Verify | Action::Overwrite => {}
        }

        let checks: Vec<_> =
            crate::path::PathDiff::subset_eq_iter_inner(expected_root, actual_root).collect();
        self.verify(checks);
    }

    #[track_caller]
    pub fn subset_matches(
        &self,
        pattern_root: impl Into<std::path::PathBuf>,
        actual_root: impl Into<std::path::PathBuf>,
    ) {
        let pattern_root = pattern_root.into();
        let actual_root = actual_root.into();
        self.subset_matches_inner(pattern_root, actual_root)
    }

    #[track_caller]
    fn subset_matches_inner(
        &self,
        expected_root: std::path::PathBuf,
        actual_root: std::path::PathBuf,
    ) {
        match self.action {
            Action::Skip => {
                return;
            }
            Action::Ignore | Action::Verify | Action::Overwrite => {}
        }

        let checks: Vec<_> = crate::path::PathDiff::subset_matches_iter_inner(
            expected_root,
            actual_root,
            &self.substitutions,
            self.normalize_paths,
        )
        .collect();
        self.verify(checks);
    }

    #[track_caller]
    fn verify(
        &self,
        mut checks: Vec<Result<(std::path::PathBuf, std::path::PathBuf), crate::path::PathDiff>>,
    ) {
        if checks.iter().all(Result::is_ok) {
            for check in checks {
                let (_expected_path, _actual_path) = check.unwrap();
                crate::debug!(
                    "{}: is {}",
                    _expected_path.display(),
                    self.palette.info("good")
                );
            }
        } else {
            checks.sort_by_key(|c| match c {
                Ok((expected_path, _actual_path)) => Some(expected_path.clone()),
                Err(diff) => diff.expected_path().map(|p| p.to_owned()),
            });

            let mut buffer = String::new();
            let mut ok = true;
            for check in checks {
                use std::fmt::Write;
                match check {
                    Ok((expected_path, _actual_path)) => {
                        let _ = writeln!(
                            &mut buffer,
                            "{}: is {}",
                            expected_path.display(),
                            self.palette.info("good"),
                        );
                    }
                    Err(diff) => {
                        let _ = diff.write(&mut buffer, self.palette);
                        match self.action {
                            Action::Skip => unreachable!("Bailed out earlier"),
                            Action::Ignore | Action::Verify => {
                                ok = false;
                            }
                            Action::Overwrite => {
                                if let Err(err) = diff.overwrite() {
                                    ok = false;
                                    let path = diff
                                        .expected_path()
                                        .expect("always present when overwrite can fail");
                                    let _ = writeln!(
                                        &mut buffer,
                                        "{} to overwrite {}: {}",
                                        self.palette.error("Failed"),
                                        path.display(),
                                        err
                                    );
                                }
                            }
                        }
                    }
                }
            }
            if ok {
                use std::io::Write;
                let _ = write!(stderr(), "{}", buffer);
                match self.action {
                    Action::Skip => unreachable!("Bailed out earlier"),
                    Action::Ignore => {
                        let _ =
                            write!(stderr(), "{}", self.palette.warn("Ignoring above failures"));
                    }
                    Action::Verify => unreachable!("Something had to fail to get here"),
                    Action::Overwrite => {
                        let _ = write!(
                            stderr(),
                            "{}",
                            self.palette.warn("Overwrote above failures")
                        );
                    }
                }
            } else {
                match self.action {
                    Action::Skip => unreachable!("Bailed out earlier"),
                    Action::Ignore => unreachable!("Shouldn't be able to fail"),
                    Action::Verify => {
                        use std::fmt::Write;
                        if let Some(action_var) = self.action_var.as_deref() {
                            writeln!(
                                &mut buffer,
                                "{}",
                                self.palette
                                    .hint(format_args!("Update with {}=overwrite", action_var))
                            )
                            .unwrap();
                        }
                    }
                    Action::Overwrite => {}
                }
                panic!("{}", buffer);
            }
        }
    }
}

/// # Customize Behavior
impl Assert {
    /// Override the color palette
    pub fn palette(mut self, palette: crate::report::Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Read the failure action from an environment variable
    pub fn action_env(mut self, var_name: &str) -> Self {
        let action = Action::with_env_var(var_name);
        self.action = action.unwrap_or(self.action);
        self.action_var = Some(var_name.to_owned());
        self
    }

    /// Override the failure action
    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self.action_var = None;
        self
    }

    /// Override the default [`Redactions`][crate::Redactions]
    pub fn substitutions(mut self, substitutions: crate::Redactions) -> Self {
        self.substitutions = substitutions;
        self
    }

    /// Specify whether text should have path separators normalized
    ///
    /// The default is normalized
    pub fn normalize_paths(mut self, yes: bool) -> Self {
        self.normalize_paths = yes;
        self
    }
}

impl Default for Assert {
    fn default() -> Self {
        Self {
            action: Default::default(),
            action_var: Default::default(),
            normalize_paths: true,
            substitutions: Default::default(),
            palette: crate::report::Palette::color(),
        }
        .substitutions(crate::Redactions::with_exe())
    }
}
