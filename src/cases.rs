#[derive(Debug, Default)]
pub struct TestCases {
    runner: std::cell::RefCell<crate::RunnerSpec>,
    has_run: std::cell::Cell<bool>,
}

impl TestCases {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn default_bin_path(&self, path: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().default_bin_path(path.as_ref());
        self
    }

    pub fn default_bin_name(&self, path: impl AsRef<str>) -> &Self {
        self.runner.borrow_mut().default_bin_name(path.as_ref());
        self
    }

    pub fn case(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().case(glob.as_ref());
        self
    }

    pub fn pass(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().pass(glob.as_ref());
        self
    }

    pub fn fail(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().fail(glob.as_ref());
        self
    }

    pub fn run(&self) {
        self.has_run.set(true);
        self.runner.borrow_mut().prepare().run();
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
