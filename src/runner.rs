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
    pub(crate) env: crate::Env,
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
            env: Default::default(),
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
        run.env.update(&self.env);

        let stdin_path = self.path.with_extension("stdin");
        let stdin = if stdin_path.exists() {
            Some(
                File::read_from(&stdin_path, run.binary)
                    .map_err(|e| {
                        self.to_err(format!("Failed to read {}: {}", stdin_path.display(), e))
                    })?
                    .into_bytes(),
            )
        } else {
            None
        };

        let output = run.to_output(stdin).map_err(|e| self.to_err(e))?;

        self.validate_status(&run, &output)?;
        self.validate_stream(&run, &output, Stdio::Stdout)?;
        self.validate_stream(&run, &output, Stdio::Stderr)?;

        Ok(CaseStatus::Success {
            path: self.path.clone(),
        })
    }

    fn validate_status(
        &self,
        run: &crate::TryCmd,
        output: &std::process::Output,
    ) -> Result<(), CaseStatus> {
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
                        stdout: output.stdout.clone(),
                        stderr: output.stderr.clone(),
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
                        stdout: output.stdout.clone(),
                        stderr: output.stderr.clone(),
                    });
                }
            }
            crate::CommandStatus::Interrupted => {
                if let Some(code) = output.status.code() {
                    return Err(CaseStatus::UnexpectedStatus {
                        path: self.path.clone(),
                        expected: "interrupted".into(),
                        actual: code.to_string(),
                        stdout: output.stdout.clone(),
                        stderr: output.stderr.clone(),
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
                            stdout: output.stdout.clone(),
                            stderr: output.stderr.clone(),
                        });
                    }
                } else {
                    return Err(CaseStatus::UnexpectedStatus {
                        path: self.path.clone(),
                        expected: expected_code.to_string(),
                        actual: "interrupted".into(),
                        stdout: output.stdout.clone(),
                        stderr: output.stderr.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_stream(
        &self,
        run: &crate::TryCmd,
        output: &std::process::Output,
        stream: Stdio,
    ) -> Result<(), CaseStatus> {
        let stdout = match stream {
            Stdio::Stdout => &output.stdout,
            Stdio::Stderr => &output.stderr,
        };

        let stdout = if run.binary {
            let data = stdout.clone();
            File::Binary(data)
        } else {
            let data = String::from_utf8(stdout.clone()).map_err(|_| CaseStatus::InvalidUtf8 {
                path: self.path.clone(),
                stream: Stdio::Stdout,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            })?;
            File::Text(data)
        };

        let stdout_path = self.path.with_extension(stream.as_str());
        if stdout_path.exists() {
            let expected_stdout = File::read_from(&stdout_path, run.binary).map_err(|e| {
                self.to_err(format!("Failed to read {}: {}", stdout_path.display(), e))
            })?;

            if stdout != expected_stdout {
                return Err(CaseStatus::MismatchOutput {
                    path: self.path.clone(),
                    stream: Stdio::Stdout,
                    expected: expected_stdout,
                    stdout: output.stdout.clone(),
                    stderr: output.stderr.clone(),
                });
            }
        }

        Ok(())
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
    InvalidUtf8 {
        path: std::path::PathBuf,
        stream: Stdio,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    MismatchOutput {
        path: std::path::PathBuf,
        stream: Stdio,
        expected: File,
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
            Self::InvalidUtf8 {
                path,
                stream,
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
                    "Expected utf-8 on {}",
                    match stream {
                        Stdio::Stdout => palette.info.paint(stream.as_str()),
                        Stdio::Stderr => palette.error.paint(stream.as_str()),
                    },
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
            Self::MismatchOutput {
                path,
                stream,
                expected,
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
                    "{} didn't match expectations",
                    match stream {
                        Stdio::Stdout => palette.info.paint(stream.as_str()),
                        Stdio::Stderr => palette.error.paint(stream.as_str()),
                    },
                )?;
                writeln!(f, "stdout:")?;
                writeln!(f, "{}", palette.info.paint(String::from_utf8_lossy(stdout)))?;
                writeln!(f, "expected {}:", stream)?;
                writeln!(f, "{}", palette.warn.paint(expected.to_string_lossy()))?;
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Stdio {
    Stdout,
    Stderr,
}

impl Stdio {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

impl std::fmt::Display for Stdio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum File {
    Binary(Vec<u8>),
    Text(String),
}

impl File {
    pub(crate) fn read_from(path: &std::path::Path, binary: bool) -> Result<Self, std::io::Error> {
        let data = if binary {
            let data = std::fs::read(&path)?;
            Self::Binary(data)
        } else {
            let data = std::fs::read_to_string(&path)?;
            let data = String::from_iter(normalize_line_endings::normalized(data.chars()));
            Self::Text(data)
        };
        Ok(data)
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Binary(data) => data,
            Self::Text(data) => data.into_bytes(),
        }
    }

    pub(crate) fn to_string_lossy(&self) -> String {
        match self {
            Self::Binary(data) => String::from_utf8_lossy(&data).into_owned(),
            Self::Text(data) => data.clone(),
        }
    }
}
