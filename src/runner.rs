#[derive(Debug)]
pub(crate) struct Runner {
    default_bin: Option<crate::Bin>,
    cases: Vec<Case>,
}

impl Runner {
    pub(crate) fn new() -> Self {
        Self {
            default_bin: None,
            cases: Default::default(),
        }
    }

    pub(crate) fn default_bin(&mut self, bin: Option<crate::Bin>) {
        self.default_bin = bin;
    }

    pub(crate) fn case(&mut self, case: Case) {
        self.cases.push(case);
    }

    pub(crate) fn run(&self) {}
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) struct Case {
    pub(crate) name: String,
    pub(crate) path: std::path::PathBuf,
    pub(crate) expected: Option<crate::Expected>,
    pub(crate) error: Option<String>,
}

impl Case {
    pub(crate) fn error(path: std::path::PathBuf, error: impl std::fmt::Display) -> Self {
        let name = path.display().to_string();
        Self {
            name: name,
            path: path,
            expected: None,
            error: Some(error.to_string()),
        }
    }
}
