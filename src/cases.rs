use std::borrow::Cow;

/// Entry point for running tests
#[derive(Debug, Default)]
pub struct TestCases {
    runner: std::cell::RefCell<crate::RunnerSpec>,
    bins: std::cell::RefCell<crate::BinRegistry>,
    substitutions: std::cell::RefCell<snapbox::Substitutions>,
    has_run: std::cell::Cell<bool>,
    file_loaders: std::cell::RefCell<crate::schema::TryCmdLoaders>,
}

impl TestCases {
    pub fn new() -> Self {
        let s = Self::default();
        s.runner
            .borrow_mut()
            .include(parse_include(std::env::args_os()));
        s
    }

    /// Load tests from `glob`
    pub fn case(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().case(glob.as_ref(), None);
        self
    }

    /// Overwrite expected status for a test
    pub fn pass(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::schema::CommandStatus::Success));
        self
    }

    /// Overwrite expected status for a test
    pub fn fail(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::schema::CommandStatus::Failed));
        self
    }

    /// Overwrite expected status for a test
    pub fn interrupted(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().case(
            glob.as_ref(),
            Some(crate::schema::CommandStatus::Interrupted),
        );
        self
    }

    /// Overwrite expected status for a test
    pub fn skip(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::schema::CommandStatus::Skipped));
        self
    }

    /// Set default bin, by path, for commands
    pub fn default_bin_path(&self, path: impl AsRef<std::path::Path>) -> &Self {
        let bin = Some(crate::schema::Bin::Path(path.as_ref().into()));
        self.runner.borrow_mut().default_bin(bin);
        self
    }

    /// Set default bin, by name, for commands
    pub fn default_bin_name(&self, name: impl AsRef<str>) -> &Self {
        let bin = Some(crate::schema::Bin::Name(name.as_ref().into()));
        self.runner.borrow_mut().default_bin(bin);
        self
    }

    /// Set default timeout for commands
    pub fn timeout(&self, time: std::time::Duration) -> &Self {
        self.runner.borrow_mut().timeout(Some(time));
        self
    }

    /// Set default environment variable
    pub fn env(&self, key: impl Into<String>, value: impl Into<String>) -> &Self {
        self.runner.borrow_mut().env(key, value);
        self
    }

    /// Add a bin to the "PATH" for cases to use
    pub fn register_bin(
        &self,
        name: impl Into<String>,
        path: impl Into<crate::schema::Bin>,
    ) -> &Self {
        self.bins
            .borrow_mut()
            .register_bin(name.into(), path.into());
        self
    }

    /// Add a series of bins to the "PATH" for cases to use
    pub fn register_bins<N: Into<String>, B: Into<crate::schema::Bin>>(
        &self,
        bins: impl IntoIterator<Item = (N, B)>,
    ) -> &Self {
        self.bins
            .borrow_mut()
            .register_bins(bins.into_iter().map(|(n, b)| (n.into(), b.into())));
        self
    }

    /// Define a function used to load a filesystem path into a test.
    ///
    /// `extension` is the file extension to register the loader for, without
    /// the leading dot. e.g. `toml`, `json`, or `trycmd`.
    ///
    /// By default there are loaders for `toml`, `trycmd`, and `md` extensions.
    /// Calling this function with those extensions will overwrite the default
    /// loaders.
    pub fn file_extension_loader(
        &self,
        extension: impl Into<std::ffi::OsString>,
        loader: crate::schema::TryCmdLoader,
    ) -> &Self {
        self.file_loaders
            .borrow_mut()
            .insert(extension.into(), loader);
        self
    }

    /// Add a variable for normalizing output
    ///
    /// Variable names must be
    /// - Surrounded by `[]`
    /// - Consist of uppercase letters
    ///
    /// Variables will be preserved through `TRYCMD=overwrite` / `TRYCMD=dump`.
    ///
    /// **NOTE:** We do basic search/replaces so new any new output will blindly be replaced.
    ///
    /// Reserved names:
    /// - `[..]`
    /// - `[EXE]`
    /// - `[CWD]`
    /// - `[ROOT]`
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// #[test]
    /// fn cli_tests() {
    ///     trycmd::TestCases::new()
    ///         .case("tests/cmd/*.trycmd")
    ///         .insert_var("[VAR]", "value");
    /// }
    /// ```
    pub fn insert_var(
        &self,
        var: &'static str,
        value: impl Into<Cow<'static, str>>,
    ) -> Result<&Self, crate::Error> {
        self.substitutions.borrow_mut().insert(var, value)?;
        Ok(self)
    }

    /// Batch add variables for normalizing output
    ///
    /// See `insert_var`.
    pub fn extend_vars(
        &self,
        vars: impl IntoIterator<Item = (&'static str, impl Into<Cow<'static, str>>)>,
    ) -> Result<&Self, crate::Error> {
        self.substitutions.borrow_mut().extend(vars)?;
        Ok(self)
    }

    /// Run tests
    ///
    /// This will happen on `drop` if not done explicitly
    pub fn run(&self) {
        self.has_run.set(true);

        let mode = parse_mode(std::env::var_os("TRYCMD").as_deref());
        mode.initialize().unwrap();

        let runner = self.runner.borrow_mut().prepare();
        runner.run(
            &self.file_loaders.borrow(),
            &mode,
            &self.bins.borrow(),
            &self.substitutions.borrow(),
        );
    }
}

impl std::panic::RefUnwindSafe for TestCases {}

#[doc(hidden)]
impl Drop for TestCases {
    fn drop(&mut self) {
        if !self.has_run.get() && !std::thread::panicking() {
            self.run();
        }
    }
}

// Filter which test cases are run by trybuild.
//
//     $ cargo test -- ui trybuild=tuple_structs.rs
//
// The first argument after `--` must be the trybuild test name i.e. the name of
// the function that has the #[test] attribute and calls trybuild. That's to get
// Cargo to run the test at all. The next argument starting with `trybuild=`
// provides a filename filter. Only test cases whose filename contains the
// filter string will be run.
#[allow(clippy::needless_collect)] // false positive https://github.com/rust-lang/rust-clippy/issues/5991
fn parse_include(args: impl IntoIterator<Item = std::ffi::OsString>) -> Option<Vec<String>> {
    let filters = args
        .into_iter()
        .flat_map(std::ffi::OsString::into_string)
        .filter_map(|arg| {
            const PREFIX: &str = "trycmd=";
            if let Some(remainder) = arg.strip_prefix(PREFIX) {
                if remainder.is_empty() {
                    None
                } else {
                    Some(remainder.to_owned())
                }
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    if filters.is_empty() {
        None
    } else {
        Some(filters)
    }
}

fn parse_mode(var: Option<&std::ffi::OsStr>) -> crate::Mode {
    if var == Some(std::ffi::OsStr::new("overwrite")) {
        crate::Mode::Overwrite
    } else if var == Some(std::ffi::OsStr::new("dump")) {
        crate::Mode::Dump("dump".into())
    } else {
        crate::Mode::Fail
    }
}
