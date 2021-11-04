use rayon::prelude::*;

#[derive(Debug)]
pub(crate) struct Runner {
    cases: Vec<Case>,
}

impl Runner {
    pub(crate) fn new() -> Self {
        Self {
            cases: Default::default(),
        }
    }

    pub(crate) fn case(&mut self, case: Case) {
        self.cases.push(case);
    }

    pub(crate) fn run(&self) {
        let palette = crate::Palette::current();
        if self.cases.is_empty() {
            eprintln!(
                "{}",
                palette
                    .warn
                    .paint("There are no trybuild tests enabled yet")
            );
        } else {
            let failures: Vec<_> = self
                .cases
                .par_iter()
                .filter_map(|c| {
                    if let Err(err) = c.run() {
                        eprintln!("{}", palette.error.paint(&err));
                        Some(err)
                    } else {
                        None
                    }
                })
                .collect();

            if 0 < failures.len() {
                panic!("{} of {} tests failed", failures.len(), self.cases.len());
            }
        }
    }
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
    pub(crate) default_bin: Option<crate::Bin>,
    pub(crate) error: Option<String>,
}

impl Case {
    pub(crate) fn error(path: std::path::PathBuf, error: impl std::fmt::Display) -> Self {
        let name = path.display().to_string();
        Self {
            name: name,
            path: path,
            expected: None,
            default_bin: None,
            error: Some(error.to_string()),
        }
    }

    pub(crate) fn run(&self) -> Result<(), String> {
        Ok(())
    }
}
