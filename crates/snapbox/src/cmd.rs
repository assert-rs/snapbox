pub use snapbox_macros::cargo_bin;

/// Process spawning for testing of non-interactive commands
#[derive(Debug)]
pub struct Command {
    cmd: std::process::Command,
    stdin: Option<crate::Data>,
    timeout: Option<std::time::Duration>,
    _stderr_to_stdout: bool,
}

impl Command {
    pub fn new(program: impl AsRef<std::ffi::OsStr>) -> Self {
        Self {
            cmd: std::process::Command::new(program),
            stdin: None,
            timeout: None,
            _stderr_to_stdout: false,
        }
    }

    /// Constructs a new `Command` from a `std` `Command`.
    pub fn from_std(cmd: std::process::Command) -> Self {
        Self {
            cmd,
            stdin: None,
            timeout: None,
            _stderr_to_stdout: false,
        }
    }

    pub fn arg(mut self, arg: impl AsRef<std::ffi::OsStr>) -> Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Self {
        self.cmd.args(args);
        self
    }

    pub fn env(
        mut self,
        key: impl AsRef<std::ffi::OsStr>,
        value: impl AsRef<std::ffi::OsStr>,
    ) -> Self {
        self.cmd.env(key, value);
        self
    }

    pub fn envs(
        mut self,
        vars: impl IntoIterator<Item = (impl AsRef<std::ffi::OsStr>, impl AsRef<std::ffi::OsStr>)>,
    ) -> Self {
        self.cmd.envs(vars);
        self
    }

    pub fn env_remove(mut self, key: impl AsRef<std::ffi::OsStr>) -> Self {
        self.cmd.env_remove(key);
        self
    }

    pub fn env_clear(mut self) -> Self {
        self.cmd.env_clear();
        self
    }

    pub fn current_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.cmd.current_dir(dir);
        self
    }

    pub fn stdin(mut self, stream: impl Into<crate::Data>) -> Self {
        self.stdin = Some(stream.into());
        self
    }

    #[cfg(feature = "cmd")]
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[cfg(feature = "cmd")]
    pub fn stderr_to_stdout(mut self) -> Self {
        self._stderr_to_stdout = true;
        self
    }

    #[track_caller]
    pub fn assert(self) -> OutputAssert {
        match self.output() {
            Ok(output) => OutputAssert::new(output),
            Err(err) => {
                panic!("Failed to spawn: {}", err)
            }
        }
    }

    #[cfg(feature = "cmd")]
    pub fn output(self) -> Result<std::process::Output, std::io::Error> {
        if self._stderr_to_stdout {
            self.single_output()
        } else {
            self.split_output()
        }
    }

    #[cfg(not(feature = "cmd"))]
    pub fn output(self) -> Result<std::process::Output, std::io::Error> {
        self.split_output()
    }

    #[cfg(feature = "cmd")]
    fn single_output(mut self) -> Result<std::process::Output, std::io::Error> {
        use std::io::Read;

        self.cmd.stdin(std::process::Stdio::piped());
        let (mut reader, writer) = os_pipe::pipe()?;
        let writer_clone = writer.try_clone()?;
        self.cmd.stdout(writer);
        self.cmd.stderr(writer_clone);
        let mut child = self.cmd.spawn()?;
        // Avoid a deadlock! This parent process is still holding open pipe
        // writers (inside the Command object), and we have to close those
        // before we read. Here we do this by dropping the Command object.
        drop(self.cmd);

        let (stdout, stderr) = process_io(&mut child, self.stdin.as_ref().map(|d| d.as_bytes()))?;

        let status = wait(child, self.timeout)?;
        let _stdout = stdout
            .and_then(|t| t.join().unwrap().ok())
            .unwrap_or_default();
        let _stderr = stderr
            .and_then(|t| t.join().unwrap().ok())
            .unwrap_or_default();

        let mut stdout = Vec::new();
        reader.read_to_end(&mut stdout)?;
        Ok(std::process::Output {
            status,
            stdout,
            stderr: Default::default(),
        })
    }

    fn split_output(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.stdin(std::process::Stdio::piped());
        self.cmd.stdout(std::process::Stdio::piped());
        self.cmd.stderr(std::process::Stdio::piped());
        let mut child = self.cmd.spawn()?;

        let (stdout, stderr) = process_io(&mut child, self.stdin.as_ref().map(|d| d.as_bytes()))?;

        let status = wait(child, self.timeout)?;
        let stdout = stdout
            .and_then(|t| t.join().unwrap().ok())
            .unwrap_or_default();
        let stderr = stderr
            .and_then(|t| t.join().unwrap().ok())
            .unwrap_or_default();

        Ok(std::process::Output {
            status,
            stdout,
            stderr,
        })
    }
}

