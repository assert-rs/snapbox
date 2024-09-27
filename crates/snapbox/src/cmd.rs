//! Run commands and assert on their behavior

#[cfg(feature = "color")]
use anstream::panic;

use crate::IntoData;

/// Process spawning for testing of non-interactive commands
#[derive(Debug)]
pub struct Command {
    cmd: std::process::Command,
    stdin: Option<crate::Data>,
    timeout: Option<std::time::Duration>,
    _stderr_to_stdout: bool,
    config: crate::Assert,
}

/// # Builder API
impl Command {
    pub fn new(program: impl AsRef<std::ffi::OsStr>) -> Self {
        Self {
            cmd: std::process::Command::new(program),
            stdin: None,
            timeout: None,
            _stderr_to_stdout: false,
            config: crate::Assert::new().action_env(crate::assert::DEFAULT_ACTION_ENV),
        }
    }

    /// Constructs a new `Command` from a `std` `Command`.
    pub fn from_std(cmd: std::process::Command) -> Self {
        Self {
            cmd,
            stdin: None,
            timeout: None,
            _stderr_to_stdout: false,
            config: crate::Assert::new().action_env(crate::assert::DEFAULT_ACTION_ENV),
        }
    }

    /// Customize the assertion behavior
    pub fn with_assert(mut self, config: crate::Assert) -> Self {
        self.config = config;
        self
    }

    /// Adds an argument to pass to the program.
    ///
    /// Only one argument can be passed per use. So instead of:
    ///
    /// ```no_run
    /// # snapbox::cmd::Command::new("sh")
    /// .arg("-C /path/to/repo")
    /// # ;
    /// ```
    ///
    /// usage would be:
    ///
    /// ```no_run
    /// # snapbox::cmd::Command::new("sh")
    /// .arg("-C")
    /// .arg("/path/to/repo")
    /// # ;
    /// ```
    ///
    /// To pass multiple arguments see [`args`].
    ///
    /// [`args`]: Command::args()
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    ///
    /// Command::new("ls")
    ///         .arg("-l")
    ///         .arg("-a")
    ///         .assert()
    ///         .success();
    /// ```
    pub fn arg(mut self, arg: impl AsRef<std::ffi::OsStr>) -> Self {
        self.cmd.arg(arg);
        self
    }

    /// Adds multiple arguments to pass to the program.
    ///
    /// To pass a single argument see [`arg`].
    ///
    /// [`arg`]: Command::arg()
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    ///
    /// Command::new("ls")
    ///         .args(&["-l", "-a"])
    ///         .assert()
    ///         .success();
    /// ```
    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Self {
        self.cmd.args(args);
        self
    }

    /// Inserts or updates an environment variable mapping.
    ///
    /// Note that environment variable names are case-insensitive (but case-preserving) on Windows,
    /// and case-sensitive on all other platforms.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    ///
    /// Command::new("ls")
    ///         .env("PATH", "/bin")
    ///         .assert()
    ///         .failure();
    /// ```
    pub fn env(
        mut self,
        key: impl AsRef<std::ffi::OsStr>,
        value: impl AsRef<std::ffi::OsStr>,
    ) -> Self {
        self.cmd.env(key, value);
        self
    }

    /// Adds or updates multiple environment variable mappings.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    /// use std::process::Stdio;
    /// use std::env;
    /// use std::collections::HashMap;
    ///
    /// let filtered_env : HashMap<String, String> =
    ///     env::vars().filter(|&(ref k, _)|
    ///         k == "TERM" || k == "TZ" || k == "LANG" || k == "PATH"
    ///     ).collect();
    ///
    /// Command::new("printenv")
    ///         .env_clear()
    ///         .envs(&filtered_env)
    ///         .assert()
    ///         .success();
    /// ```
    pub fn envs(
        mut self,
        vars: impl IntoIterator<Item = (impl AsRef<std::ffi::OsStr>, impl AsRef<std::ffi::OsStr>)>,
    ) -> Self {
        self.cmd.envs(vars);
        self
    }

    /// Removes an environment variable mapping.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    ///
    /// Command::new("ls")
    ///         .env_remove("PATH")
    ///         .assert()
    ///         .failure();
    /// ```
    pub fn env_remove(mut self, key: impl AsRef<std::ffi::OsStr>) -> Self {
        self.cmd.env_remove(key);
        self
    }

