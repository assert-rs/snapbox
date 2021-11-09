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

    pub(crate) fn run(&self, mode: &Mode, bins: &crate::BinRegistry) {
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
                .filter_map(|c| match c.run(mode, bins) {
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
                let stderr = std::io::stderr();
                let mut stderr = stderr.lock();
                let _ = writeln!(
                    stderr,
                    "{}",
                    palette
                        .hint
                        .paint("Update snapshots with `TRYCMD=overwrite`"),
                );
                let _ = writeln!(
                    stderr,
                    "{}",
                    palette.hint.paint("Debug output with `TRYCMD=dump`"),
                );
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
    pub(crate) path: std::path::PathBuf,
    pub(crate) expected: Option<crate::schema::CommandStatus>,
    pub(crate) timeout: Option<std::time::Duration>,
    pub(crate) default_bin: Option<crate::schema::Bin>,
    pub(crate) env: crate::schema::Env,
    pub(crate) error: Option<SpawnStatus>,
}

impl Case {
    pub(crate) fn with_error(path: std::path::PathBuf, error: impl std::fmt::Display) -> Self {
        Self {
            path,
            expected: None,
            timeout: None,
            default_bin: None,
            env: Default::default(),
            error: Some(SpawnStatus::Failure(error.to_string())),
        }
    }

    pub(crate) fn run(&self, mode: &Mode, bins: &crate::BinRegistry) -> Result<Output, Output> {
        let mut output = Output::default();

        if self.expected == Some(crate::schema::CommandStatus::Skipped) {
            assert_eq!(output.spawn.status, SpawnStatus::Skipped);
            return Ok(output);
        }
        if let Some(err) = self.error.clone() {
            output.spawn.status = err;
            return Err(output);
        }

        let mut sequence =
            crate::schema::TryCmd::load(&self.path).map_err(|e| output.clone().error(e))?;
        if sequence.run.bin.is_none() {
            sequence.run.bin = self.default_bin.clone()
        }
        sequence.run.bin = sequence
            .run
            .bin
            .map(|name| bins.resolve_bin(name))
            .transpose()
            .map_err(|e| output.clone().error(e))?;
        if sequence.run.timeout.is_none() {
            sequence.run.timeout = self.timeout;
        }
        if self.expected.is_some() {
            sequence.run.status = self.expected;
        }
        sequence.run.env.update(&self.env);

        let fs = crate::FilesystemContext::new(
            &self.path,
            sequence.fs.base.as_deref(),
            sequence.fs.sandbox(),
            mode,
        )
        .map_err(|e| {
            output
                .clone()
                .error(format!("Failed to initialize sandbox: {}", e))
        })?;
        let cwd = fs
            .path()
            .map(|p| sequence.fs.rel_cwd().map(|rel| p.join(rel)))
            .transpose()
            .map_err(|e| output.clone().error(e))?;

        let cmd_output = sequence
            .run
            .to_output(cwd.as_deref())
            .map_err(|e| output.clone().error(e))?;
        let output = output.output(cmd_output);

        // For dump mode's sake, allow running all
        let mut ok = output.is_ok();
        let mut output = match self.validate_spawn(output, sequence.run.status()) {
            Ok(output) => output,
            Err(output) => {
                ok = false;
                output
            }
        };
        if let Some(mut stdout) = output.stdout {
            if !sequence.run.binary {
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
            if !sequence.run.binary {
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
        if sequence.fs.sandbox() {
            output.fs =
                match self.validate_fs(fs.path().expect("sandbox must be filled"), output.fs, mode)
                {
                    Ok(fs) => fs,
                    Err(fs) => {
                        ok = false;
                        fs
                    }
                };
        }
        if let Err(err) = fs.close() {
            output.fs.context.push(FileStatus::Failure(format!(
                "Failed to cleanup sandbox: {}",
                err
            )));
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
        expected: crate::schema::CommandStatus,
    ) -> Result<Output, Output> {
        let status = output.spawn.exit.expect("bale out before now");
        match expected {
            crate::schema::CommandStatus::Success => {
                if !status.success() {
                    output.spawn.status = SpawnStatus::Expected("success".into());
                }
            }
            crate::schema::CommandStatus::Failed => {
                if status.success() || status.code().is_none() {
                    output.spawn.status = SpawnStatus::Expected("failure".into());
                }
            }
            crate::schema::CommandStatus::Interrupted => {
                if status.code().is_some() {
                    output.spawn.status = SpawnStatus::Expected("interrupted".into());
                }
            }
            crate::schema::CommandStatus::Skipped => unreachable!("handled earlier"),
            crate::schema::CommandStatus::Code(expected_code) => {
                if Some(expected_code) != status.code() {
                    output.spawn.status = SpawnStatus::Expected(expected_code.to_string());
                }
            }
        }

        Ok(output)
    }

    fn validate_stream(&self, mut stream: Stream, mode: &Mode) -> Result<Stream, Stream> {
        if let Mode::Dump(root) = mode {
            let stdout_path = root.join(
                self.path
                    .with_extension(stream.stream.as_str())
                    .file_name()
                    .unwrap(),
            );
            stream.content.write_to(&stdout_path).map_err(|e| {
                let mut stream = stream.clone();
                stream.status = StreamStatus::Failure(e);
                stream
            })?;
        } else {
            let stdout_path = self.path.with_extension(stream.stream.as_str());
            if stdout_path.exists() {
                let expected_content =
                    crate::File::read_from(&stdout_path, stream.content.is_binary()).map_err(
                        |e| {
                            let mut stream = stream.clone();
                            stream.status = StreamStatus::Failure(e);
                            stream
                        },
                    )?;

                if let crate::File::Text(e) = &expected_content {
                    stream.content = stream.content.map_text(|t| crate::elide::normalize(t, e));
                }
                if stream.content != expected_content {
                    match mode {
                        Mode::Fail => {
                            stream.status = StreamStatus::Expected(expected_content);
                            return Err(stream);
                        }
                        Mode::Overwrite => {
                            stream.content.write_to(&stdout_path).map_err(|e| {
                                let mut stream = stream.clone();
                                stream.status = StreamStatus::Failure(e);
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

    fn validate_fs(
        &self,
        actual_root: &std::path::Path,
        mut fs: Filesystem,
        mode: &Mode,
    ) -> Result<Filesystem, Filesystem> {
        let mut ok = true;

        if let Mode::Dump(_) = mode {
            // Handled as part of FilesystemContext
        } else {
            let fixture_root = self.path.with_extension("out");
            if fixture_root.exists() {
                for expected_path in crate::FsIterate::new(&fixture_root) {
                    if expected_path
                        .as_deref()
                        .map(|p| p.is_dir())
                        .unwrap_or_default()
                    {
                        continue;
                    }

                    match self.validate_path(expected_path, &fixture_root, actual_root) {
                        Ok(status) => {
                            fs.context.push(status);
                        }
                        Err(status) => {
                            let mut is_current_ok = false;
                            if *mode == Mode::Overwrite {
                                match &status {
                                    FileStatus::TypeMismatch {
                                        expected_path,
                                        actual_path,
                                        ..
                                    } => {
                                        if crate::shallow_copy(expected_path, actual_path).is_ok() {
                                            is_current_ok = true;
                                        }
                                    }
                                    FileStatus::LinkMismatch {
                                        expected_path,
                                        actual_path,
                                        ..
                                    } => {
                                        if crate::shallow_copy(expected_path, actual_path).is_ok() {
                                            is_current_ok = true;
                                        }
                                    }
                                    FileStatus::ContentMismatch {
                                        expected_path,
                                        actual_content,
                                        ..
                                    } => {
                                        if actual_content.write_to(expected_path).is_ok() {
                                            is_current_ok = true;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            fs.context.push(status);
                            if !is_current_ok {
                                ok = false;
                            }
                        }
                    }
                }
            }
        }

        if ok {
            Ok(fs)
        } else {
            Err(fs)
        }
    }

    fn validate_path(
        &self,
        expected_path: Result<std::path::PathBuf, std::io::Error>,
        fixture_root: &std::path::Path,
        actual_root: &std::path::Path,
    ) -> Result<FileStatus, FileStatus> {
        let expected_path = expected_path.map_err(|e| FileStatus::Failure(e.to_string()))?;
        let expected_meta = expected_path
            .symlink_metadata()
            .map_err(|e| FileStatus::Failure(e.to_string()))?;
        let expected_target = std::fs::read_link(&expected_path).ok();

        let rel = expected_path.strip_prefix(&fixture_root).unwrap();
        let actual_path = actual_root.join(rel);
        let actual_meta = actual_path.symlink_metadata().ok();
        let actual_target = std::fs::read_link(&actual_path).ok();

        let expected_type = if expected_meta.is_dir() {
            FileType::Dir
        } else if expected_meta.is_file() {
            FileType::File
        } else if expected_target.is_some() {
            FileType::Symlink
        } else {
            FileType::Unknown
        };
        let actual_type = if let Some(actual_meta) = actual_meta {
            if actual_meta.is_dir() {
                FileType::Dir
            } else if actual_meta.is_file() {
                FileType::File
            } else if actual_target.is_some() {
                FileType::Symlink
            } else {
                FileType::Unknown
            }
        } else {
            FileType::Missing
        };
        if expected_type != actual_type {
            return Err(FileStatus::TypeMismatch {
                expected_path,
                actual_path,
                expected_type,
                actual_type,
            });
        }

        match expected_type {
            FileType::Symlink => {
                if expected_target != actual_target {
                    return Err(FileStatus::LinkMismatch {
                        expected_path,
                        actual_path,
                        expected_target: expected_target.unwrap(),
                        actual_target: actual_target.unwrap(),
                    });
                }
            }
            FileType::File => {
                let expected_content = crate::File::read_from(&expected_path, true)
                    .map_err(|e| FileStatus::Failure(e))?
                    .try_utf8();
                let mut actual_content = crate::File::read_from(&actual_path, true)
                    .map_err(|e| FileStatus::Failure(e))?
                    .try_utf8();

                if let crate::File::Text(e) = &expected_content {
                    actual_content = actual_content.map_text(|t| crate::elide::normalize(t, e));
                }
                if expected_content != actual_content {
                    return Err(FileStatus::ContentMismatch {
                        expected_path,
                        actual_path,
                        expected_content,
                        actual_content,
                    });
                }
            }
            FileType::Dir | FileType::Unknown | FileType::Missing => {}
        }

        Ok(FileStatus::Ok {
            expected_path,
            actual_path,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Output {
    spawn: Spawn,
    stdout: Option<Stream>,
    stderr: Option<Stream>,
    fs: Filesystem,
}

impl Output {
    fn output(mut self, output: std::process::Output) -> Self {
        self.spawn.exit = Some(output.status);
        assert_eq!(self.spawn.status, SpawnStatus::Skipped);
        self.spawn.status = SpawnStatus::Ok;
        self.stdout = Some(Stream {
            stream: Stdio::Stdout,
            content: crate::File::Binary(output.stdout),
            status: StreamStatus::Ok,
        });
        self.stderr = Some(Stream {
            stream: Stdio::Stderr,
            content: crate::File::Binary(output.stderr),
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
            && self.fs.is_ok()
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
            fs: Default::default(),
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
        self.fs.fmt(f)?;

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
                            "Expected {}, was {}",
                            palette.info.paint(expected),
                            palette.error.paint("success")
                        )?;
                    } else if let Some(code) = exit.code() {
                        writeln!(
                            f,
                            "Expected {}, was {}",
                            palette.info.paint(expected),
                            palette.error.paint(code)
                        )?;
                    } else {
                        writeln!(
                            f,
                            "Expected {}, was {}",
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
    content: crate::File,
    status: StreamStatus,
}

impl Stream {
    fn utf8(mut self) -> Self {
        if self.content.utf8().is_err() {
            self.status = StreamStatus::Failure("invalid UTF-8".to_string());
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
                #[allow(unused_mut)]
                let mut rendered = false;
                #[cfg(feature = "diff")]
                if let (crate::File::Text(expected), crate::File::Text(actual)) =
                    (&expected, &self.content)
                {
                    let diff =
                        crate::diff::diff(expected, actual, self.stream, self.stream, palette);
                    writeln!(f, "{}", diff)?;
                    rendered = true;
                }

                if !rendered {
                    writeln!(f, "{} {}:", self.stream, palette.info.paint("(expected)"))?;
                    writeln!(f, "{}", palette.info.paint(&expected))?;
                    writeln!(f, "{} {}:", self.stream, palette.error.paint("(actual)"))?;
                    writeln!(f, "{}", palette.error.paint(&self.content))?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StreamStatus {
    Ok,
    Failure(String),
    Expected(crate::File),
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

#[derive(Clone, Default, Debug, PartialEq, Eq)]
struct Filesystem {
    context: Vec<FileStatus>,
}

impl Filesystem {
    fn is_ok(&self) -> bool {
        if self.context.is_empty() {
            true
        } else {
            self.context.iter().all(FileStatus::is_ok)
        }
    }
}

impl std::fmt::Display for Filesystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for status in &self.context {
            status.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FileStatus {
    Ok {
        expected_path: std::path::PathBuf,
        actual_path: std::path::PathBuf,
    },
    Failure(String),
    TypeMismatch {
        expected_path: std::path::PathBuf,
        actual_path: std::path::PathBuf,
        expected_type: FileType,
        actual_type: FileType,
    },
    LinkMismatch {
        expected_path: std::path::PathBuf,
        actual_path: std::path::PathBuf,
        expected_target: std::path::PathBuf,
        actual_target: std::path::PathBuf,
    },
    ContentMismatch {
        expected_path: std::path::PathBuf,
        actual_path: std::path::PathBuf,
        expected_content: crate::File,
        actual_content: crate::File,
    },
}

impl FileStatus {
    fn is_ok(&self) -> bool {
        match self {
            Self::Ok { .. } => true,
            Self::Failure(_)
            | Self::TypeMismatch { .. }
            | Self::LinkMismatch { .. }
            | Self::ContentMismatch { .. } => false,
        }
    }
}

impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = crate::Palette::current();

        match &self {
            FileStatus::Ok {
                expected_path,
                actual_path: _actual_path,
            } => {
                writeln!(
                    f,
                    "{}: is {}",
                    expected_path.display(),
                    palette.info.paint("good"),
                )?;
            }
            FileStatus::Failure(msg) => {
                writeln!(f, "{}", palette.error.paint(msg))?;
            }
            FileStatus::TypeMismatch {
                expected_path,
                actual_path: _actual_path,
                expected_type,
                actual_type,
            } => {
                writeln!(
                    f,
                    "{}: Expected {}, was {}",
                    expected_path.display(),
                    palette.info.paint(expected_type),
                    palette.error.paint(actual_type)
                )?;
            }
            FileStatus::LinkMismatch {
                expected_path,
                actual_path: _actual_path,
                expected_target,
                actual_target,
            } => {
                writeln!(
                    f,
                    "{}: Expected {}, was {}",
                    expected_path.display(),
                    palette.info.paint(expected_target.display()),
                    palette.error.paint(actual_target.display())
                )?;
            }
            FileStatus::ContentMismatch {
                expected_path,
                actual_path,
                expected_content,
                actual_content,
            } => {
                #[allow(unused_mut)]
                let mut rendered = false;
                #[cfg(feature = "diff")]
                if let (crate::File::Text(expected), crate::File::Text(actual)) =
                    (&expected_content, &actual_content)
                {
                    let diff = crate::diff::diff(
                        expected,
                        actual,
                        expected_path.display(),
                        actual_path.display(),
                        palette,
                    );
                    writeln!(f, "{}", diff)?;
                    rendered = true;
                }

                if !rendered {
                    writeln!(
                        f,
                        "{} {}:",
                        expected_path.display(),
                        palette.info.paint("(expected)")
                    )?;
                    writeln!(f, "{}", palette.info.paint(&expected_content))?;
                    writeln!(
                        f,
                        "{} {}:",
                        actual_path.display(),
                        palette.error.paint("(actual)")
                    )?;
                    writeln!(f, "{}", palette.error.paint(&actual_content))?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FileType {
    Dir,
    File,
    Symlink,
    Unknown,
    Missing,
}

impl FileType {
    fn as_str(&self) -> &str {
        match self {
            Self::Dir => "dir",
            Self::File => "file",
            Self::Symlink => "symlink",
            Self::Unknown => "unknown",
            Self::Missing => "missing",
        }
    }
}

impl std::fmt::Display for FileType {
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
            Self::Dump(root) => {
                std::fs::create_dir_all(root)?;
                let gitignore_path = root.join(".gitignore");
                std::fs::write(gitignore_path, "*\n")?;
            }
        }

        Ok(())
    }
}