fn process_io(
    child: &mut std::process::Child,
    input: Option<&[u8]>,
) -> std::io::Result<(Stream, Stream)> {
    use std::io::Write;

    let input = input.map(|b| b.to_owned());
    let stdin = input.and_then(|i| {
        child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&i)))
    });
    fn read<R>(mut input: R) -> std::thread::JoinHandle<std::io::Result<Vec<u8>>>
    where
        R: std::io::Read + Send + 'static,
    {
        std::thread::spawn(move || {
            let mut ret = Vec::new();
            input.read_to_end(&mut ret).map(|_| ret)
        })
    }
    let stdout = child.stdout.take().map(read);
    let stderr = child.stderr.take().map(read);

    // Finish writing stdin before waiting, because waiting drops stdin.
    stdin.and_then(|t| t.join().unwrap().ok());

    Ok((stdout, stderr))
}

impl From<std::process::Command> for Command {
    fn from(cmd: std::process::Command) -> Self {
        Self::from_std(cmd)
    }
}

/// Assert the state of a [`Command`][cmd::Command]'s [`Output`].
///
/// Create an `OutputAssert` through the [`Command::assert`].
///
/// [`Output`]: std::process::Output
pub struct OutputAssert {
    output: std::process::Output,
    config: crate::Assert,
}

impl OutputAssert {
    /// Create an `Assert` for a given [`Output`].
    ///
    /// [`Output`]: std::process::Output
    pub fn new(output: std::process::Output) -> Self {
        Self {
            output,
            config: crate::Assert::new(),
        }
    }

    /// Customize the assertion behavior
    pub fn with_assert(mut self, config: crate::Assert) -> Self {
        self.config = config;
        self
    }

    /// Access the contained [`Output`].
    ///
    /// [`Output`]: std::process::Output
    pub fn get_output(&self) -> &std::process::Output {
        &self.output
    }

