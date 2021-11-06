use std::io::prelude::*;

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

    pub(crate) fn run(&self, mode: &Mode) {
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
                .filter_map(|c| match c.run(mode) {
                    Ok(status) => {
                        let stderr = std::io::stderr();
                        let mut stderr = stderr.lock();
                        let _ = writeln!(
                            stderr,
                            "{} {} ... {}",
                            palette.hint.paint("Testing"),
                            c.path.display(),
                            palette.info.paint("ok")
                        );
                        if !status.is_ok() {
                            // Assuming `status` will print the newline
                            let _ = write!(stderr, "{}", &status);
                        }
                        None
                    }
                    Err(status) => {
                        let stderr = std::io::stderr();
                        let mut stderr = stderr.lock();
                        let _ = writeln!(
                            stderr,
                            "{} {} ... {}",
                            palette.hint.paint("Testing"),
                            c.path.display(),
                            palette.error.paint("failed")
                        );
                        // Assuming `status` will print the newline
                        let _ = write!(stderr, "{}", &status);
                        Some(status)
                    }
                })
                .collect();

            if !failures.is_empty() {
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
    pub(crate) error: Option<SpawnStatus>,
}

impl Case {
    pub(crate) fn with_error(path: std::path::PathBuf, error: impl std::fmt::Display) -> Self {
        let name = path.display().to_string();
        Self {
            name,
            path: path.clone(),
            expected: None,
            timeout: None,
            default_bin: None,
            env: Default::default(),
            error: Some(SpawnStatus::Failure(error.to_string())),
        }
    }

    pub(crate) fn run(&self, mode: &Mode) -> Result<Output, Output> {
        let mut output = Output::default();

        if self.expected == Some(crate::CommandStatus::Skip) {
            assert_eq!(output.spawn.status, SpawnStatus::Skipped);
            return Ok(output);
        }
        if let Some(err) = self.error.clone() {
            output.spawn.status = err;
            return Err(output);
        }

        let mut run = crate::TryCmd::load(&self.path).map_err(|e| output.clone().error(e))?;
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
                        output.clone().error(format!(
                            "Failed to read {}: {}",
                            stdin_path.display(),
                            e
                        ))
                    })?
                    .into_bytes(),
            )
        } else {
            None
        };

        let cmd_output = run.to_output(stdin).map_err(|e| output.clone().error(e))?;
        let output = output.output(cmd_output);

        // For dump mode's sake, allow running all
        let mut ok = output.is_ok();
        let mut output = match self.validate_spawn(output, run.status()) {
            Ok(output) => output,
            Err(output) => {
                ok = false;
                output
            }
        };
        if let Some(mut stdout) = output.stdout {
            if !run.binary {
                stdout = stdout.utf8();
            }
            if stdout.is_ok() {
                stdout = match self.validate_stream(stdout, mode) {
                    Ok(stdout) => stdout,
                    Err(stdout) => {
                        ok = false;
                        stdout
                    }
                };
            }
            output.stdout = Some(stdout);
        }
        if let Some(mut stderr) = output.stderr {
            if !run.binary {
                stderr = stderr.utf8();
            }
            if stderr.is_ok() {
                stderr = match self.validate_stream(stderr, mode) {
                    Ok(stderr) => stderr,
                    Err(stderr) => {
                        ok = false;
                        stderr
                    }
                };
            }
            output.stderr = Some(stderr);
        }

        if ok {
            Ok(output)
        } else {
            Err(output)
        }
    }

    fn validate_spawn(
        &self,
        mut output: Output,
        expected: crate::CommandStatus,
    ) -> Result<Output, Output> {
        let status = output.spawn.exit.expect("bale out before now");
        match expected {
            crate::CommandStatus::Pass => {
                if !status.success() {
                    output.spawn.status = SpawnStatus::Expected("success".into());
                }
            }
            crate::CommandStatus::Fail => {
                if status.success() || status.code().is_none() {
                    output.spawn.status = SpawnStatus::Expected("failure".into());
                }
            }
            crate::CommandStatus::Interrupted => {
                if status.code().is_some() {
                    output.spawn.status = SpawnStatus::Expected("interrupted".into());
                }
            }
            crate::CommandStatus::Skip => unreachable!("handled earlier"),
            crate::CommandStatus::Code(expected_code) => {
                if Some(expected_code) != status.code() {
                    output.spawn.status = SpawnStatus::Expected(expected_code.to_string());
                }
            }
        }

        Ok(output)
    }

    fn validate_stream(&self, mut stream: Stream, mode: &Mode) -> Result<Stream, Stream> {
        if let Mode::Dump(path) = mode {
            let stdout_path = path.join(
                self.path
                    .with_extension(stream.stream.as_str())
                    .file_name()
                    .unwrap(),
            );
            stream.content.write_to(&stdout_path).map_err(|e| {
                let mut stream = stream.clone();
                stream.status = StreamStatus::Failure(format!(
                    "Failed to read {}: {}",
                    stdout_path.display(),
                    e
                ));
                stream
            })?;
        } else {
            let stdout_path = self.path.with_extension(stream.stream.as_str());
            if stdout_path.exists() {
                let expected_content = File::read_from(&stdout_path, stream.content.is_binary())
                    .map_err(|e| {
                        let mut stream = stream.clone();
                        stream.status = StreamStatus::Failure(format!(
                            "Failed to read {}: {}",
                            stdout_path.display(),
                            e
                        ));
                        stream
                    })?;

                if stream.content != expected_content {
                    match mode {
                        Mode::Fail => {
                            stream.status = StreamStatus::Expected(expected_content);
                            return Err(stream);
                        }
                        Mode::Overwrite => {
                            stream.content.write_to(&stdout_path).map_err(|e| {
                                let mut stream = stream.clone();
                                stream.status = StreamStatus::Failure(format!(
                                    "Failed to write {}: {}",
                                    stdout_path.display(),
                                    e
                                ));
                                stream
                            })?;
                            stream.status = StreamStatus::Expected(expected_content);
                            return Ok(stream);
                        }
                        Mode::Dump(_) => unreachable!("handled earlier"),
                    }
                }
            }
        }

        Ok(stream)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Output {
    spawn: Spawn,
    stdout: Option<Stream>,
    stderr: Option<Stream>,
}

impl Output {
    fn output(mut self, output: std::process::Output) -> Self {
        self.spawn.exit = Some(output.status);
        assert_eq!(self.spawn.status, SpawnStatus::Skipped);
        self.spawn.status = SpawnStatus::Ok;
        self.stdout = Some(Stream {
            stream: Stdio::Stdout,
            content: File::Binary(output.stdout),
            status: StreamStatus::Ok,
        });
        self.stderr = Some(Stream {
            stream: Stdio::Stderr,
            content: File::Binary(output.stderr),
            status: StreamStatus::Ok,
        });
        self
    }

    fn error(mut self, msg: impl std::fmt::Display) -> Self {
        self.spawn.status = SpawnStatus::Failure(msg.to_string());
        self
    }

    fn is_ok(&self) -> bool {
        self.spawn.is_ok()
            && self.stdout.as_ref().map(|s| s.is_ok()).unwrap_or_default()
            && self.stderr.as_ref().map(|s| s.is_ok()).unwrap_or_default()
    }
}

impl Default for Output {
    fn default() -> Self {
        Self {
            spawn: Spawn {
                exit: None,
                status: SpawnStatus::Skipped,
            },
            stdout: None,
            stderr: None,
        }
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.spawn.fmt(f)?;
        if let Some(stdout) = &self.stdout {
            stdout.fmt(f)?;
        }
        if let Some(stderr) = &self.stderr {
            stderr.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Spawn {
    exit: Option<std::process::ExitStatus>,
    status: SpawnStatus,
}

impl Spawn {
    fn is_ok(&self) -> bool {
        self.status.is_ok()
    }
}

impl std::fmt::Display for Spawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = crate::Palette::current();

        match &self.status {
            SpawnStatus::Ok => {
                if let Some(exit) = self.exit {
                    if exit.success() {
                        writeln!(f, "Exit: {}", palette.info.paint("success"))?;
                    } else if let Some(code) = exit.code() {
                        writeln!(f, "Exit: {}", palette.error.paint(code))?;
                    } else {
                        writeln!(f, "Exit: {}", palette.error.paint("interrupted"))?;
                    }
                }
            }
            SpawnStatus::Skipped => {
                writeln!(f, "{}", palette.warn.paint("Skipped"))?;
            }
            SpawnStatus::Failure(msg) => {
                writeln!(f, "Failed: {}", palette.error.paint(msg))?;
            }
            SpawnStatus::Expected(expected) => {
                if let Some(exit) = self.exit {
                    if exit.success() {
                        writeln!(
                            f,
                            "Expected {}, got {}",
                            palette.info.paint(expected),
                            palette.error.paint("success")
                        )?;
                    } else if let Some(code) = exit.code() {
                        writeln!(
                            f,
                            "Expected {}, got {}",
                            palette.info.paint(expected),
                            palette.error.paint(code)
                        )?;
                    } else {
                        writeln!(
                            f,
                            "Expected {}, got {}",
                            palette.info.paint(expected),
                            palette.error.paint("interrupted")
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SpawnStatus {
    Ok,
    Skipped,
    Failure(String),
    Expected(String),
}

impl SpawnStatus {
    fn is_ok(&self) -> bool {
        match self {
            Self::Ok | Self::Skipped => true,
            Self::Failure(_) | Self::Expected(_) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Stream {
    stream: Stdio,
    content: File,
    status: StreamStatus,
}

impl Stream {
    fn utf8(mut self) -> Self {
        if let Err(_) = self.content.utf8() {
            self.status = StreamStatus::Failure("invalud UTF-8".to_string());
        }
        self
    }

    fn is_ok(&self) -> bool {
        self.status.is_ok()
    }
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = crate::Palette::current();

        match &self.status {
            StreamStatus::Ok => {
                writeln!(f, "{}:", self.stream)?;
                writeln!(f, "{}", palette.info.paint(&self.content))?;
            }
            StreamStatus::Failure(msg) => {
                writeln!(
                    f,
                    "{} {}:",
                    self.stream,
                    palette.error.paint(format_args!("({})", msg))
                )?;
                writeln!(f, "{}", palette.info.paint(&self.content))?;
            }
            StreamStatus::Expected(expected) => {
                writeln!(f, "{} {}:", self.stream, palette.info.paint("(expected)"))?;
                writeln!(f, "{}", palette.info.paint(&expected))?;
                writeln!(f, "{} {}:", self.stream, palette.error.paint("(actual)"))?;
                writeln!(f, "{}", palette.error.paint(&self.content))?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StreamStatus {
    Ok,
    Failure(String),
    Expected(File),
}

impl StreamStatus {
    fn is_ok(&self) -> bool {
        match self {
            Self::Ok => true,
            Self::Failure(_) | Self::Expected(_) => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Stdio {
    Stdout,
    Stderr,
}

impl Stdio {
    fn as_str(&self) -> &str {
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
pub(crate) enum Mode {
    Fail,
    Overwrite,
    Dump(std::path::PathBuf),
}

impl Mode {
    pub(crate) fn initialize(&self) -> Result<(), std::io::Error> {
        match self {
            Self::Fail => {}
            Self::Overwrite => {}
            Self::Dump(path) => {
                std::fs::create_dir_all(path)?;
                let gitignore_path = path.join(".gitignore");
                std::fs::write(gitignore_path, "*\n")?;
            }
        }

        Ok(())
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
            let data = normalize_line_endings::normalized(data.chars()).collect();
            Self::Text(data)
        };
        Ok(data)
    }

    pub(crate) fn write_to(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        std::fs::write(path, self.as_bytes())
    }

    pub(crate) fn is_binary(&self) -> bool {
        match self {
            Self::Binary(_) => true,
            Self::Text(_) => false,
        }
    }

    pub(crate) fn utf8(&mut self) -> Result<(), std::str::Utf8Error> {
        match self {
            Self::Binary(data) => {
                *self = Self::Text(String::from_utf8(data.clone()).map_err(|e| e.utf8_error())?);
                Ok(())
            }
            Self::Text(_) => Ok(()),
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Binary(data) => data,
            Self::Text(data) => data.as_bytes(),
        }
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Binary(data) => data,
            Self::Text(data) => data.into_bytes(),
        }
    }
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary(data) => String::from_utf8_lossy(data).fmt(f),
            Self::Text(data) => data.fmt(f),
        }
    }
}
