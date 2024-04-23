//! [`Harness`] for discovering test inputs and asserting against snapshot files
//!
//! This is a custom test harness and should be put in its own test binary with
//! [`test.harness = false`](https://doc.rust-lang.org/stable/cargo/reference/cargo-targets.html#the-harness-field).
//!
//! # Examples
//!
//! ```rust,no_run
//! fn some_func(num: usize) -> usize {
//!     // ...
//! #    10
//! }
//!
//! tryfn::Harness::new(
//!     "tests/fixtures/invalid",
//!     setup,
//!     test,
//! )
//! .select(["tests/cases/*.in"])
//! .test();
//!
//! fn setup(input_path: std::path::PathBuf) -> tryfn::Case {
//!     let name = input_path.file_name().unwrap().to_str().unwrap().to_owned();
//!     let expected = tryfn::Data::read_from(&input_path.with_extension("out"), None);
//!     tryfn::Case {
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
//!     let actual = some_func(num);
//!
//!     Ok(actual)
//! }
//! ```

use libtest_mimic::Trial;

pub use snapbox::assert::Action;
pub use snapbox::data::DataFormat;
pub use snapbox::Data;

/// [`Harness`] for discovering test inputs and asserting against snapshot files
pub struct Harness<S, T, I, E> {
    root: std::path::PathBuf,
    overrides: Option<ignore::overrides::Override>,
    setup: S,
    test: T,
    config: snapbox::Assert,
    test_output: std::marker::PhantomData<I>,
    test_error: std::marker::PhantomData<E>,
}

impl<S, T, I, E> Harness<S, T, I, E>
where
    S: Setup + Send + Sync + 'static,
    T: Test<I, E> + Clone + Send + Sync + 'static,
    I: std::fmt::Display,
    E: std::fmt::Display,
{
    /// Specify where the test scenarios
    ///
    /// - `input_root`: where to find the files.  See [`Self::select`] for restricting what files
    ///   are considered
    /// - `setup`: Given a path, choose the test name and the output location
    /// - `test`: Given a path, return the actual output value
    ///
    /// By default [`filters`][snapbox::filters] are applied, including:
    /// - `...` is a line-wildcard when on a line by itself
    /// - `[..]` is a character-wildcard when inside a line
    /// - `[EXE]` matches `.exe` on Windows
    /// - `\` to `/`
    /// - Newlines
    ///
    /// To limit this to newline normalization for text, have [`Setup`] call [`Data::raw`][snapbox::Data::raw] on `expected`.
    pub fn new(input_root: impl Into<std::path::PathBuf>, setup: S, test: T) -> Self {
        Self {
            root: input_root.into(),
            overrides: None,
            setup,
            test,
            config: snapbox::Assert::new().action_env(snapbox::assert::DEFAULT_ACTION_ENV),
            test_output: Default::default(),
            test_error: Default::default(),
        }
    }

    /// Path patterns for selecting input files
    ///
    /// This uses gitignore syntax
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
        self.config = self.config.action_env(var_name);
        self
    }

    /// Override the failure action
    pub fn action(mut self, action: Action) -> Self {
        self.config = self.config.action(action);
        self
    }

    /// Customize the assertion behavior
    pub fn with_assert(mut self, config: snapbox::Assert) -> Self {
        self.config = config;
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

        let shared_config = std::sync::Arc::new(self.config);
        let tests: Vec<_> = tests
            .into_iter()
            .map(|path| {
                let case = self.setup.setup(path);
                assert!(
                    case.expected.source().map(|s| s.is_path()).unwrap_or(false),
                    "`Case::expected` must be from a file"
                );
                let test = self.test.clone();
                let config = shared_config.clone();
                Trial::test(case.name.clone(), move || {
                    let actual = test.run(&case.fixture)?;
                    let actual = actual.to_string();
                    let actual = snapbox::Data::text(actual);
                    config.try_eq(case.expected.clone(), actual, Some(&case.name))?;
                    Ok(())
                })
                .with_ignored_flag(shared_config.selected_action() == Action::Ignore)
            })
            .collect();

        let args = libtest_mimic::Arguments::from_args();
        libtest_mimic::run(&args, tests).exit()
    }
}

/// Function signature for generating a test [`Case`] from a path fixture
pub trait Setup {
    fn setup(&self, fixture: std::path::PathBuf) -> Case;
}

impl<F> Setup for F
where
    F: Fn(std::path::PathBuf) -> Case,
{
    fn setup(&self, fixture: std::path::PathBuf) -> Case {
        (self)(fixture)
    }
}

/// Function signature for running a test [`Case`]
pub trait Test<S, E>
where
    S: std::fmt::Display,
    E: std::fmt::Display,
{
    fn run(&self, fixture: &std::path::Path) -> Result<S, E>;
}

impl<F, S, E> Test<S, E> for F
where
    F: Fn(&std::path::Path) -> Result<S, E>,
    S: std::fmt::Display,
    E: std::fmt::Display,
{
    fn run(&self, fixture: &std::path::Path) -> Result<S, E> {
        (self)(fixture)
    }
}

/// A test case enumerated by the [`Harness`] with data from the [`Setup`] function
pub struct Case {
    /// Display name
    pub name: String,
    /// Input for the test
    pub fixture: std::path::PathBuf,
    /// What the actual output should be compared against or updated
    ///
    /// Generally derived from `fixture` and loaded with [`Data::read_from`]
    pub expected: Data,
}