    /// Ensure the command succeeded.
    #[track_caller]
    pub fn success(self) -> Self {
        if !self.output.status.success() {
            let desc = format!(
                "Expected {}, was {}",
                self.config.palette.info("success"),
                self.config.palette.error(display_code(&self.output))
            );

            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command failed.
    #[track_caller]
    pub fn failure(self) -> Self {
        if self.output.status.success() {
            let desc = format!(
                "Expected {}, was {}",
                self.config.palette.info("failure"),
                self.config.palette.error("success")
            );

            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command aborted before returning a code.
    #[track_caller]
    pub fn interrupted(self) -> Self {
        if self.output.status.code().is_some() {
            let desc = format!(
                "Expected {}, was {}",
                self.config.palette.info("interrupted"),
                self.config.palette.error(display_code(&self.output))
            );

            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command returned the expected code.
    #[track_caller]
    pub fn code(self, expected: i32) -> Self {
        if self.output.status.code() != Some(expected) {
            let desc = format!(
                "Expected {}, was {}",
                self.config.palette.info(expected),
                self.config.palette.error(display_code(&self.output))
            );

            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command wrote the expected data to `stdout`.
    #[track_caller]
    pub fn stdout_eq(self, expected: impl Into<crate::Data>) -> Self {
        let expected = expected.into();
        self.stdout_eq_inner(expected)
    }

    #[track_caller]
    fn stdout_eq_inner(self, expected: crate::Data) -> Self {
        let actual = crate::Data::from(self.output.stdout.as_slice());
        let (actual, pattern) = self.config.normalize_eq(actual, Ok(expected));
        if let Err(desc) = pattern.and_then(|p| {
            self.config
                .try_verify(&actual, &p, Some(&"stdout"), Some(&"stdout"))
        }) {
            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_status(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }

        self
    }

    /// Ensure the command wrote the expected data to `stdout`.
    #[track_caller]
    pub fn stdout_eq_path(self, expected_path: impl AsRef<std::path::Path>) -> Self {
        let expected_path = expected_path.as_ref();
        self.stdout_eq_path_inner(expected_path)
    }

    #[track_caller]
    fn stdout_eq_path_inner(self, expected_path: &std::path::Path) -> Self {
        let actual = crate::Data::from(self.output.stdout.as_slice());
        let expected = crate::Data::read_from(expected_path, self.config.binary);
        let (actual, pattern) = self.config.normalize_eq(actual, expected);
        self.config.do_action(
            actual,
            pattern,
            expected_path,
            Some(&"stdout"),
            Some(&expected_path.display()),
        );

        self
    }

    /// Ensure the command wrote the expected data to `stdout`.
    #[track_caller]
    pub fn stdout_matches(self, expected: impl Into<crate::Data>) -> Self {
        let expected = expected.into();
        self.stdout_matches_inner(expected)
    }

    #[track_caller]
    fn stdout_matches_inner(self, expected: crate::Data) -> Self {
        let actual = crate::Data::from(self.output.stdout.as_slice());
        let (actual, pattern) = self.config.normalize_match(actual, Ok(expected));
        if let Err(desc) = pattern.and_then(|p| {
            self.config
                .try_verify(&actual, &p, Some(&"stdout"), Some(&"stdout"))
        }) {
            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_status(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }

        self
    }

    /// Ensure the command wrote the expected data to `stdout`.
    #[track_caller]
    pub fn stdout_matches_path(self, expected_path: impl AsRef<std::path::Path>) -> Self {
        let expected_path = expected_path.as_ref();
        self.stdout_matches_path_inner(expected_path)
    }

    #[track_caller]
    fn stdout_matches_path_inner(self, expected_path: &std::path::Path) -> Self {
        let actual = crate::Data::from(self.output.stdout.as_slice());
        let expected = crate::Data::read_from(expected_path, self.config.binary);
        let (actual, pattern) = self.config.normalize_match(actual, expected);
        self.config.do_action(
            actual,
            pattern,
            expected_path,
            Some(&"stdout"),
            Some(&expected_path.display()),
        );

        self
    }

    /// Ensure the command wrote the expected data to `stderr`.
    #[track_caller]
    pub fn stderr_eq(self, expected: impl Into<crate::Data>) -> Self {
        let expected = expected.into();
        self.stderr_eq_inner(expected)
    }

    #[track_caller]
    fn stderr_eq_inner(self, expected: crate::Data) -> Self {
        let actual = crate::Data::from(self.output.stderr.as_slice());
        let (actual, pattern) = self.config.normalize_eq(actual, Ok(expected));
        if let Err(desc) = pattern.and_then(|p| {
            self.config
                .try_verify(&actual, &p, Some(&"stderr"), Some(&"stderr"))
        }) {
            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_status(&mut buf).unwrap();
            self.write_stdout(&mut buf).unwrap();
            panic!("{}", buf);
        }

        self
    }

    /// Ensure the command wrote the expected data to `stderr`.
    #[track_caller]
    pub fn stderr_eq_path(self, expected_path: impl AsRef<std::path::Path>) -> Self {
        let expected_path = expected_path.as_ref();
        self.stderr_eq_path_inner(expected_path)
    }

    #[track_caller]
    fn stderr_eq_path_inner(self, expected_path: &std::path::Path) -> Self {
        let actual = crate::Data::from(self.output.stderr.as_slice());
        let expected = crate::Data::read_from(expected_path, self.config.binary);
        let (actual, pattern) = self.config.normalize_eq(actual, expected);
        self.config.do_action(
            actual,
            pattern,
            expected_path,
            Some(&"stderr"),
            Some(&expected_path.display()),
        );

        self
    }

    /// Ensure the command wrote the expected data to `stderr`.
    #[track_caller]
    pub fn stderr_matches(self, expected: impl Into<crate::Data>) -> Self {
        let expected = expected.into();
        self.stderr_matches_inner(expected)
    }

    #[track_caller]
    fn stderr_matches_inner(self, expected: crate::Data) -> Self {
        let actual = crate::Data::from(self.output.stderr.as_slice());
        let (actual, pattern) = self.config.normalize_match(actual, Ok(expected));
        if let Err(desc) = pattern.and_then(|p| {
            self.config
                .try_verify(&actual, &p, Some(&"stderr"), Some(&"stderr"))
        }) {
            use std::fmt::Write;
            let mut buf = String::new();
            write!(&mut buf, "{}", desc).unwrap();
            self.write_status(&mut buf).unwrap();
            self.write_stdout(&mut buf).unwrap();
            panic!("{}", buf);
        }

        self
    }

    /// Ensure the command wrote the expected data to `stderr`.
    #[track_caller]
    pub fn stderr_matches_path(self, expected_path: impl AsRef<std::path::Path>) -> Self {
        let expected_path = expected_path.as_ref();
        self.stderr_matches_path_inner(expected_path)
    }

    #[track_caller]
    fn stderr_matches_path_inner(self, expected_path: &std::path::Path) -> Self {
        let actual = crate::Data::from(self.output.stderr.as_slice());
        let expected = crate::Data::read_from(expected_path, self.config.binary);
        let (actual, pattern) = self.config.normalize_match(actual, expected);
        self.config.do_action(
            actual,
            pattern,
            expected_path,
            Some(&"stderr"),
            Some(&expected_path.display()),
        );

        self
    }

    fn write_status(&self, writer: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        writeln!(writer, "Exit status: {}", display_code(&self.output))?;
        Ok(())
    }

    fn write_stdout(&self, writer: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        if !self.output.stdout.is_empty() {
            writeln!(writer, "stdout:")?;
            writeln!(writer, "```")?;
            writeln!(writer, "{}", String::from_utf8_lossy(&self.output.stdout))?;
            writeln!(writer, "```")?;
        }
        Ok(())
    }

    fn write_stderr(&self, writer: &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
        if !self.output.stderr.is_empty() {
            writeln!(writer, "stderr:")?;
            writeln!(writer, "```")?;
            writeln!(writer, "{}", String::from_utf8_lossy(&self.output.stderr))?;
            writeln!(writer, "```")?;
        }
        Ok(())
    }
}

fn display_code(output: &std::process::Output) -> String {
    if let Some(code) = output.status.code() {
        code.to_string()
    } else {
        "interrupted".to_owned()
    }
}

type Stream = Option<std::thread::JoinHandle<Result<Vec<u8>, std::io::Error>>>;

#[cfg(feature = "cmd")]
fn wait(
    mut child: std::process::Child,
    timeout: Option<std::time::Duration>,
) -> std::io::Result<std::process::ExitStatus> {
    if let Some(timeout) = timeout {
        wait_timeout::ChildExt::wait_timeout(&mut child, timeout)
            .transpose()
            .unwrap_or_else(|| {
                let _ = child.kill();
                child.wait()
            })
    } else {
        child.wait()
    }
}

#[cfg(not(feature = "cmd"))]
fn wait(
    mut child: std::process::Child,
    _timeout: Option<std::time::Duration>,
) -> std::io::Result<std::process::ExitStatus> {
    child.wait()
}