    /// Clears the entire environment map for the child process.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    ///
    /// Command::new("ls")
    ///         .env_clear()
    ///         .assert()
    ///         .failure();
    /// ```
    pub fn env_clear(mut self) -> Self {
        self.cmd.env_clear();
        self
    }

    /// Sets the working directory for the child process.
    ///
    /// # Platform-specific behavior
    ///
    /// If the program path is relative (e.g., `"./script.sh"`), it's ambiguous
    /// whether it should be interpreted relative to the parent's working
    /// directory or relative to `current_dir`. The behavior in this case is
    /// platform specific and unstable, and it's recommended to use
    /// [`canonicalize`] to get an absolute program path instead.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use snapbox::cmd::Command;
    ///
    /// Command::new("ls")
    ///         .current_dir("/bin")
    ///         .assert()
    ///         .success();
    /// ```
    ///
    /// [`canonicalize`]: std::fs::canonicalize()
    pub fn current_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.cmd.current_dir(dir);
        self
    }

    /// Write `buffer` to `stdin` when the `Command` is run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use snapbox::cmd::Command;
    ///
    /// let mut cmd = Command::new("cat")
    ///     .arg("-et")
    ///     .stdin("42")
    ///     .assert()
    ///     .stdout_eq("42");
    /// ```
    pub fn stdin(mut self, stream: impl IntoData) -> Self {
        self.stdin = Some(stream.into_data());
        self
    }

    /// Error out if a timeout is reached
    ///
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .timeout(std::time::Duration::from_secs(1))
    ///     .env("sleep", "100")
    ///     .assert()
    ///     .failure();
    /// ```
    #[cfg(feature = "cmd")]
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Merge `stderr` into `stdout`
    #[cfg(feature = "cmd")]
    pub fn stderr_to_stdout(mut self) -> Self {
        self._stderr_to_stdout = true;
        self
    }
}

/// # Run Command
impl Command {
    /// Run the command and assert on the results
    ///
    /// ```rust
    /// use snapbox::cmd::Command;
    ///
    /// let mut cmd = Command::new("cat")
    ///     .arg("-et")
    ///     .stdin("42")
    ///     .assert()
    ///     .stdout_eq("42");
    /// ```
    #[track_caller]
    pub fn assert(self) -> OutputAssert {
        let config = self.config.clone();
        match self.output() {
            Ok(output) => OutputAssert::new(output).with_assert(config),
            Err(err) => {
                panic!("Failed to spawn: {}", err)
            }
        }
    }

    /// Run the command and capture the `Output`
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
        self.cmd.stdin(std::process::Stdio::piped());
        let (reader, writer) = os_pipe::pipe()?;
        let writer_clone = writer.try_clone()?;
        self.cmd.stdout(writer);
        self.cmd.stderr(writer_clone);
        let mut child = self.cmd.spawn()?;
        // Avoid a deadlock! This parent process is still holding open pipe
        // writers (inside the Command object), and we have to close those
        // before we read. Here we do this by dropping the Command object.
        drop(self.cmd);

        let stdin = self
            .stdin
            .as_ref()
            .map(|d| d.to_bytes())
            .transpose()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        let stdout = process_single_io(&mut child, reader, stdin)?;

        let status = wait(child, self.timeout)?;
        let stdout = stdout.join().unwrap().ok().unwrap_or_default();

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

        let stdin = self
            .stdin
            .as_ref()
            .map(|d| d.to_bytes())
            .transpose()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        let (stdout, stderr) = process_split_io(&mut child, stdin)?;

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

fn process_split_io(
    child: &mut std::process::Child,
    input: Option<Vec<u8>>,
) -> std::io::Result<(Option<Stream>, Option<Stream>)> {
    use std::io::Write;

    let stdin = input.and_then(|i| {
        child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&i)))
    });
    let stdout = child.stdout.take().map(threaded_read);
    let stderr = child.stderr.take().map(threaded_read);

    // Finish writing stdin before waiting, because waiting drops stdin.
    stdin.and_then(|t| t.join().unwrap().ok());

    Ok((stdout, stderr))
}

