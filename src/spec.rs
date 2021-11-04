use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct RunnerSpec {
    default_bin: Option<crate::Bin>,
    cases: Vec<CaseSpec>,
    include: Option<Vec<String>>,
}

impl RunnerSpec {
    pub(crate) fn new() -> Self {
        Self {
            default_bin: None,
            cases: Default::default(),
            include: None,
        }
    }

    pub(crate) fn default_bin(&mut self, bin: Option<crate::Bin>) {
        self.default_bin = bin;
    }

    pub(crate) fn include(&mut self, include: Option<Vec<String>>) {
        self.include = include;
    }

    pub(crate) fn case(&mut self, glob: &std::path::Path) {
        self.cases.push(CaseSpec {
            glob: glob.into(),
            expected: None,
        });
    }

    pub(crate) fn pass(&mut self, glob: &std::path::Path) {
        self.cases.push(CaseSpec {
            glob: glob.into(),
            expected: Some(crate::Expected::Pass),
        });
    }

    pub(crate) fn fail(&mut self, glob: &std::path::Path) {
        self.cases.push(CaseSpec {
            glob: glob.into(),
            expected: Some(crate::Expected::Fail),
        });
    }

    pub(crate) fn prepare(&mut self) -> crate::Runner {
        let mut runner = crate::Runner::new();
        runner.default_bin(self.default_bin.clone());

        // Both sort and let the last writer win to allow overriding specific cases within a glob
        let mut cases: BTreeMap<std::path::PathBuf, crate::Case> = BTreeMap::new();

        for spec in &self.cases {
            if let Some(glob) = get_glob(&spec.glob) {
                match ::glob::glob(glob) {
                    Ok(paths) => {
                        for path in paths {
                            match path {
                                Ok(path) => {
                                    if let Some(name) = get_name(&path) {
                                        cases.insert(
                                            path.clone(),
                                            crate::Case {
                                                name: name.to_owned(),
                                                path: path,
                                                expected: spec.expected,
                                                error: None,
                                            },
                                        );
                                    } else {
                                        cases.insert(
                                            path.clone(),
                                            crate::Case::error(path, "path has no name"),
                                        );
                                    }
                                }
                                Err(err) => {
                                    let path = err.path().to_owned();
                                    let err = err.into_error();
                                    cases.insert(path.clone(), crate::Case::error(path, err));
                                }
                            }
                        }
                    }
                    Err(err) => {
                        cases.insert(
                            spec.glob.clone(),
                            crate::Case::error(spec.glob.clone(), err),
                        );
                    }
                }
            } else if let Some(name) = get_name(&spec.glob) {
                let path = spec.glob.as_path();
                cases.insert(
                    path.into(),
                    crate::Case {
                        name: name.into(),
                        path: path.into(),
                        expected: spec.expected,
                        error: None,
                    },
                );
            } else {
                cases.insert(
                    spec.glob.clone(),
                    crate::Case::error(spec.glob.clone(), "path has no name"),
                );
            }
        }

        for case in cases.into_values() {
            if self.is_included(&case) {
                runner.case(case);
            }
        }

        runner
    }

    fn is_included(&self, case: &crate::Case) -> bool {
        if let Some(include) = self.include.as_deref() {
            include
                .into_iter()
                .any(|i| case.path.to_string_lossy().contains(i))
        } else {
            true
        }
    }
}

impl Default for RunnerSpec {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct CaseSpec {
    glob: std::path::PathBuf,
    expected: Option<crate::Expected>,
}

fn get_glob(path: &std::path::Path) -> Option<&str> {
    if let Some(utf8) = path.to_str() {
        if utf8.contains('*') {
            return Some(utf8);
        }
    }

    None
}

fn get_name(path: &std::path::Path) -> Option<&str> {
    path.file_name().and_then(|os| os.to_str())
}
