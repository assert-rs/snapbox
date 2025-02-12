use std::io::prelude::*;

#[cfg(feature = "color")]
use anstream::eprintln;
#[cfg(feature = "color")]
use anstream::panic;
#[cfg(feature = "color")]
use anstream::stderr;
#[cfg(not(feature = "color"))]
use std::eprintln;
#[cfg(not(feature = "color"))]
use std::io::stderr;

use rayon::prelude::*;
use snapbox::data::DataFormat;
use snapbox::dir::FileType;
use snapbox::filter::{Filter as _, FilterNewlines, FilterPaths, NormalizeToExpected};
use snapbox::IntoData;

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

    pub(crate) fn run(
        &self,
        mode: &Mode,
        bins: &crate::BinRegistry,
        substitutions: &snapbox::Redactions,
    ) {
        #![allow(unexpected_cfgs)] // HACK: until we upgrade the minimum anstream
        let palette = snapbox::report::Palette::color();

        if self.cases.is_empty() {
            eprintln!("{}", palette.warn("There are no trycmd tests enabled yet"));
        } else {
            let failures: Vec<_> = self
                .cases
                .par_iter()
                .flat_map(|c| {
                    let results = c.run(mode, bins, substitutions);

                    let stderr = stderr();
                    let mut stderr = stderr.lock();

                    results
                        .into_iter()
                        .filter_map(|s| {
                            snapbox::debug!("Case: {:#?}", s);
                            match s {
                                Ok(status) => {
                                    let _ = write!(
                                        stderr,
                                        "{} {} ... {}",
                                        palette.hint("Testing"),
                                        status.name(),
                                        status.spawn.status.summary(),
                                    );
                                    if let Some(duration) = status.duration {
                                        let _ = write!(
                                            stderr,
                                            " {}",
                                            palette.hint(humantime::format_duration(duration)),
                                        );
                                    }
                                    let _ = writeln!(stderr);
                                    if !status.is_ok() {
                                        // Assuming `status` will print the newline
                                        let _ = write!(stderr, "{}", &status);
                                    }
                                    None
                                }
                                Err(status) => {
                                    let _ = write!(
                                        stderr,
                                        "{} {} ... {}",
                                        palette.hint("Testing"),
                                        status.name(),
                                        palette.error("failed"),
                                    );
                                    if let Some(duration) = status.duration {
                                        let _ = write!(
                                            stderr,
                                            " {}",
                                            palette.hint(humantime::format_duration(duration)),
                                        );
                                    }
                                    let _ = writeln!(stderr);
                                    // Assuming `status` will print the newline
                                    let _ = write!(stderr, "{}", &status);
                                    Some(status)
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            if !failures.is_empty() {
                let stderr = stderr();
                let mut stderr = stderr.lock();
                let _ = writeln!(
                    stderr,
                    "{}",
                    palette.hint("Update snapshots with `TRYCMD=overwrite`"),
                );
                let _ = writeln!(
                    stderr,
                    "{}",
                    palette.hint("Debug output with `TRYCMD=dump`"),
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
    pub(crate) fn with_error(path: std::path::PathBuf, error: crate::Error) -> Self {
        Self {
            path,
            expected: None,
            timeout: None,
            default_bin: None,
            env: Default::default(),
            error: Some(SpawnStatus::Failure(error)),
        }
    }

    pub(crate) fn run(
        &self,
        mode: &Mode,
        bins: &crate::BinRegistry,
        substitutions: &snapbox::Redactions,
    ) -> Vec<Result<Output, Output>> {
        if self.expected == Some(crate::schema::CommandStatus::Skipped) {
            let output = Output::sequence(self.path.clone());
            assert_eq!(output.spawn.status, SpawnStatus::Skipped);
            return vec![Ok(output)];
        }

        if let Some(err) = self.error.clone() {
            let mut output = Output::step(self.path.clone(), "setup".into());
            output.spawn.status = err;
            return vec![Err(output)];
        }

        let mut sequence = match crate::schema::TryCmd::load(&self.path) {
            Ok(sequence) => sequence,
            Err(e) => {
                let output = Output::step(self.path.clone(), "setup".into());
                return vec![Err(output.error(e))];
            }
        };

        if sequence.steps.is_empty() {
            let output = Output::sequence(self.path.clone());
            assert_eq!(output.spawn.status, SpawnStatus::Skipped);
            return vec![Ok(output)];
        }

        let fs_context = match fs_context(
            &self.path,
            sequence.fs.base.as_deref(),
            sequence.fs.sandbox(),
            mode,
        ) {
            Ok(fs_context) => fs_context,
            Err(e) => {
                let output = Output::step(self.path.clone(), "setup".into());
                return vec![Err(
                    output.error(format!("Failed to initialize sandbox: {e}").into())
                )];
            }
        };
        let cwd = match fs_context
            .path()
            .map(|p| {
                sequence.fs.rel_cwd().map(|rel| {
                    let p = p.join(rel);
                    snapbox::dir::strip_trailing_slash(&p).to_owned()
                })
            })
            .transpose()
        {
            Ok(cwd) => cwd.or_else(|| std::env::current_dir().ok()),
            Err(e) => {
                let output = Output::step(self.path.clone(), "setup".into());
                return vec![Err(output.error(e))];
            }
        };
        let mut substitutions = substitutions.clone();
        if let Some(root) = fs_context.path() {
            substitutions.insert("[ROOT]", root.to_owned()).unwrap();
        }
        if let Some(cwd) = cwd.clone().or_else(|| std::env::current_dir().ok()) {
            substitutions.insert("[CWD]", cwd).unwrap();
        }
        substitutions
            .insert("[EXE]", std::env::consts::EXE_SUFFIX)
            .unwrap();
        snapbox::debug!("{:?}", substitutions);

        let mut outputs = Vec::with_capacity(sequence.steps.len());
        let mut prior_step_failed = false;
        for step in &mut sequence.steps {
            if prior_step_failed {
                step.expected_status = Some(crate::schema::CommandStatus::Skipped);
            }

            let step_status = self.run_step(step, cwd.as_deref(), bins, &substitutions);
            if fs_context.is_mutable() && step_status.is_err() && *mode == Mode::Fail {
                prior_step_failed = true;
            }
            outputs.push(step_status);
        }
        match mode {
            Mode::Dump(root) => {
                for output in &mut outputs {
                    let output = match output {
                        Ok(output) => output,
                        Err(output) => output,
                    };
                    output.stdout =
                        match self.dump_stream(root, output.id.as_deref(), output.stdout.take()) {
                            Ok(stream) => stream,
                            Err(stream) => stream,
                        };
                    output.stderr =
                        match self.dump_stream(root, output.id.as_deref(), output.stderr.take()) {
                            Ok(stream) => stream,
                            Err(stream) => stream,
                        };
                }
            }
            Mode::Overwrite => {
                // `rev()` to ensure we don't mess up our line number info
                for step_status in outputs.iter_mut().rev() {
                    if let Err(output) = step_status {
                        let res = sequence.overwrite(
                            &self.path,
                            output.id.as_deref(),
                            output.stdout.as_ref().map(|s| &s.content),
                            output.stderr.as_ref().map(|s| &s.content),
                            output.spawn.exit,
                        );

                        if res.is_ok() {
                            *step_status = Ok(output.clone());
                        }
                    }
                }
            }
            Mode::Fail => {}
        }

        if sequence.fs.sandbox() {
            let mut ok = true;
            let mut output = Output::step(self.path.clone(), "teardown".into());

            output.fs = match self.validate_fs(
                fs_context.path().expect("sandbox must be filled"),
                output.fs,
                mode,
                &substitutions,
            ) {
                Ok(fs) => fs,
                Err(fs) => {
                    ok = false;
                    fs
                }
            };
            if let Err(err) = fs_context.close() {
                ok = false;
                output.fs.context.push(FileStatus::Failure(
                    format!("Failed to cleanup sandbox: {err}").into(),
                ));
            }

            let output = if ok {
                output.spawn.status = SpawnStatus::Ok;
                Ok(output)
            } else {
                output.spawn.status = SpawnStatus::Failure("Files left in unexpected state".into());
                Err(output)
            };
            outputs.push(output);
        }

        outputs
    }

    #[allow(clippy::result_large_err)]
    pub(crate) fn run_step(
        &self,
        step: &mut crate::schema::Step,
        cwd: Option<&std::path::Path>,
        bins: &crate::BinRegistry,
        substitutions: &snapbox::Redactions,
    ) -> Result<Output, Output> {
        let output = if let Some(id) = step.id.clone() {
            Output::step(self.path.clone(), id)
        } else {
            Output::sequence(self.path.clone())
        };

        let mut bin = step.bin.take();
        if bin.is_none() {
            bin.clone_from(&self.default_bin);
        }
        bin = bin
            .map(|name| bins.resolve_bin(name))
            .transpose()
            .map_err(|e| output.clone().error(e))?;
        step.bin = bin;
        if step.timeout.is_none() {
            step.timeout = self.timeout;
        }
        if self.expected.is_some() {
            step.expected_status = self.expected;
        }
        step.env.update(&self.env);

        if step.expected_status() == crate::schema::CommandStatus::Skipped {
            assert_eq!(output.spawn.status, SpawnStatus::Skipped);
            return Ok(output);
        }

        match &step.bin {
            Some(crate::schema::Bin::Path(_)) => {}
            Some(crate::schema::Bin::Name(_name)) => {
                // Unhandled by resolve
                snapbox::debug!("bin={:?} not found", _name);
                assert_eq!(output.spawn.status, SpawnStatus::Skipped);
                return Ok(output);
            }
            Some(crate::schema::Bin::Error(_)) => {}
            // Unlike `Name`, this always represents a bug
            None => {}
            Some(crate::schema::Bin::Ignore) => {
                // Unhandled by resolve
                assert_eq!(output.spawn.status, SpawnStatus::Skipped);
                return Ok(output);
            }
        }

        let cmd = step.to_command(cwd).map_err(|e| output.clone().error(e))?;
        let timer = std::time::Instant::now();
        let cmd_output = cmd
            .output()
            .map_err(|e| output.clone().error(e.to_string().into()))?;

        let output = output.output(cmd_output);
        let output = output.duration(timer.elapsed());

        // For Mode::Dump's sake, allow running all
        let output = self.validate_spawn(output, step.expected_status());
        let output = self.validate_streams(output, step, substitutions);

        if output.is_ok() {
            Ok(output)
        } else {
            Err(output)
        }
    }

    fn validate_spawn(&self, mut output: Output, expected: crate::schema::CommandStatus) -> Output {
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

        output
    }

    fn validate_streams(
        &self,
        mut output: Output,
        step: &crate::schema::Step,
        substitutions: &snapbox::Redactions,
    ) -> Output {
        output.stdout = self.validate_stream(
            output.stdout,
            step.expected_stdout.as_ref(),
            step.binary,
            substitutions,
        );
        output.stderr = self.validate_stream(
            output.stderr,
            step.expected_stderr.as_ref(),
            step.binary,
            substitutions,
        );

        output
    }

    fn validate_stream(
        &self,
        stream: Option<Stream>,
        expected_content: Option<&crate::Data>,
        binary: bool,
        substitutions: &snapbox::Redactions,
    ) -> Option<Stream> {
        let mut stream = stream?;

        if !binary {
            stream = stream.make_text();
            if !stream.is_ok() {
                return Some(stream);
            }
        }

        if let Some(expected_content) = expected_content {
            stream.content = NormalizeToExpected::new()
                .redact_with(substitutions)
                .normalize(stream.content, expected_content);

            if stream.content != *expected_content {
                stream.status = StreamStatus::Expected(expected_content.clone());
                return Some(stream);
            }
        }

        Some(stream)
    }

    fn dump_stream(
        &self,
        root: &std::path::Path,
        id: Option<&str>,
        stream: Option<Stream>,
    ) -> Result<Option<Stream>, Option<Stream>> {
        if let Some(stream) = stream {
            let file_name = match id {
                Some(id) => {
                    format!(
                        "{}-{}.{}",
                        self.path.file_stem().unwrap().to_string_lossy(),
                        id,
                        stream.stream.as_str(),
                    )
                }
                None => {
                    format!(
                        "{}.{}",
                        self.path.file_stem().unwrap().to_string_lossy(),
                        stream.stream.as_str(),
                    )
                }
            };
            let stream_path = root.join(file_name);
            stream.content.write_to_path(&stream_path).map_err(|e| {
                let mut stream = stream.clone();
                if stream.is_ok() {
                    stream.status = StreamStatus::Failure(e);
                }
                stream
            })?;
            Ok(Some(stream))
        } else {
            Ok(None)
        }
    }

    fn validate_fs(
        &self,
        actual_root: &std::path::Path,
        mut fs: Filesystem,
        mode: &Mode,
        substitutions: &snapbox::Redactions,
    ) -> Result<Filesystem, Filesystem> {
        let mut ok = true;

        #[cfg(feature = "filesystem")]
        if let Mode::Dump(_) = mode {
            // Handled as part of DirRoot
        } else {
            let fixture_root = self.path.with_extension("out");
            if fixture_root.exists() {
                for status in snapbox::dir::PathDiff::subset_matches_iter(
                    fixture_root,
                    actual_root,
                    substitutions,
                ) {
                    match status {
                        Ok((expected_path, actual_path)) => {
                            fs.context.push(FileStatus::Ok {
                                actual_path,
                                expected_path,
                            });
                        }
                        Err(diff) => {
                            let mut is_current_ok = false;
                            if *mode == Mode::Overwrite && diff.overwrite().is_ok() {
                                is_current_ok = true;
                            }
                            fs.context.push(diff.into());
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Output {
    path: std::path::PathBuf,
    id: Option<String>,
    spawn: Spawn,
    stdout: Option<Stream>,
    stderr: Option<Stream>,
    fs: Filesystem,
    duration: Option<std::time::Duration>,
}

impl Output {
    fn sequence(path: std::path::PathBuf) -> Self {
        Self {
            path,
            id: None,
            spawn: Spawn {
                exit: None,
                status: SpawnStatus::Skipped,
            },
            stdout: None,
            stderr: None,
            fs: Default::default(),
            duration: Default::default(),
        }
    }

    fn step(path: std::path::PathBuf, step: String) -> Self {
        Self {
            path,
            id: Some(step),
            spawn: Default::default(),
            stdout: None,
            stderr: None,
            fs: Default::default(),
            duration: Default::default(),
        }
    }

    fn output(mut self, output: std::process::Output) -> Self {
        self.spawn.exit = Some(output.status);
        assert_eq!(self.spawn.status, SpawnStatus::Skipped);
        self.spawn.status = SpawnStatus::Ok;
        self.stdout = Some(Stream {
            stream: Stdio::Stdout,
            content: output.stdout.into_data(),
            status: StreamStatus::Ok,
        });
        self.stderr = Some(Stream {
            stream: Stdio::Stderr,
            content: output.stderr.into_data(),
            status: StreamStatus::Ok,
        });
        self
    }

    fn error(mut self, msg: crate::Error) -> Self {
        self.spawn.status = SpawnStatus::Failure(msg);
        self
    }

    fn duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    fn is_ok(&self) -> bool {
        self.spawn.is_ok()
            && self.stdout.as_ref().map(|s| s.is_ok()).unwrap_or(true)
            && self.stderr.as_ref().map(|s| s.is_ok()).unwrap_or(true)
            && self.fs.is_ok()
    }

    fn name(&self) -> String {
        self.id
            .as_deref()
            .map(|id| format!("{}:{}", self.path.display(), id))
            .unwrap_or_else(|| self.path.display().to_string())
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

impl Default for Spawn {
    fn default() -> Self {
        Self {
            exit: None,
            status: SpawnStatus::Skipped,
        }
    }
}

impl std::fmt::Display for Spawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = snapbox::report::Palette::color();

        match &self.status {
            SpawnStatus::Ok => {
                if let Some(exit) = self.exit {
                    if exit.success() {
                        writeln!(f, "Exit: {}", palette.info("success"))?;
                    } else if let Some(code) = exit.code() {
                        writeln!(f, "Exit: {}", palette.error(code))?;
                    } else {
                        writeln!(f, "Exit: {}", palette.error("interrupted"))?;
                    }
                }
            }
            SpawnStatus::Skipped => {
                writeln!(f, "{}", palette.warn("Skipped"))?;
            }
            SpawnStatus::Failure(msg) => {
                writeln!(f, "Failed: {}", palette.error(msg))?;
            }
            SpawnStatus::Expected(expected) => {
                if let Some(exit) = self.exit {
                    if exit.success() {
                        writeln!(
                            f,
                            "Expected {}, was {}",
                            palette.info(expected),
                            palette.error("success")
                        )?;
                    } else {
                        writeln!(
                            f,
                            "Expected {}, was {}",
                            palette.info(expected),
                            palette.error(snapbox::cmd::display_exit_status(exit))
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
    Failure(crate::Error),
    Expected(String),
}

impl SpawnStatus {
    fn is_ok(&self) -> bool {
        match self {
            Self::Ok | Self::Skipped => true,
            Self::Failure(_) | Self::Expected(_) => false,
        }
    }

    fn summary(&self) -> impl std::fmt::Display {
        let palette = snapbox::report::Palette::color();
        match self {
            Self::Ok => palette.info("ok"),
            Self::Skipped => palette.warn("ignored"),
            Self::Failure(_) | Self::Expected(_) => palette.error("failed"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Stream {
    stream: Stdio,
    content: crate::Data,
    status: StreamStatus,
}

impl Stream {
    fn make_text(mut self) -> Self {
        let content = self.content.coerce_to(DataFormat::Text);
        if content.format() != DataFormat::Text {
            self.status = StreamStatus::Failure("Unable to convert underlying Data to Text".into());
        }
        self.content = FilterNewlines.filter(FilterPaths.filter(content));
        self
    }

    fn is_ok(&self) -> bool {
        self.status.is_ok()
    }
}

impl std::fmt::Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = snapbox::report::Palette::color();

        match &self.status {
            StreamStatus::Ok => {
                writeln!(f, "{}:", self.stream)?;
                writeln!(f, "{}", palette.info(&self.content))?;
            }
            StreamStatus::Failure(msg) => {
                writeln!(
                    f,
                    "{} {}:",
                    self.stream,
                    palette.error(format_args!("({msg})"))
                )?;
                writeln!(f, "{}", palette.info(&self.content))?;
            }
            StreamStatus::Expected(expected) => {
                snapbox::report::write_diff(
                    f,
                    expected,
                    &self.content,
                    Some(&self.stream),
                    Some(&self.stream),
                    palette,
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StreamStatus {
    Ok,
    Failure(crate::Error),
    Expected(crate::Data),
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
    Failure(crate::Error),
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
        expected_content: crate::Data,
        actual_content: crate::Data,
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

impl From<snapbox::dir::PathDiff> for FileStatus {
    fn from(other: snapbox::dir::PathDiff) -> Self {
        match other {
            snapbox::dir::PathDiff::Failure(err) => FileStatus::Failure(err),
            snapbox::dir::PathDiff::TypeMismatch {
                expected_path,
                actual_path,
                expected_type,
                actual_type,
            } => FileStatus::TypeMismatch {
                actual_path,
                expected_path,
                actual_type,
                expected_type,
            },
            snapbox::dir::PathDiff::LinkMismatch {
                expected_path,
                actual_path,
                expected_target,
                actual_target,
            } => FileStatus::LinkMismatch {
                actual_path,
                expected_path,
                actual_target,
                expected_target,
            },
            snapbox::dir::PathDiff::ContentMismatch {
                expected_path,
                actual_path,
                expected_content,
                actual_content,
            } => FileStatus::ContentMismatch {
                actual_path,
                expected_path,
                actual_content,
                expected_content,
            },
        }
    }
}

impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let palette = snapbox::report::Palette::color();

        match &self {
            Self::Ok {
                expected_path,
                actual_path: _actual_path,
            } => {
                writeln!(
                    f,
                    "{}: is {}",
                    expected_path.display(),
                    palette.info("good"),
                )?;
            }
            Self::Failure(msg) => {
                writeln!(f, "{}", palette.error(msg))?;
            }
            Self::TypeMismatch {
                expected_path,
                actual_path: _actual_path,
                expected_type,
                actual_type,
            } => {
                writeln!(
                    f,
                    "{}: Expected {}, was {}",
                    expected_path.display(),
                    palette.info(expected_type),
                    palette.error(actual_type)
                )?;
            }
            Self::LinkMismatch {
                expected_path,
                actual_path: _actual_path,
                expected_target,
                actual_target,
            } => {
                writeln!(
                    f,
                    "{}: Expected {}, was {}",
                    expected_path.display(),
                    palette.info(expected_target.display()),
                    palette.error(actual_target.display())
                )?;
            }
            Self::ContentMismatch {
                expected_path,
                actual_path,
                expected_content,
                actual_content,
            } => {
                snapbox::report::write_diff(
                    f,
                    expected_content,
                    actual_content,
                    Some(&expected_path.display()),
                    Some(&actual_path.display()),
                    palette,
                )?;
            }
        }

        Ok(())
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

#[cfg_attr(not(feature = "filesystem"), allow(unused_variables))]
fn fs_context(
    path: &std::path::Path,
    cwd: Option<&std::path::Path>,
    sandbox: bool,
    mode: &Mode,
) -> Result<snapbox::dir::DirRoot, crate::Error> {
    if sandbox {
        #[cfg(feature = "filesystem")]
        match mode {
            Mode::Dump(root) => {
                let target = root.join(path.with_extension("out").file_name().unwrap());
                let mut context = snapbox::dir::DirRoot::mutable_at(&target)?;
                if let Some(cwd) = cwd {
                    context = context.with_template(cwd)?;
                }
                Ok(context)
            }
            Mode::Fail | Mode::Overwrite => {
                let mut context = snapbox::dir::DirRoot::mutable_temp()?;
                if let Some(cwd) = cwd {
                    context = context.with_template(cwd)?;
                }
                Ok(context)
            }
        }
        #[cfg(not(feature = "filesystem"))]
        Err("Sandboxing is disabled".into())
    } else {
        Ok(cwd
            .map(snapbox::dir::DirRoot::immutable)
            .unwrap_or_else(snapbox::dir::DirRoot::none))
    }
}
