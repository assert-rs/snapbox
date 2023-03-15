//! [`Harness`] for discovering test inputs and asserting against snapshot files
//!
//! # Examples
//!
//! ```rust,no_run
//! snapbox::harness::Harness::new(
//!     "tests/fixtures/invalid",
//!     setup,
//!     test,
//! )
//! .select(["tests/cases/*.in"])
//! .action_env("SNAPSHOTS")
//! .test();
//!
//! fn setup(input_path: std::path::PathBuf) -> snapbox::harness::Case {
//!     let name = input_path.file_name().unwrap().to_str().unwrap().to_owned();
//!     let expected = input_path.with_extension("out");
//!     snapbox::harness::Case {
//!         name,
//!         fixture: input_path,
//!         expected,
//!     }
//! }
//!
//! fn test(input_path: &std::path::Path) -> Result<usize, Box<dyn std::error::Error>> {
//!     let raw = std::fs::read_to_string(input_path)?;
//!     let num = raw.parse::<usize>()?;
//!
//!     let actual = num + 10;
//!
//!     Ok(actual)
//! }
//! ```

use crate::data::{DataFormat, NormalizeNewlines};
use crate::Action;

use libtest_mimic::Trial;

pub struct Harness<S, T> {
    root: std::path::PathBuf,
    overrides: Option<ignore::overrides::Override>,
    setup: S,
    test: T,
    action: Action,
}

impl<S, T, I, E> Harness<S, T>
where
    I: std::fmt::Display,
    E: std::fmt::Display,
    S: Fn(std::path::PathBuf) -> Case + Send + Sync + 'static,
    T: Fn(&std::path::Path) -> Result<I, E> + Send + Sync + 'static + Clone,
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

    /// Path patterns for selecting input files
    ///
    /// This used gitignore syntax
    pub fn select<'p>(mut self, patterns: impl IntoIterator<Item = &'p str>) -> Self {
        let mut overrides = ignore::overrides::OverrideBuilder::new(&self.root);
        for line in patterns {
            overrides.add(line).unwrap();
        }
        self.overrides = Some(overrides.build().unwrap());
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

    /// Run tests
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
                    .then_some(path)
            } else {
                Some(path)
            }
        });

        let tests: Vec<_> = tests
            .into_iter()
            .map(|path| {
                let case = (self.setup)(path);
                let test = self.test.clone();
                Trial::test(case.name.clone(), move || {
                    let actual = (test)(&case.fixture)?;
                    let actual = actual.to_string();
                    let actual = crate::Data::text(actual).normalize(NormalizeNewlines);
                    #[allow(deprecated)]
                    let verify = Verifier::new()
                        .palette(crate::report::Palette::auto())
                        .action(self.action);
                    verify.verify(&case.expected, actual)?;
                    Ok(())
                })
                .with_ignored_flag(self.action == Action::Ignore)
            })
            .collect();

        let args = libtest_mimic::Arguments::from_args();
        libtest_mimic::run(&args, tests).exit()
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

    fn verify(&self, expected_path: &std::path::Path, actual: crate::Data) -> crate::Result<()> {
        match self.action {
            Action::Skip => Ok(()),
            Action::Ignore => {
                let _ = self.try_verify(expected_path, actual);
                Ok(())
            }
            Action::Verify => self.try_verify(expected_path, actual),
            Action::Overwrite => self.try_overwrite(expected_path, actual),
        }
    }

    fn try_overwrite(
        &self,
        expected_path: &std::path::Path,
        actual: crate::Data,
    ) -> crate::Result<()> {
        actual.write_to(expected_path)?;
        Ok(())
    }

    fn try_verify(
        &self,
        expected_path: &std::path::Path,
        actual: crate::Data,
    ) -> crate::Result<()> {
        let expected = crate::Data::read_from(expected_path, Some(DataFormat::Text))?
            .normalize(NormalizeNewlines);

        if expected != actual {
            let mut buf = String::new();
            crate::report::write_diff(
                &mut buf,
                &expected,
                &actual,
                Some(&expected_path.display()),
                None,
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
            #[allow(deprecated)]
            palette: crate::report::Palette::auto(),
            action: Action::Verify,
        }
    }
}

pub struct Case {
    pub name: String,
    pub fixture: std::path::PathBuf,
    pub expected: std::path::PathBuf,
}
