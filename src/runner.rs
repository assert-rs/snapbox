use std::iter::FromIterator;

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
                palette.warn.paint("There are no trycmd tests enabled yet")
            );
        } else {
            let failures: Vec<_> = self
                .cases
                .par_iter()
                .filter_map(|c| match c.run() {
                    Ok(status) => {
                        eprintln!("{}", &status);
                        None
                    }
                    Err(status) => {
                        eprintln!("{}", &status);
                        Some(status)
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
    pub(crate) expected: Option<crate::CommandStatus>,
    pub(crate) timeout: Option<std::time::Duration>,
    pub(crate) default_bin: Option<crate::Bin>,
    pub(crate) error: Option<CaseStatus>,
}

impl Case {
    pub(crate) fn with_error(path: std::path::PathBuf, error: impl std::fmt::Display) -> Self {
        let name = path.display().to_string();
        Self {
            name: name,
            path: path.clone(),
            expected: None,
            timeout: None,
            default_bin: None,
            error: Some(CaseStatus::Failure {
                path,
                message: error.to_string(),
            }),
        }
    }

    pub(crate) fn to_err(&self, error: impl std::fmt::Display) -> CaseStatus {
        CaseStatus::Failure {
            path: self.path.clone(),
            message: error.to_string(),
        }
    }

    pub(crate) fn run(&self) -> Result<CaseStatus, CaseStatus> {
        if self.expected == Some(crate::CommandStatus::Skip) {
            return Ok(CaseStatus::Skipped {
                path: self.path.clone(),
            });
        }
        if let Some(err) = self.error.clone() {
            return Err(err);
        }

        let mut run = crate::TryCmd::load(&self.path).map_err(|e| self.to_err(e))?;
        if run.bin.is_none() {
            run.bin = self.default_bin.clone()
        }
        if run.timeout.is_none() {
            run.timeout = self.timeout;
        }
        if self.expected.is_some() {
            run.status = self.expected;
        }

        let stdin_path = self.path.with_extension("stdin");
        let stdin = if stdin_path.exists() {
            Some(read_stdin(&stdin_path, run.binary).map_err(|e| {
                self.to_err(format!("Failed to read {}: {}", stdin_path.display(), e))
            })?)
        } else {
            None
        };

        let output = run.to_output(stdin).map_err(|e| self.to_err(e))?;

        match run.status() {
            crate::CommandStatus::Pass => {
                if !output.status.success() {
                    return Err(CaseStatus::UnexpectedStatus {
                        path: self.path.clone(),
                        expected: "success".into(),
                        actual: output
                            .status
                            .code()
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "interrupted".into()),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    });
                }
            }
            crate::CommandStatus::Fail => {
                if output.status.success() || output.status.code().is_none() {
                    return Err(CaseStatus::UnexpectedStatus {
                        path: self.path.clone(),
                        expected: "failure".into(),
                        actual: output
                            .status
                            .code()
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "interrupted".into()),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    });
                }
            }
            crate::CommandStatus::Interrupted => {
                if let Some(code) = output.status.code() {
                    return Err(CaseStatus::UnexpectedStatus {
                        path: self.path.clone(),
                        expected: "interrupted".into(),
                        actual: code.to_string(),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    });
                }
            }
            crate::CommandStatus::Skip => unreachable!("handled earlier"),
            crate::CommandStatus::Code(expected_code) => {
                if let Some(actual_code) = output.status.code() {
                    if actual_code != expected_code {
                        return Err(CaseStatus::UnexpectedStatus {
                            path: self.path.clone(),
                            expected: expected_code.to_string(),
                            actual: actual_code.to_string(),
                            stdout: output.stdout,
                            stderr: output.stderr,
                        });
                    }
                } else {
                    return Err(CaseStatus::UnexpectedStatus {
                        path: self.path.clone(),
                        expected: expected_code.to_string(),
                        actual: "interrupted".into(),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    });
                }
            }
        }

        Ok(CaseStatus::Success {
            path: self.path.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CaseStatus {
    Success {
        path: std::path::PathBuf,
    },
    Skipped {
        path: std::path::PathBuf,
    },
    Failure {
        path: std::path::PathBuf,
        message: String,
    },
    UnexpectedStatus {
        path: std::path::PathBuf,
        expected: String,
        actual: String,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = crate::Palette::current();

        match self {
            Self::Success { path } => {
                writeln!(
                    f,
                    "{} {} ... {}",
                    palette.hint.paint("Testing"),
                    path.display(),
                    palette.error.paint("ok")
                )?;
            }
            Self::Skipped { path } => {
                writeln!(
                    f,
                    "{} {} ... {}",
                    palette.hint.paint("Testing"),
                    path.display(),
                    palette.warn.paint("ignored")
                )?;
            }
            Self::Failure { path, message } => {
                writeln!(
                    f,
                    "{} {} ... {}",
                    palette.hint.paint("Testing"),
                    path.display(),
                    palette.error.paint("failed")
                )?;
                writeln!(f, "{}", palette.error.paint(message))?;
            }
            Self::UnexpectedStatus {
                path,
                expected,
                actual,
                stdout,
                stderr,
            } => {
                writeln!(
                    f,
                    "{} {} ... {}",
                    palette.hint.paint("Testing"),
                    path.display(),
                    palette.error.paint("failed")
                )?;
                writeln!(
                    f,
                    "Expected {}, got {}",
                    palette.info.paint(expected),
                    palette.error.paint(actual)
                )?;
                writeln!(f, "stdout:")?;
                writeln!(f, "{}", palette.info.paint(String::from_utf8_lossy(stdout)))?;
                writeln!(f, "stderr:")?;
                writeln!(
                    f,
                    "{}",
                    palette.error.paint(String::from_utf8_lossy(stderr))
                )?;
            }
        }

        Ok(())
    }
}

fn read_stdin(path: &std::path::Path, binary: bool) -> Result<Vec<u8>, std::io::Error> {
    let stdin = if binary {
        let stdin = std::fs::read(&path)?;
        stdin
    } else {
        let stdin = std::fs::read_to_string(&path)?;
        let stdin = String::from_iter(normalize_line_endings::normalized(stdin.chars()));
        stdin.into_bytes()
    };
    Ok(stdin)
}