#[cfg(feature = "cmd")]
fn process_single_io(
    child: &mut std::process::Child,
    stdout: os_pipe::PipeReader,
    input: Option<Vec<u8>>,
) -> std::io::Result<Stream> {
    use std::io::Write;

    let stdin = input.and_then(|i| {
        child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&i)))
    });
    let stdout = threaded_read(stdout);
    debug_assert!(child.stdout.is_none());
    debug_assert!(child.stderr.is_none());

    // Finish writing stdin before waiting, because waiting drops stdin.
    stdin.and_then(|t| t.join().unwrap().ok());

    Ok(stdout)
}

type Stream = std::thread::JoinHandle<Result<Vec<u8>, std::io::Error>>;

fn threaded_read<R>(mut input: R) -> Stream
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut ret = Vec::new();
        input.read_to_end(&mut ret).map(|_| ret)
    })
}

impl From<std::process::Command> for Command {
    fn from(cmd: std::process::Command) -> Self {
        Self::from_std(cmd)
    }
}

/// Assert the state of a [`Command`]'s [`Output`].
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
            config: crate::Assert::new().action_env(crate::assert::DEFAULT_ACTION_ENV),
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
    ///
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .assert()
    ///     .success();
    /// ```
    #[track_caller]
    pub fn success(self) -> Self {
        if !self.output.status.success() {
            let desc = format!(
                "Expected {}, was {}",
                self.config.palette.info("success"),
                self.config
                    .palette
                    .error(display_exit_status(self.output.status))
            );

            use std::fmt::Write;
            let mut buf = String::new();
            writeln!(&mut buf, "{desc}").unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command failed.
    ///
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .env("exit", "1")
    ///     .assert()
    ///     .failure();
    /// ```
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
            writeln!(&mut buf, "{desc}").unwrap();
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
                self.config
                    .palette
                    .error(display_exit_status(self.output.status))
            );

            use std::fmt::Write;
            let mut buf = String::new();
            writeln!(&mut buf, "{desc}").unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command returned the expected code.
    ///
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .env("exit", "42")
    ///     .assert()
    ///     .code(42);
    /// ```
    #[track_caller]
    pub fn code(self, expected: i32) -> Self {
        if self.output.status.code() != Some(expected) {
            let desc = format!(
                "Expected {}, was {}",
                self.config.palette.info(expected),
                self.config
                    .palette
                    .error(display_exit_status(self.output.status))
            );

            use std::fmt::Write;
            let mut buf = String::new();
            writeln!(&mut buf, "{desc}").unwrap();
            self.write_stdout(&mut buf).unwrap();
            self.write_stderr(&mut buf).unwrap();
            panic!("{}", buf);
        }
        self
    }

    /// Ensure the command wrote the expected data to `stdout`.
    ///
    /// By default [`filters`][crate::filter] are applied, including:
    /// - `...` is a line-wildcard when on a line by itself
    /// - `[..]` is a character-wildcard when inside a line
    /// - `[EXE]` matches `.exe` on Windows
    /// - `"{...}"` is a JSON value wildcard
    /// - `"...": "{...}"` is a JSON key-value wildcard
    /// - `\` to `/`
    /// - Newlines
    ///
    /// To limit this to newline normalization for text, call [`Data::raw`][crate::Data::raw] on `expected`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .env("stdout", "hello")
    ///     .env("stderr", "world")
    ///     .assert()
    ///     .stdout_eq("he[..]o");
    /// ```
    ///
    /// Can combine this with [`file!`][crate::file]
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    /// use snapbox::file;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .env("stdout", "hello")
    ///     .env("stderr", "world")
    ///     .assert()
    ///     .stdout_eq(file!["stdout.log"]);
    /// ```
    #[track_caller]
    pub fn stdout_eq(self, expected: impl IntoData) -> Self {
        let expected = expected.into_data();
        self.stdout_eq_inner(expected)
    }

    #[track_caller]
    #[deprecated(since = "0.6.0", note = "Replaced with `OutputAssert::stdout_eq`")]
    pub fn stdout_eq_(self, expected: impl IntoData) -> Self {
        self.stdout_eq(expected)
    }

    #[track_caller]
    fn stdout_eq_inner(self, expected: crate::Data) -> Self {
        let actual = self.output.stdout.as_slice().into_data();
        if let Err(err) = self.config.try_eq(Some(&"stdout"), actual, expected) {
            err.panic();
        }

        self
    }

    /// Ensure the command wrote the expected data to `stderr`.
    ///
    /// By default [`filters`][crate::filter] are applied, including:
    /// - `...` is a line-wildcard when on a line by itself
    /// - `[..]` is a character-wildcard when inside a line
    /// - `[EXE]` matches `.exe` on Windows
    /// - `"{...}"` is a JSON value wildcard
    /// - `"...": "{...}"` is a JSON key-value wildcard
    /// - `\` to `/`
    /// - Newlines
    ///
    /// To limit this to newline normalization for text, call [`Data::raw`][crate::Data::raw] on `expected`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .env("stdout", "hello")
    ///     .env("stderr", "world")
    ///     .assert()
    ///     .stderr_eq("wo[..]d");
    /// ```
    ///
    /// Can combine this with [`file!`][crate::file]
    /// ```rust,no_run
    /// use snapbox::cmd::Command;
    /// use snapbox::cmd::cargo_bin;
    /// use snapbox::file;
    ///
    /// let assert = Command::new(cargo_bin("snap-fixture"))
    ///     .env("stdout", "hello")
    ///     .env("stderr", "world")
    ///     .assert()
    ///     .stderr_eq(file!["stderr.log"]);
    /// ```
    #[track_caller]
    pub fn stderr_eq(self, expected: impl IntoData) -> Self {
        let expected = expected.into_data();
        self.stderr_eq_inner(expected)
    }

    #[track_caller]
    #[deprecated(since = "0.6.0", note = "Replaced with `OutputAssert::stderr_eq`")]
    pub fn stderr_eq_(self, expected: impl IntoData) -> Self {
        self.stderr_eq(expected)
    }

    #[track_caller]
    fn stderr_eq_inner(self, expected: crate::Data) -> Self {
        let actual = self.output.stderr.as_slice().into_data();
        if let Err(err) = self.config.try_eq(Some(&"stderr"), actual, expected) {
            err.panic();
        }

        self
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

/// Converts an [`std::process::ExitStatus`]  to a human-readable value
#[cfg(not(feature = "cmd"))]
pub fn display_exit_status(status: std::process::ExitStatus) -> String {
    basic_exit_status(status)
}

/// Converts an [`std::process::ExitStatus`]  to a human-readable value
#[cfg(feature = "cmd")]
pub fn display_exit_status(status: std::process::ExitStatus) -> String {
    #[cfg(unix)]
    fn detailed_exit_status(status: std::process::ExitStatus) -> Option<String> {
        use std::os::unix::process::ExitStatusExt;

        let signal = status.signal()?;
        let name = match signal as libc::c_int {
            libc::SIGABRT => ", SIGABRT: process abort signal",
            libc::SIGALRM => ", SIGALRM: alarm clock",
            libc::SIGFPE => ", SIGFPE: erroneous arithmetic operation",
            libc::SIGHUP => ", SIGHUP: hangup",
            libc::SIGILL => ", SIGILL: illegal instruction",
            libc::SIGINT => ", SIGINT: terminal interrupt signal",
            libc::SIGKILL => ", SIGKILL: kill",
            libc::SIGPIPE => ", SIGPIPE: write on a pipe with no one to read",
            libc::SIGQUIT => ", SIGQUIT: terminal quit signal",
            libc::SIGSEGV => ", SIGSEGV: invalid memory reference",
            libc::SIGTERM => ", SIGTERM: termination signal",
            libc::SIGBUS => ", SIGBUS: access to undefined memory",
            #[cfg(not(target_os = "haiku"))]
            libc::SIGSYS => ", SIGSYS: bad system call",
            libc::SIGTRAP => ", SIGTRAP: trace/breakpoint trap",
            _ => "",
        };
        Some(format!("signal: {signal}{name}"))
    }

    #[cfg(windows)]
    fn detailed_exit_status(status: std::process::ExitStatus) -> Option<String> {
        use windows_sys::Win32::Foundation::*;

        let extra = match status.code().unwrap() as NTSTATUS {
            STATUS_ACCESS_VIOLATION => "STATUS_ACCESS_VIOLATION",
            STATUS_IN_PAGE_ERROR => "STATUS_IN_PAGE_ERROR",
            STATUS_INVALID_HANDLE => "STATUS_INVALID_HANDLE",
            STATUS_INVALID_PARAMETER => "STATUS_INVALID_PARAMETER",
            STATUS_NO_MEMORY => "STATUS_NO_MEMORY",
            STATUS_ILLEGAL_INSTRUCTION => "STATUS_ILLEGAL_INSTRUCTION",
            STATUS_NONCONTINUABLE_EXCEPTION => "STATUS_NONCONTINUABLE_EXCEPTION",
            STATUS_INVALID_DISPOSITION => "STATUS_INVALID_DISPOSITION",
            STATUS_ARRAY_BOUNDS_EXCEEDED => "STATUS_ARRAY_BOUNDS_EXCEEDED",
            STATUS_FLOAT_DENORMAL_OPERAND => "STATUS_FLOAT_DENORMAL_OPERAND",
            STATUS_FLOAT_DIVIDE_BY_ZERO => "STATUS_FLOAT_DIVIDE_BY_ZERO",
            STATUS_FLOAT_INEXACT_RESULT => "STATUS_FLOAT_INEXACT_RESULT",
            STATUS_FLOAT_INVALID_OPERATION => "STATUS_FLOAT_INVALID_OPERATION",
            STATUS_FLOAT_OVERFLOW => "STATUS_FLOAT_OVERFLOW",
            STATUS_FLOAT_STACK_CHECK => "STATUS_FLOAT_STACK_CHECK",
            STATUS_FLOAT_UNDERFLOW => "STATUS_FLOAT_UNDERFLOW",
            STATUS_INTEGER_DIVIDE_BY_ZERO => "STATUS_INTEGER_DIVIDE_BY_ZERO",
            STATUS_INTEGER_OVERFLOW => "STATUS_INTEGER_OVERFLOW",
            STATUS_PRIVILEGED_INSTRUCTION => "STATUS_PRIVILEGED_INSTRUCTION",
            STATUS_STACK_OVERFLOW => "STATUS_STACK_OVERFLOW",
            STATUS_DLL_NOT_FOUND => "STATUS_DLL_NOT_FOUND",
            STATUS_ORDINAL_NOT_FOUND => "STATUS_ORDINAL_NOT_FOUND",
            STATUS_ENTRYPOINT_NOT_FOUND => "STATUS_ENTRYPOINT_NOT_FOUND",
            STATUS_CONTROL_C_EXIT => "STATUS_CONTROL_C_EXIT",
            STATUS_DLL_INIT_FAILED => "STATUS_DLL_INIT_FAILED",
            STATUS_FLOAT_MULTIPLE_FAULTS => "STATUS_FLOAT_MULTIPLE_FAULTS",
            STATUS_FLOAT_MULTIPLE_TRAPS => "STATUS_FLOAT_MULTIPLE_TRAPS",
            STATUS_REG_NAT_CONSUMPTION => "STATUS_REG_NAT_CONSUMPTION",
            STATUS_HEAP_CORRUPTION => "STATUS_HEAP_CORRUPTION",
            STATUS_STACK_BUFFER_OVERRUN => "STATUS_STACK_BUFFER_OVERRUN",
            STATUS_ASSERTION_FAILURE => "STATUS_ASSERTION_FAILURE",
            _ => return None,
        };
        Some(extra.to_owned())
    }

    if let Some(extra) = detailed_exit_status(status) {
        format!("{} ({})", basic_exit_status(status), extra)
    } else {
        basic_exit_status(status)
    }
}

fn basic_exit_status(status: std::process::ExitStatus) -> String {
    if let Some(code) = status.code() {
        code.to_string()
    } else {
        "interrupted".to_owned()
    }
}

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

pub use snapbox_macros::cargo_bin;

/// Look up the path to a cargo-built binary within an integration test.
///
/// **NOTE:** Prefer [`cargo_bin!`] as this makes assumptions about cargo
pub fn cargo_bin(name: &str) -> std::path::PathBuf {
    let file_name = format!("{}{}", name, std::env::consts::EXE_SUFFIX);
    let target_dir = target_dir();
    target_dir.join(file_name)
}

// Adapted from
// https://github.com/rust-lang/cargo/blob/485670b3983b52289a2f353d589c57fae2f60f82/tests/testsuite/support/mod.rs#L507
fn target_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap()
}

#[cfg(feature = "examples")]
pub use examples::{compile_example, compile_examples};

#[cfg(feature = "examples")]
pub(crate) mod examples {
    /// Prepare an example for testing
    ///
    /// Unlike `cargo_bin!`, this does not inherit all of the current compiler settings.  It
    /// will match the current target and profile but will not get feature flags.  Pass those arguments
    /// to the compiler via `args`.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// snapbox::cmd::compile_example("snap-example-fixture", []);
    /// ```
    #[cfg(feature = "examples")]
    pub fn compile_example<'a>(
        target_name: &str,
        args: impl IntoIterator<Item = &'a str>,
    ) -> crate::assert::Result<std::path::PathBuf> {
        crate::debug!("Compiling example {}", target_name);
        let messages = escargot::CargoBuild::new()
            .current_target()
            .current_release()
            .example(target_name)
            .args(args)
            .exec()
            .map_err(|e| crate::assert::Error::new(e.to_string()))?;
        for message in messages {
            let message = message.map_err(|e| crate::assert::Error::new(e.to_string()))?;
            let message = message
                .decode()
                .map_err(|e| crate::assert::Error::new(e.to_string()))?;
            crate::debug!("Message: {:?}", message);
            if let Some(bin) = decode_example_message(&message) {
                let (name, bin) = bin?;
                assert_eq!(target_name, name);
                return bin;
            }
        }

        Err(crate::assert::Error::new(format!(
            "Unknown error building example {target_name}"
        )))
    }

    /// Prepare all examples for testing
    ///
    /// Unlike `cargo_bin!`, this does not inherit all of the current compiler settings.  It
    /// will match the current target and profile but will not get feature flags.  Pass those arguments
    /// to the compiler via `args`.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// let examples = snapbox::cmd::compile_examples([]).unwrap().collect::<Vec<_>>();
    /// ```
    #[cfg(feature = "examples")]
    pub fn compile_examples<'a>(
        args: impl IntoIterator<Item = &'a str>,
    ) -> crate::assert::Result<
        impl Iterator<Item = (String, crate::assert::Result<std::path::PathBuf>)>,
    > {
        crate::debug!("Compiling examples");
        let mut examples = std::collections::BTreeMap::new();

        let messages = escargot::CargoBuild::new()
            .current_target()
            .current_release()
            .examples()
            .args(args)
            .exec()
            .map_err(|e| crate::assert::Error::new(e.to_string()))?;
        for message in messages {
            let message = message.map_err(|e| crate::assert::Error::new(e.to_string()))?;
            let message = message
                .decode()
                .map_err(|e| crate::assert::Error::new(e.to_string()))?;
            crate::debug!("Message: {:?}", message);
            if let Some(bin) = decode_example_message(&message) {
                let (name, bin) = bin?;
                examples.insert(name.to_owned(), bin);
            }
        }

        Ok(examples.into_iter())
    }

    #[allow(clippy::type_complexity)]
    fn decode_example_message<'m>(
        message: &'m escargot::format::Message<'_>,
    ) -> Option<crate::assert::Result<(&'m str, crate::assert::Result<std::path::PathBuf>)>> {
        match message {
            escargot::format::Message::CompilerMessage(msg) => {
                let level = msg.message.level;
                if level == escargot::format::diagnostic::DiagnosticLevel::Ice
                    || level == escargot::format::diagnostic::DiagnosticLevel::Error
                {
                    let output = msg
                        .message
                        .rendered
                        .as_deref()
                        .unwrap_or_else(|| msg.message.message.as_ref())
                        .to_owned();
                    if is_example_target(&msg.target) {
                        let bin = Err(crate::assert::Error::new(output));
                        Some(Ok((msg.target.name.as_ref(), bin)))
                    } else {
                        Some(Err(crate::assert::Error::new(output)))
                    }
                } else {
                    None
                }
            }
            escargot::format::Message::CompilerArtifact(artifact) => {
                if !artifact.profile.test && is_example_target(&artifact.target) {
                    let path = artifact
                        .executable
                        .clone()
                        .expect("cargo is new enough for this to be present");
                    let bin = Ok(path.into_owned());
                    Some(Ok((artifact.target.name.as_ref(), bin)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_example_target(target: &escargot::format::Target<'_>) -> bool {
        target.crate_types == ["bin"] && target.kind == ["example"]
    }
}
