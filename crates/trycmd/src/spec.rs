use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct RunnerSpec {
    cases: Vec<CaseSpec>,
    include: Option<Vec<String>>,
    default_bin: Option<crate::schema::Bin>,
    timeout: Option<std::time::Duration>,
    env: crate::schema::Env,
}

impl RunnerSpec {
    pub(crate) fn new() -> Self {
        Self {
            cases: Default::default(),
            include: None,
            default_bin: None,
            timeout: Default::default(),
            env: Default::default(),
        }
    }

    pub(crate) fn case(
        &mut self,
        glob: &std::path::Path,
        #[cfg_attr(miri, allow(unused_variables))] expected: Option<crate::schema::CommandStatus>,
    ) {
        self.cases.push(CaseSpec {
            glob: glob.into(),
            #[cfg(not(miri))]
            expected,
            #[cfg(miri)]
            expected: Some(crate::schema::CommandStatus::Skipped),
        });
    }

    pub(crate) fn include(&mut self, include: Option<Vec<String>>) {
        self.include = include;
    }

    pub(crate) fn default_bin(&mut self, bin: Option<crate::schema::Bin>) {
        self.default_bin = bin;
    }

    pub(crate) fn timeout(&mut self, time: Option<std::time::Duration>) {
        self.timeout = time;
    }

    pub(crate) fn env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.add.insert(key.into(), value.into());
    }

    pub(crate) fn prepare(&mut self) -> crate::Runner {
        let mut runner = crate::Runner::new();

        // Both sort and let the last writer win to allow overriding specific cases within a glob
        let mut cases: BTreeMap<std::path::PathBuf, crate::Case> = BTreeMap::new();

        for spec in &self.cases {
            if let Some(glob) = get_glob(&spec.glob) {
                match ::glob::glob(glob) {
                    Ok(paths) => {
                        for path in paths {
                            match path {
                                Ok(path) => {
                                    cases.insert(
                                        path.clone(),
                                        crate::Case {
                                            path,
                                            expected: spec.expected,
                                            default_bin: self.default_bin.clone(),
                                            timeout: self.timeout,
                                            env: self.env.clone(),
                                            error: None,
                                        },
                                    );
                                }
                                Err(err) => {
                                    let path = err.path().to_owned();
                                    let err = crate::Error::new(err.into_error().to_string());
                                    cases.insert(path.clone(), crate::Case::with_error(path, err));
                                }
                            }
                        }
                    }
                    Err(err) => {
                        let err = crate::Error::new(err.to_string());
                        cases.insert(
                            spec.glob.clone(),
                            crate::Case::with_error(spec.glob.clone(), err),
                        );
                    }
                }
            } else {
                let path = spec.glob.as_path();
                cases.insert(
                    path.into(),
                    crate::Case {
                        path: path.into(),
                        expected: spec.expected,
                        default_bin: self.default_bin.clone(),
                        timeout: self.timeout,
                        env: self.env.clone(),
                        error: None,
                    },
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
                .iter()
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
    expected: Option<crate::schema::CommandStatus>,
}

fn get_glob(path: &std::path::Path) -> Option<&str> {
    if let Some(utf8) = path.to_str() {
        if utf8.contains('*') {
            return Some(utf8);
        }
    }

    None
}
