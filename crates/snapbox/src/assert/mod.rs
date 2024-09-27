mod action;
mod error;

#[cfg(feature = "color")]
use anstream::panic;
#[cfg(feature = "color")]
use anstream::stderr;
#[cfg(not(feature = "color"))]
use std::io::stderr;

use crate::filter::{Filter as _, FilterNewlines, FilterPaths, NormalizeToExpected};
use crate::IntoData;

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
/// Assert::new().eq(actual, file!["output.txt"]);
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
    /// By default [`filters`][crate::filter] are applied, including:
    /// - `...` is a line-wildcard when on a line by itself
    /// - `[..]` is a character-wildcard when inside a line
    /// - `[EXE]` matches `.exe` on Windows
    /// - `"{...}"` is a JSON value wildcard
    /// - `"...": "{...}"` is a JSON key-value wildcard
    /// - `\` to `/`
    /// - Newlines
    ///
    /// To limit this to newline normalization for text, call [`Data::raw`][crate::Data::raw] on `expected`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use snapbox::Assert;
    /// let actual = "something";
    /// let expected = "so[..]g";
    /// Assert::new().eq(actual, expected);
    /// ```
    ///
    /// Can combine this with [`file!`][crate::file]
    /// ```rust,no_run
    /// # use snapbox::Assert;
    /// # use snapbox::file;
    /// let actual = "something";
    /// Assert::new().eq(actual, file!["output.txt"]);
    /// ```
    #[track_caller]
    pub fn eq(&self, actual: impl IntoData, expected: impl IntoData) {
        let expected = expected.into_data();
        let actual = actual.into_data();
        if let Err(err) = self.try_eq(Some(&"In-memory"), actual, expected) {
            err.panic();
        }
    }

    #[track_caller]
    #[deprecated(since = "0.6.0", note = "Replaced with `Assert::eq`")]
    pub fn eq_(&self, actual: impl IntoData, expected: impl IntoData) {
        self.eq(actual, expected);
    }

    pub fn try_eq(
        &self,
        actual_name: Option<&dyn std::fmt::Display>,
        actual: crate::Data,
        expected: crate::Data,
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

        let (actual, expected) = self.normalize(actual, expected);

        self.do_action(actual_name, actual, expected)
    }

    pub fn normalize(
        &self,
        mut actual: crate::Data,
        mut expected: crate::Data,
    ) -> (crate::Data, crate::Data) {
        if expected.filters.is_newlines_set() {
            expected = FilterNewlines.filter(expected);
        }

        // On `expected` being an error, make a best guess
        actual = actual.coerce_to(expected.against_format());
        actual = actual.coerce_to(expected.intended_format());

        if self.normalize_paths && expected.filters.is_paths_set() {
            actual = FilterPaths.filter(actual);
        }
        if expected.filters.is_newlines_set() {
            actual = FilterNewlines.filter(actual);
        }

        let mut normalize = NormalizeToExpected::new();
        if expected.filters.is_redaction_set() {
            normalize = normalize.redact_with(&self.substitutions);
        }
        if expected.filters.is_unordered_set() {
            normalize = normalize.unordered();
        }
        actual = normalize.normalize(actual, &expected);

        (actual, expected)
    }

    fn do_action(
        &self,
        actual_name: Option<&dyn std::fmt::Display>,
        actual: crate::Data,
        expected: crate::Data,
    ) -> Result<()> {
        let result = self.try_verify(actual_name, &actual, &expected);
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
                        .hint(format!("Update with {action_var}=overwrite"))
                } else {
                    crate::report::Styled::new(String::new(), Default::default())
                };
                Err(Error::new(format_args!("{err}{message}")))
            }
            Action::Overwrite => {
                use std::io::Write;

                if let Some(source) = expected.source() {
                    if let Err(message) = actual.write_to(source) {
                        Err(Error::new(format_args!("{err}Update failed: {message}")))
                    } else {
                        let _ = writeln!(stderr(), "{}: {}", self.palette.warn("Fixing"), err);
                        Ok(())
                    }
                } else {
                    Err(Error::new(format_args!("{err}")))
                }
            }
        }
    }

    fn try_verify(
        &self,
        actual_name: Option<&dyn std::fmt::Display>,
        actual: &crate::Data,
        expected: &crate::Data,
    ) -> Result<()> {
        if actual != expected {
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
        self.subset_eq_inner(expected_root, actual_root);
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
            crate::dir::PathDiff::subset_eq_iter_inner(expected_root, actual_root).collect();
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
        self.subset_matches_inner(pattern_root, actual_root);
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

        let checks: Vec<_> = crate::dir::PathDiff::subset_matches_iter_inner(
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
        mut checks: Vec<Result<(std::path::PathBuf, std::path::PathBuf), crate::dir::PathDiff>>,
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
                let _ = write!(stderr(), "{buffer}");
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
                                    .hint(format_args!("Update with {action_var}=overwrite"))
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
    pub fn redact_with(mut self, substitutions: crate::Redactions) -> Self {
        self.substitutions = substitutions;
        self
    }

    /// Override the default [`Redactions`][crate::Redactions]
    #[deprecated(since = "0.6.2", note = "Replaced with `Assert::redact_with`")]
    pub fn substitutions(self, substitutions: crate::Redactions) -> Self {
        self.redact_with(substitutions)
    }

    /// Specify whether text should have path separators normalized
    ///
    /// The default is normalized
    pub fn normalize_paths(mut self, yes: bool) -> Self {
        self.normalize_paths = yes;
        self
    }
}

impl Assert {
    pub fn selected_action(&self) -> Action {
        self.action
    }

    pub fn redactions(&self) -> &crate::Redactions {
        &self.substitutions
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
        .redact_with(crate::Redactions::with_exe())
    }
}
