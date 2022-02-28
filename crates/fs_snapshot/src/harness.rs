pub struct Harness<S, T> {
    root: std::path::PathBuf,
    overrides: Option<ignore::overrides::Override>,
    setup: S,
    test: T,
    action: crate::Action,
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
            action: crate::Action::Verify,
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
        let var_value = std::env::var_os(var_name);
        self.action = match var_value.as_ref().and_then(|s| s.to_str()) {
            Some("skip") => crate::Action::Skip,
            Some("ignore") => crate::Action::Ignore,
            Some("verify") => crate::Action::Verify,
            Some("overwrite") => crate::Action::Overwrite,
            Some(_) | None => self.action,
        };
        self
    }

    pub fn action(mut self, action: crate::Action) -> Self {
        self.action = action;
        self
    }

    /// Deprecated, replaced with [`Harness::action`]
    #[deprecated = "Replaced with `Harness::action`"]
    pub fn overwrite(mut self, yes: bool) -> Self {
        if yes {
            self.action = crate::Action::Overwrite;
        } else {
            self.action = crate::Action::Verify;
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
                    let verify = Verifier::new()
                        .palette(crate::color::Palette::current())
                        .action(self.action);
                    verify.verify(&actual, &test.data.expected)
                }
                Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
            }
        })
        .exit()
    }
}

struct Verifier {
    palette: crate::color::Palette,
    action: crate::Action,
}

impl Verifier {
    fn new() -> Self {
        Default::default()
    }

    fn palette(mut self, palette: crate::color::Palette) -> Self {
        self.palette = palette;
        self
    }

    fn action(mut self, action: crate::Action) -> Self {
        self.action = action;
        self
    }

    fn verify(&self, actual: &str, expected_path: &std::path::Path) -> libtest_mimic::Outcome {
        match self.action {
            crate::Action::Skip => libtest_mimic::Outcome::Ignored,
            crate::Action::Ignore => {
                let _ = self.do_verify(&actual, expected_path);
                libtest_mimic::Outcome::Ignored
            }
            crate::Action::Verify => self.do_verify(&actual, expected_path),
            crate::Action::Overwrite => self.do_overwrite(&actual, expected_path),
        }
    }

    fn do_overwrite(
        &self,
        actual: &str,
        expected_path: &std::path::Path,
    ) -> libtest_mimic::Outcome {
        match self.try_overwrite(actual, expected_path) {
            Ok(()) => libtest_mimic::Outcome::Passed,
            Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
        }
    }

    fn try_overwrite(&self, actual: &str, expected_path: &std::path::Path) -> Result<(), String> {
        std::fs::write(
            expected_path,
            normalize_line_endings::normalized(actual.chars()).collect::<String>(),
        )
        .map_err(|e| format!("Failed to write to {}: {}", expected_path.display(), e))
    }

    fn do_verify(&self, actual: &str, expected_path: &std::path::Path) -> libtest_mimic::Outcome {
        match self.try_verify(actual, expected_path) {
            Ok(()) => libtest_mimic::Outcome::Passed,
            Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
        }
    }

    fn try_verify(&self, actual: &str, expected_path: &std::path::Path) -> Result<(), String> {
        let expected = std::fs::read_to_string(expected_path)
            .map_err(|e| format!("Failed to read {}: {}", expected_path.display(), e))?;
        let expected: String = normalize_line_endings::normalized(expected.chars()).collect();

        let actual: String = normalize_line_endings::normalized(actual.chars()).collect();

        if actual != expected {
            #[cfg(feature = "diff")]
            {
                let diff = crate::utils::render_diff(
                    &expected,
                    &actual,
                    expected_path.display(),
                    expected_path.display(),
                    self.palette,
                );
                Err(diff)
            }
            #[cfg(not(feature = "diff"))]
            {
                use std::fmt::Write;

                let mut buf = String::new();
                writeln!(
                    buf,
                    "{} {}:",
                    expected_path.display(),
                    self.palette.info.paint("(expected)")
                )
                .map_err(|e| e.to_string())?;
                writeln!(buf, "{}", self.palette.info.paint(&expected))
                    .map_err(|e| e.to_string())?;
                writeln!(
                    buf,
                    "{} {}:",
                    expected_path.display(),
                    self.palette.error.paint("(actual)")
                )
                .map_err(|e| e.to_string())?;
                writeln!(buf, "{}", self.palette.error.paint(&actual))
                    .map_err(|e| e.to_string())?;

                Err(buf)
            }
        } else {
            Ok(())
        }
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self {
            palette: crate::color::Palette::current(),
            action: crate::Action::Verify,
        }
    }
}

pub type Test = libtest_mimic::Test<Case>;

pub struct Case {
    pub fixture: std::path::PathBuf,
    pub expected: std::path::PathBuf,
}
