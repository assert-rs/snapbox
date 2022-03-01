/// Test action
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

pub struct Harness<S, T> {
    root: std::path::PathBuf,
    overrides: Option<ignore::overrides::Override>,
    setup: S,
    test: T,
    action: Action,
}

impl<S, T> Harness<S, T>
where
    S: Fn(std::path::PathBuf) -> Test + Send + Sync + 'static,
    T: Fn(&std::path::Path) -> Result<String, String> + Send + Sync + 'static,
{
    pub fn new(root: impl Into<std::path::PathBuf>, setup: S, test: T) -> Self {
        Self {
            root: root.into(),
            overrides: None,
            setup,
            test,
            action: Action::Verify,
        }
    }

    pub fn select<'p>(mut self, patterns: impl IntoIterator<Item = &'p str>) -> Self {
        let mut overrides = ignore::overrides::OverrideBuilder::new(&self.root);
        for line in patterns {
            overrides.add(line).unwrap();
        }
        self.overrides = Some(overrides.build().unwrap());
        self
    }

    pub fn action_env(mut self, var_name: &str) -> Self {
        let action = Action::with_env_var(var_name);
        self.action = action.unwrap_or(self.action);
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    /// Deprecated, replaced with [`Harness::action`]
    #[deprecated = "Replaced with `Harness::action`"]
    pub fn overwrite(mut self, yes: bool) -> Self {
        if yes {
            self.action = Action::Overwrite;
        } else {
            self.action = Action::Verify;
        }
        self
    }

    pub fn test(self) -> ! {
        let mut walk = ignore::WalkBuilder::new(&self.root);
        walk.standard_filters(false);
        let tests = walk.build().filter_map(|entry| {
            let entry = entry.unwrap();
            let is_dir = entry.file_type().map(|f| f.is_dir()).unwrap_or(false);
            let path = entry.into_path();
            if let Some(overrides) = &self.overrides {
                overrides
                    .matched(&path, is_dir)
                    .is_whitelist()
                    .then(|| path)
            } else {
                Some(path)
            }
        });

        let tests: Vec<_> = tests.into_iter().map(|path| (self.setup)(path)).collect();

        let args = libtest_mimic::Arguments::from_args();
        libtest_mimic::run_tests(&args, tests, move |test| {
            match (self.test)(&test.data.fixture) {
                Ok(actual) => {
                    let actual = crate::Data::Text(actual).map_text(crate::utils::normalize_lines);
                    let verify = Verifier::new()
                        .palette(crate::report::Palette::auto())
                        .action(self.action);
                    verify.verify(actual, &test.data.expected)
                }
                Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
            }
        })
        .exit()
    }
}

struct Verifier {
    palette: crate::report::Palette,
    action: Action,
}

impl Verifier {
    fn new() -> Self {
        Default::default()
    }

    fn palette(mut self, palette: crate::report::Palette) -> Self {
        self.palette = palette;
        self
    }

    fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    fn verify(
        &self,
        actual: crate::Data,
        expected_path: &std::path::Path,
    ) -> libtest_mimic::Outcome {
        match self.action {
            Action::Skip => libtest_mimic::Outcome::Ignored,
            Action::Ignore => {
                let _ = self.do_verify(actual, expected_path);
                libtest_mimic::Outcome::Ignored
            }
            Action::Verify => self.do_verify(actual, expected_path),
            Action::Overwrite => self.do_overwrite(actual, expected_path),
        }
    }

    fn do_overwrite(
        &self,
        actual: crate::Data,
        expected_path: &std::path::Path,
    ) -> libtest_mimic::Outcome {
        match self.try_overwrite(actual, expected_path) {
            Ok(()) => libtest_mimic::Outcome::Passed,
            Err(err) => libtest_mimic::Outcome::Failed {
                msg: Some(err.to_string()),
            },
        }
    }

    fn try_overwrite(
        &self,
        actual: crate::Data,
        expected_path: &std::path::Path,
    ) -> crate::Result<()> {
        actual.write_to(expected_path)?;
        Ok(())
    }

    fn do_verify(
        &self,
        actual: crate::Data,
        expected_path: &std::path::Path,
    ) -> libtest_mimic::Outcome {
        match self.try_verify(actual, expected_path) {
            Ok(()) => libtest_mimic::Outcome::Passed,
            Err(err) => libtest_mimic::Outcome::Failed {
                msg: Some(err.to_string()),
            },
        }
    }

    fn try_verify(
        &self,
        actual: crate::Data,
        expected_path: &std::path::Path,
    ) -> crate::Result<()> {
        let expected = crate::Data::read_from(expected_path, Some(false))?
            .map_text(crate::utils::normalize_lines);

        if actual != expected {
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

impl Default for Verifier {
    fn default() -> Self {
        Self {
            palette: crate::report::Palette::auto(),
            action: Action::Verify,
        }
    }
}

pub type Test = libtest_mimic::Test<Case>;

pub struct Case {
    pub fixture: std::path::PathBuf,
    pub expected: std::path::PathBuf,
}
