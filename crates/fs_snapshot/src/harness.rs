pub struct Harness<S, T> {
    root: std::path::PathBuf,
    overrides: Option<ignore::overrides::Override>,
    setup: S,
    test: T,
    overwrite: bool,
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
            overwrite: false,
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

    pub fn overwrite(mut self, yes: bool) -> Self {
        self.overwrite = yes;
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
                    if self.overwrite {
                        overwrite(&actual, &test.data)
                    } else {
                        verify(&actual, &test.data)
                    }
                }
                Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
            }
        })
        .exit()
    }
}

fn overwrite(actual: &str, case: &Case) -> libtest_mimic::Outcome {
    match try_overwrite(actual, case) {
        Ok(()) => libtest_mimic::Outcome::Passed,
        Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
    }
}

fn try_overwrite(actual: &str, case: &Case) -> Result<(), String> {
    std::fs::write(
        &case.expected,
        normalize_line_endings::normalized(actual.chars()).collect::<String>(),
    )
    .map_err(|e| format!("Failed to write to {}: {}", case.expected.display(), e))
}

fn verify(actual: &str, case: &Case) -> libtest_mimic::Outcome {
    match try_verify(actual, case) {
        Ok(()) => libtest_mimic::Outcome::Passed,
        Err(err) => libtest_mimic::Outcome::Failed { msg: Some(err) },
    }
}

fn try_verify(actual: &str, case: &Case) -> Result<(), String> {
    let palette = crate::color::Palette::current();
    let expected = std::fs::read_to_string(&case.expected)
        .map_err(|e| format!("Failed to read {}: {}", case.expected.display(), e))?;
    let expected: String = normalize_line_endings::normalized(expected.chars()).collect();

    let actual: String = normalize_line_endings::normalized(actual.chars()).collect();

    if actual != expected {
        #[cfg(feature = "diff")]
        {
            let diff = crate::diff::diff(
                &expected,
                &actual,
                case.expected.display(),
                case.expected.display(),
                palette,
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
                case.expected.display(),
                palette.info.paint("(expected)")
            )
            .map_err(|e| e.to_string())?;
            writeln!(buf, "{}", palette.info.paint(&expected)).map_err(|e| e.to_string())?;
            writeln!(
                buf,
                "{} {}:",
                case.expected.display(),
                palette.error.paint("(actual)")
            )
            .map_err(|e| e.to_string())?;
            writeln!(buf, "{}", palette.error.paint(&actual)).map_err(|e| e.to_string())?;

            Err(buf)
        }
    } else {
        Ok(())
    }
}

pub type Test = libtest_mimic::Test<Case>;

pub struct Case {
    pub fixture: std::path::PathBuf,
    pub expected: std::path::PathBuf,
}
