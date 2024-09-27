//! `cmd.toml` Schema
//!
//! [`OneShot`] is the top-level item in the `cmd.toml` files.

use snapbox::filter::{Filter as _, FilterNewlines, FilterPaths};
use std::collections::BTreeMap;
use std::collections::VecDeque;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(crate) struct TryCmd {
    pub(crate) steps: Vec<Step>,
    pub(crate) fs: Filesystem,
}

impl TryCmd {
    pub(crate) fn load(path: &std::path::Path) -> Result<Self, crate::Error> {
        let mut sequence = if let Some(ext) = path.extension() {
            if ext == std::ffi::OsStr::new("toml") {
                let raw = std::fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                let one_shot = OneShot::parse_toml(&raw)?;
                let mut sequence: Self = one_shot.into();
                let is_binary = match sequence.steps[0].binary {
                    true => snapbox::data::DataFormat::Binary,
                    false => snapbox::data::DataFormat::Text,
                };

                if sequence.steps[0].stdin.is_none() {
                    let stdin_path = path.with_extension("stdin");
                    let stdin = if stdin_path.exists() {
                        // No `map_text` as we will trust what the user inputted
                        Some(crate::Data::try_read_from(&stdin_path, Some(is_binary))?)
                    } else {
                        None
                    };
                    sequence.steps[0].stdin = stdin;
                }

                if sequence.steps[0].expected_stdout.is_none() {
                    let stdout_path = path.with_extension("stdout");
                    let stdout = if stdout_path.exists() {
                        Some(
                            FilterNewlines.filter(
                                FilterPaths
                                    .filter(crate::Data::read_from(&stdout_path, Some(is_binary))),
                            ),
                        )
                    } else {
                        None
                    };
                    sequence.steps[0].expected_stdout = stdout;
                }

                if sequence.steps[0].expected_stderr.is_none() {
                    let stderr_path = path.with_extension("stderr");
                    let stderr = if stderr_path.exists() {
                        Some(
                            FilterNewlines.filter(
                                FilterPaths
                                    .filter(crate::Data::read_from(&stderr_path, Some(is_binary))),
                            ),
                        )
                    } else {
                        None
                    };
                    sequence.steps[0].expected_stderr = stderr;
                }

                sequence
            } else if ext == std::ffi::OsStr::new("trycmd") || ext == std::ffi::OsStr::new("md") {
                let raw = std::fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                let normalized = snapbox::filter::normalize_lines(&raw);
                Self::parse_trycmd(&normalized)?
            } else {
                return Err(format!("Unsupported extension: {}", ext.to_string_lossy()).into());
            }
        } else {
            return Err("No extension".into());
        };

        sequence.fs.base = sequence.fs.base.take().map(|base| {
            path.parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(base)
        });
        sequence.fs.cwd = sequence.fs.cwd.take().map(|cwd| {
            path.parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(cwd)
        });

        if sequence.fs.base.is_none() {
            let base_path = path.with_extension("in");
            if base_path.exists() {
                sequence.fs.base = Some(base_path);
            } else if sequence.fs.cwd.is_some() {
                sequence.fs.base.clone_from(&sequence.fs.cwd);
            }
        }
        if sequence.fs.cwd.is_none() {
            sequence.fs.cwd.clone_from(&sequence.fs.base);
        }
        if sequence.fs.sandbox.is_none() {
            sequence.fs.sandbox = Some(path.with_extension("out").exists());
        }

        sequence.fs.base = sequence
            .fs
            .base
            .take()
            .map(|p| snapbox::dir::resolve_dir(p).map_err(|e| e.to_string()))
            .transpose()?;
        sequence.fs.cwd = sequence
            .fs
            .cwd
            .take()
            .map(|p| snapbox::dir::resolve_dir(p).map_err(|e| e.to_string()))
            .transpose()?;

        Ok(sequence)
    }

    pub(crate) fn overwrite(
        &self,
        path: &std::path::Path,
        id: Option<&str>,
        stdout: Option<&crate::Data>,
        stderr: Option<&crate::Data>,
        exit: Option<std::process::ExitStatus>,
    ) -> Result<(), crate::Error> {
        if let Some(ext) = path.extension() {
            if ext == std::ffi::OsStr::new("toml") {
                assert_eq!(id, None);

                overwrite_toml_output(path, id, stdout, "stdout", "stdout")?;
                overwrite_toml_output(path, id, stderr, "stderr", "stderr")?;

                if let Some(status) = exit {
                    let raw = std::fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    let overwritten = overwrite_toml_status(status, raw)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    std::fs::write(path, overwritten)
                        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
                }
            } else if ext == std::ffi::OsStr::new("trycmd") || ext == std::ffi::OsStr::new("md") {
                if stderr.is_some() && stderr != Some(&crate::Data::new()) {
                    panic!("stderr should have been merged: {stderr:?}");
                }
                if let (Some(id), Some(stdout)) = (id, stdout) {
                    let step = self
                        .steps
                        .iter()
                        .find(|s| s.id.as_deref() == Some(id))
                        .expect("id is valid");
                    let mut line_nums = step
                        .expected_stdout_source
                        .clone()
                        .expect("always present for .trycmd");

                    let raw = std::fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    let mut normalized = snapbox::filter::normalize_lines(&raw);

                    overwrite_trycmd_status(exit, step, &mut line_nums, &mut normalized)?;

                    let mut stdout = stdout.render().expect("at least Text");
                    // Add back trailing newline removed when parsing
                    stdout.push('\n');
                    replace_lines(&mut normalized, line_nums, &stdout)?;

                    std::fs::write(path, normalized.into_bytes())
                        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
                }
            } else {
                return Err(format!("Unsupported extension: {}", ext.to_string_lossy()).into());
            }
        } else {
            return Err("No extension".into());
        }

        Ok(())
    }

    fn parse_trycmd(s: &str) -> Result<Self, crate::Error> {
        let mut steps = Vec::new();

        let mut lines: VecDeque<_> = snapbox::utils::LinesWithTerminator::new(s)
            .enumerate()
            .map(|(i, l)| (i + 1, l))
            .collect();
        'outer: loop {
            let mut fence_pattern = "```".to_owned();
            while let Some((_, line)) = lines.pop_front() {
                let tick_end = line
                    .char_indices()
                    .find_map(|(i, c)| (c != '`').then_some(i))
                    .unwrap_or(line.len());
                if 3 <= tick_end {
                    line[..tick_end].clone_into(&mut fence_pattern);
                    let raw = line[tick_end..].trim();
                    if raw.is_empty() {
                        // Assuming a trycmd block
                        break;
                    } else {
                        let mut info = raw.split(',');
                        let lang = info.next().unwrap();
                        match lang {
                            "trycmd" | "console" => {
                                if info.any(|i| i == "ignore") {
                                    snapbox::debug!("ignore from infostring: {:?}", info);
                                } else {
                                    break;
                                }
                            }
                            _ => {
                                snapbox::debug!("ignore from lang: {:?}", lang);
                            }
                        }
                    }

                    // Irrelevant block, consume to end
                    while let Some((_, line)) = lines.pop_front() {
                        if line.starts_with(&fence_pattern) {
                            continue 'outer;
                        }
                    }
                }
            }

            'code: loop {
                let mut cmdline = Vec::new();
                let mut expected_status_source = None;
                let mut expected_status = Some(CommandStatus::Success);
                let mut stdout = String::new();
                let cmd_start;
                let mut stdout_start;

                if let Some((line_num, line)) = lines.pop_front() {
                    if line.starts_with(&fence_pattern) {
                        break;
                    } else if let Some(raw) = line.strip_prefix("$ ") {
                        cmdline.extend(shlex::Shlex::new(raw.trim()));
                        cmd_start = line_num;
                        stdout_start = line_num + 1;
                    } else {
                        return Err(format!("Expected `$` on line {line_num}, got `{line}`").into());
                    }
                } else {
                    break 'outer;
                }
                while let Some((line_num, line)) = lines.pop_front() {
                    if let Some(raw) = line.strip_prefix("> ") {
                        cmdline.extend(shlex::Shlex::new(raw.trim()));
                        stdout_start = line_num + 1;
                    } else {
                        lines.push_front((line_num, line));
                        break;
                    }
                }
                if let Some((line_num, line)) = lines.pop_front() {
                    if let Some(raw) = line.strip_prefix("? ") {
                        expected_status_source = Some(line_num);
                        expected_status = Some(raw.trim().parse::<CommandStatus>()?);
                        stdout_start = line_num + 1;
                    } else {
                        lines.push_front((line_num, line));
                    }
                }
                let mut post_stdout_start = stdout_start;
                let mut block_done = false;
                while let Some((line_num, line)) = lines.pop_front() {
                    if line.starts_with("$ ") {
                        lines.push_front((line_num, line));
                        post_stdout_start = line_num;
                        break;
                    } else if line.starts_with(&fence_pattern) {
                        block_done = true;
                        post_stdout_start = line_num;
                        break;
                    } else {
                        stdout.push_str(line);
                        post_stdout_start = line_num + 1;
                    }
                }
                if stdout.ends_with('\n') {
                    // Last newline is for formatting purposes so tests can verify cases without a
                    // trailing newline.
                    stdout.pop();
                }

                let mut env = Env::default();

                let bin = loop {
                    if cmdline.is_empty() {
                        return Err(format!("No bin specified on line {cmd_start}").into());
                    }
                    let next = cmdline.remove(0);
                    if let Some((key, value)) = next.split_once('=') {
                        env.add.insert(key.to_owned(), value.to_owned());
                    } else {
                        break next;
                    }
                };
                let step = Step {
                    id: Some(cmd_start.to_string()),
                    bin: Some(Bin::Name(bin)),
                    args: cmdline,
                    env,
                    stdin: None,
                    stderr_to_stdout: true,
                    expected_status_source,
                    expected_status,
                    expected_stdout_source: Some(stdout_start..post_stdout_start),
                    expected_stdout: Some(crate::Data::text(stdout)),
                    expected_stderr_source: None,
                    expected_stderr: None,
                    binary: false,
                    timeout: None,
                };
                steps.push(step);
                if block_done {
                    break 'code;
                }
            }
        }

        Ok(Self {
            steps,
            ..Default::default()
        })
    }
}

fn overwrite_toml_output(
    path: &std::path::Path,
    _id: Option<&str>,
    output: Option<&crate::Data>,
    output_ext: &str,
    output_field: &str,
) -> Result<(), crate::Error> {
    if let Some(output) = output {
        let output_path = path.with_extension(output_ext);
        if output_path.exists() {
            output.write_to_path(&output_path)?;
        } else if let Some(output) = output.render() {
            let raw = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let mut doc = raw
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            if let Some(output_value) = doc.get_mut(output_field) {
                *output_value = toml_edit::value(output);
            }
            std::fs::write(path, doc.to_string())
                .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        } else {
            output.write_to_path(&output_path)?;

            let raw = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let mut doc = raw
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            doc[output_field] = toml_edit::Item::None;
            std::fs::write(path, doc.to_string())
                .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        }
    }

    Ok(())
}

fn overwrite_toml_status(
    status: std::process::ExitStatus,
    raw: String,
) -> Result<String, toml_edit::TomlError> {
    let mut doc = raw.parse::<toml_edit::DocumentMut>()?;
    if let Some(code) = status.code() {
        if status.success() {
            match doc.get("status") {
                Some(toml_edit::Item::Value(toml_edit::Value::String(ref expected)))
                    if expected.value() == "success" => {}
                Some(
                    toml_edit::Item::Value(toml_edit::Value::InlineTable(_))
                    | toml_edit::Item::Table(_),
                ) => {
                    if !matches!(
                        doc["status"].get("code"),
                        Some(toml_edit::Item::Value(toml_edit::Value::Integer(ref expected)))
                            if expected.value() == &0)
                    {
                        // Remove `status` to use the default value (success)
                        doc["status"] = toml_edit::Item::None;
                    }
                }
                _ => {
                    // Remove `status` to use the default value (success)
                    doc["status"] = toml_edit::Item::None;
                }
            }
        } else {
            let code = code as i64;
            match doc.get("status") {
                Some(toml_edit::Item::Value(toml_edit::Value::String(ref expected))) => {
                    if expected.value() != "failed" {
                        doc["status"] = toml_edit::value("failed");
                    }
                }
                Some(
                    toml_edit::Item::Value(toml_edit::Value::InlineTable(_))
                    | toml_edit::Item::Table(_),
                ) => {
                    if !matches!(
                        doc["status"].get("code"),
                        Some(toml_edit::Item::Value(toml_edit::Value::Integer(ref expected)))
                            if expected.value() == &code)
                    {
                        doc["status"]["code"] = toml_edit::value(code);
                    }
                }
                _ => {
                    let mut status = toml_edit::InlineTable::default();
                    status.set_dotted(true);
                    status.insert("code", code.into());
                    doc["status"] = toml_edit::value(status);
                }
            }
        }
    } else if !matches!(
        doc.get("status"),
        Some(toml_edit::Item::Value(toml_edit::Value::String(ref expected)))
            if expected.value() == "interrupted")
    {
        doc["status"] = toml_edit::value("interrupted");
    }

    Ok(doc.to_string())
}

fn overwrite_trycmd_status(
    exit: Option<std::process::ExitStatus>,
    step: &Step,
    stdout_line_nums: &mut std::ops::Range<usize>,
    normalized: &mut String,
) -> Result<(), crate::Error> {
    let status = match exit {
        Some(status) => status,
        _ => {
            return Ok(());
        }
    };

    let formatted_status = if let Some(code) = status.code() {
        if status.success() {
            if let (true, Some(line_num)) = (
                step.expected_status != Some(CommandStatus::Success),
                step.expected_status_source,
            ) {
                replace_lines(normalized, line_num..(line_num + 1), "")?;
                *stdout_line_nums = (stdout_line_nums.start - 1)..(stdout_line_nums.end - 1);
            }
            None
        } else {
            match step.expected_status {
                Some(CommandStatus::Success | CommandStatus::Interrupted) => {
                    Some(format!("? {code}"))
                }
                Some(CommandStatus::Code(expected)) if expected != code => {
                    Some(format!("? {code}"))
                }
                _ => None,
            }
        }
    } else {
        if step.expected_status == Some(CommandStatus::Interrupted) {
            None
        } else {
            Some("? interrupted".into())
        }
    };

    if let Some(status) = formatted_status {
        if let Some(line_num) = step.expected_status_source {
            replace_lines(normalized, line_num..(line_num + 1), &status)?;
        } else {
            let line_num = stdout_line_nums.start;
            replace_lines(normalized, line_num..line_num, &status)?;
            *stdout_line_nums = (line_num + 1)..(stdout_line_nums.end + 1);
        }
    }

    Ok(())
}

/// Update an inline snapshot
fn replace_lines(
    data: &mut String,
    line_nums: std::ops::Range<usize>,
    text: &str,
) -> Result<(), crate::Error> {
    let mut output_lines = String::new();

    for (line_num, line) in snapbox::utils::LinesWithTerminator::new(data)
        .enumerate()
        .map(|(i, l)| (i + 1, l))
    {
        if line_num == line_nums.start {
            output_lines.push_str(text);
            if !text.is_empty() && !text.ends_with('\n') {
                output_lines.push('\n');
            }
        }
        if !line_nums.contains(&line_num) {
            output_lines.push_str(line);
        }
    }

    *data = output_lines;
    Ok(())
}

impl std::str::FromStr for TryCmd {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_trycmd(s)
    }
}

impl From<OneShot> for TryCmd {
    fn from(other: OneShot) -> Self {
        let OneShot {
            bin,
            args,
            env,
            stdin,
            stdout,
            stderr,
            stderr_to_stdout,
            status,
            binary,
            timeout,
            fs,
        } = other;
        Self {
            steps: vec![Step {
                id: None,
                bin,
                args: args.into_vec(),
                env,
                stdin: stdin.map(crate::Data::text),
                stderr_to_stdout,
                expected_status_source: None,
                expected_status: status,
                expected_stdout_source: None,
                expected_stdout: stdout.map(crate::Data::text),
                expected_stderr_source: None,
                expected_stderr: stderr.map(crate::Data::text),
                binary,
                timeout,
            }],
            fs,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(crate) struct Step {
    pub(crate) id: Option<String>,
    pub(crate) bin: Option<Bin>,
    pub(crate) args: Vec<String>,
    pub(crate) env: Env,
    pub(crate) stdin: Option<crate::Data>,
    pub(crate) stderr_to_stdout: bool,
    pub(crate) expected_status_source: Option<usize>,
    pub(crate) expected_status: Option<CommandStatus>,
    pub(crate) expected_stdout_source: Option<std::ops::Range<usize>>,
    pub(crate) expected_stdout: Option<crate::Data>,
    pub(crate) expected_stderr_source: Option<std::ops::Range<usize>>,
    pub(crate) expected_stderr: Option<crate::Data>,
    pub(crate) binary: bool,
    pub(crate) timeout: Option<std::time::Duration>,
}

impl Step {
    pub(crate) fn to_command(
        &self,
        cwd: Option<&std::path::Path>,
    ) -> Result<snapbox::cmd::Command, crate::Error> {
        let bin = match &self.bin {
            Some(Bin::Path(path)) => Ok(path.clone()),
            Some(Bin::Name(name)) => Err(format!("Unknown bin.name = {name}").into()),
            Some(Bin::Ignore) => Err("Internal error: tried to run an ignored bin".into()),
            Some(Bin::Error(err)) => Err(err.clone()),
            None => Err("No bin specified".into()),
        }?;
        if !bin.exists() {
            return Err(format!("Bin doesn't exist: {}", bin.display()).into());
        }

        let mut cmd = snapbox::cmd::Command::new(bin).args(&self.args);
        if let Some(cwd) = cwd {
            cmd = cmd.current_dir(cwd);
        }
        if let Some(stdin) = &self.stdin {
            cmd = cmd.stdin(stdin);
        }
        if self.stderr_to_stdout {
            cmd = cmd.stderr_to_stdout();
        }
        if let Some(timeout) = self.timeout {
            cmd = cmd.timeout(timeout);
        }
        cmd = self.env.apply(cmd);

        Ok(cmd)
    }

    pub(crate) fn expected_status(&self) -> CommandStatus {
        self.expected_status.unwrap_or_default()
    }
}

/// Top-level data in `cmd.toml` files
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OneShot {
    pub(crate) bin: Option<Bin>,
    #[serde(default)]
    pub(crate) args: Args,
    #[serde(default)]
    pub(crate) env: Env,
    #[serde(default)]
    pub(crate) stdin: Option<String>,
    #[serde(default)]
    pub(crate) stdout: Option<String>,
    #[serde(default)]
    pub(crate) stderr: Option<String>,
    #[serde(default)]
    pub(crate) stderr_to_stdout: bool,
    pub(crate) status: Option<CommandStatus>,
    #[serde(default)]
    pub(crate) binary: bool,
    #[serde(default)]
    #[serde(deserialize_with = "humantime_serde::deserialize")]
    pub(crate) timeout: Option<std::time::Duration>,
    #[serde(default)]
    pub(crate) fs: Filesystem,
}

impl OneShot {
    fn parse_toml(s: &str) -> Result<Self, crate::Error> {
        toml_edit::de::from_str(s).map_err(|e| e.to_string().into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub(crate) enum Args {
    Joined(JoinedArgs),
    Split(Vec<String>),
}

impl Args {
    fn new() -> Self {
        Self::Split(Default::default())
    }

    fn as_slice(&self) -> &[String] {
        match self {
            Self::Joined(j) => j.inner.as_slice(),
            Self::Split(v) => v.as_slice(),
        }
    }

    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Joined(j) => j.inner,
            Self::Split(v) => v,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for Args {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub(crate) struct JoinedArgs {
    inner: Vec<String>,
}

impl JoinedArgs {
    #[cfg(test)]
    pub(crate) fn from_vec(inner: Vec<String>) -> Self {
        JoinedArgs { inner }
    }

    #[allow(clippy::inherent_to_string_shadow_display)]
    fn to_string(&self) -> String {
        shlex::join(self.inner.iter().map(|s| s.as_str()))
    }
}

impl std::str::FromStr for JoinedArgs {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = shlex::Shlex::new(s).collect();
        Ok(Self { inner })
    }
}

impl std::fmt::Display for JoinedArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string().fmt(f)
    }
}

impl<'de> serde::de::Deserialize<'de> for JoinedArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        std::str::FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::ser::Serialize for JoinedArgs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Describe the command's filesystem context
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Filesystem {
    pub(crate) cwd: Option<std::path::PathBuf>,
    /// Sandbox base
    pub(crate) base: Option<std::path::PathBuf>,
    pub(crate) sandbox: Option<bool>,
}

impl Filesystem {
    pub(crate) fn sandbox(&self) -> bool {
        self.sandbox.unwrap_or_default()
    }

    pub(crate) fn rel_cwd(&self) -> Result<&std::path::Path, crate::Error> {
        if let (Some(orig_cwd), Some(orig_base)) = (self.cwd.as_deref(), self.base.as_deref()) {
            let rel_cwd = orig_cwd.strip_prefix(orig_base).map_err(|_| {
                crate::Error::new(format!(
                    "fs.cwd ({}) must be within fs.base ({})",
                    orig_cwd.display(),
                    orig_base.display()
                ))
            })?;
            Ok(rel_cwd)
        } else {
            Ok(std::path::Path::new(""))
        }
    }
}

/// Describe command's environment
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Env {
    #[serde(default)]
    pub(crate) inherit: Option<bool>,
    #[serde(default)]
    pub(crate) add: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) remove: Vec<String>,
}

impl Env {
    pub(crate) fn update(&mut self, other: &Self) {
        if self.inherit.is_none() {
            self.inherit = other.inherit;
        }
        self.add
            .extend(other.add.iter().map(|(k, v)| (k.clone(), v.clone())));
        self.remove.extend(other.remove.iter().cloned());
    }

    pub(crate) fn apply(&self, mut command: snapbox::cmd::Command) -> snapbox::cmd::Command {
        if !self.inherit() {
            command = command.env_clear();
        }
        for remove in &self.remove {
            command = command.env_remove(remove);
        }
        command.envs(&self.add)
    }

    pub(crate) fn inherit(&self) -> bool {
        self.inherit.unwrap_or(true)
    }
}

/// Target under test
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Bin {
    Path(std::path::PathBuf),
    Name(String),
    Ignore,
    #[serde(skip)]
    Error(crate::Error),
}

impl From<std::path::PathBuf> for Bin {
    fn from(other: std::path::PathBuf) -> Self {
        Self::Path(other)
    }
}

impl<'a> From<&'a std::path::PathBuf> for Bin {
    fn from(other: &'a std::path::PathBuf) -> Self {
        Self::Path(other.clone())
    }
}

impl<'a> From<&'a std::path::Path> for Bin {
    fn from(other: &'a std::path::Path) -> Self {
        Self::Path(other.to_owned())
    }
}

impl<P, E> From<Result<P, E>> for Bin
where
    P: Into<Bin>,
    E: std::fmt::Display,
{
    fn from(other: Result<P, E>) -> Self {
        match other {
            Ok(path) => path.into(),
            Err(err) => {
                let err = crate::Error::new(err.to_string());
                Bin::Error(err)
            }
        }
    }
}

/// Expected status for command
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Default)]
pub enum CommandStatus {
    #[default]
    Success,
    Failed,
    Interrupted,
    Skipped,
    Code(i32),
}

impl std::str::FromStr for CommandStatus {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            "interrupted" => Ok(Self::Interrupted),
            "skipped" => Ok(Self::Skipped),
            _ => s
                .parse::<i32>()
                .map(Self::Code)
                .map_err(|_| crate::Error::new(format!("Expected an exit code, got {s}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_trycmd_empty() {
        let expected = TryCmd {
            steps: vec![],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd("").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_empty_fence() {
        let expected = TryCmd {
            steps: vec![],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_command() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                expected_status: Some(CommandStatus::Success),
                stderr_to_stdout: true,
                expected_stdout_source: Some(4..4),
                expected_stdout: Some(crate::Data::new()),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_command_line() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                args: vec!["arg1".into(), "arg with space".into()],
                expected_status: Some(CommandStatus::Success),
                stderr_to_stdout: true,
                expected_stdout_source: Some(4..4),
                expected_stdout: Some(crate::Data::new()),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd arg1 'arg with space'
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_multi_line() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                args: vec!["arg1".into(), "arg with space".into()],
                expected_status: Some(CommandStatus::Success),
                stderr_to_stdout: true,
                expected_stdout_source: Some(5..5),
                expected_stdout: Some(crate::Data::new()),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd arg1
> 'arg with space'
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_env() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                env: Env {
                    add: IntoIterator::into_iter([
                        ("KEY1".into(), "VALUE1".into()),
                        ("KEY2".into(), "VALUE2 with space".into()),
                    ])
                    .collect(),
                    ..Default::default()
                },
                expected_status: Some(CommandStatus::Success),
                stderr_to_stdout: true,
                expected_stdout_source: Some(4..4),
                expected_stdout: Some(crate::Data::new()),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ KEY1=VALUE1 KEY2='VALUE2 with space' cmd
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_status() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                expected_status_source: Some(4),
                expected_status: Some(CommandStatus::Skipped),
                stderr_to_stdout: true,
                expected_stdout_source: Some(5..5),
                expected_stdout: Some(crate::Data::new()),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd
? skipped
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_status_code() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                expected_status_source: Some(4),
                expected_status: Some(CommandStatus::Code(-1)),
                stderr_to_stdout: true,
                expected_stdout_source: Some(5..5),
                expected_stdout: Some(crate::Data::new()),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd
? -1
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_stdout() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                expected_status: Some(CommandStatus::Success),
                stderr_to_stdout: true,
                expected_stdout_source: Some(4..6),
                expected_stdout: Some(crate::Data::text("Hello World\n")),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd
Hello World

```",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_escaped_stdout() {
        let expected = TryCmd {
            steps: vec![Step {
                id: Some("3".into()),
                bin: Some(Bin::Name("cmd".into())),
                expected_status: Some(CommandStatus::Success),
                stderr_to_stdout: true,
                expected_stdout_source: Some(4..7),
                expected_stdout: Some(crate::Data::text("```\nHello World\n```")),
                expected_stderr: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
````
$ cmd
```
Hello World
```
````",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_multi_step() {
        let expected = TryCmd {
            steps: vec![
                Step {
                    id: Some("3".into()),
                    bin: Some(Bin::Name("cmd1".into())),
                    expected_status_source: Some(4),
                    expected_status: Some(CommandStatus::Code(1)),
                    stderr_to_stdout: true,
                    expected_stdout_source: Some(5..5),
                    expected_stdout: Some(crate::Data::new()),
                    expected_stderr: None,
                    ..Default::default()
                },
                Step {
                    id: Some("5".into()),
                    bin: Some(Bin::Name("cmd2".into())),
                    expected_status: Some(CommandStatus::Success),
                    stderr_to_stdout: true,
                    expected_stdout_source: Some(6..6),
                    expected_stdout: Some(crate::Data::new()),
                    expected_stderr: None,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ cmd1
? 1
$ cmd2
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_info_string() {
        let expected = TryCmd {
            steps: vec![
                Step {
                    id: Some("3".into()),
                    bin: Some(Bin::Name("bare-cmd".into())),
                    expected_status_source: Some(4),
                    expected_status: Some(CommandStatus::Code(1)),
                    stderr_to_stdout: true,
                    expected_stdout_source: Some(5..5),
                    expected_stdout: Some(crate::Data::new()),
                    expected_stderr: None,
                    ..Default::default()
                },
                Step {
                    id: Some("8".into()),
                    bin: Some(Bin::Name("trycmd-cmd".into())),
                    expected_status_source: Some(9),
                    expected_status: Some(CommandStatus::Code(1)),
                    stderr_to_stdout: true,
                    expected_stdout_source: Some(10..10),
                    expected_stdout: Some(crate::Data::new()),
                    expected_stderr: None,
                    ..Default::default()
                },
                Step {
                    id: Some("18".into()),
                    bin: Some(Bin::Name("console-cmd".into())),
                    expected_status_source: Some(19),
                    expected_status: Some(CommandStatus::Code(1)),
                    stderr_to_stdout: true,
                    expected_stdout_source: Some(20..20),
                    expected_stdout: Some(crate::Data::new()),
                    expected_stderr: None,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd(
            "
```
$ bare-cmd
? 1
```

```trycmd
$ trycmd-cmd
? 1
```

```sh
$ sh-cmd
? 1
```

```console
$ console-cmd
? 1
```

```ignore
$ rust-cmd1
? 1
```

```trycmd,ignore
$ rust-cmd1
? 1
```

```rust
$ rust-cmd1
? 1
```
",
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_minimal() {
        let expected = OneShot {
            ..Default::default()
        };
        let actual = OneShot::parse_toml("").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_minimal_env() {
        let expected = OneShot {
            ..Default::default()
        };
        let actual = OneShot::parse_toml("[env]").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_bin_name() {
        let expected = OneShot {
            bin: Some(Bin::Name("cmd".into())),
            ..Default::default()
        };
        let actual = OneShot::parse_toml("bin.name = 'cmd'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_bin_path() {
        let expected = OneShot {
            bin: Some(Bin::Path("/usr/bin/cmd".into())),
            ..Default::default()
        };
        let actual = OneShot::parse_toml("bin.path = '/usr/bin/cmd'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_args_split() {
        let expected = OneShot {
            args: Args::Split(vec!["arg1".into(), "arg with space".into()]),
            ..Default::default()
        };
        let actual = OneShot::parse_toml(r#"args = ["arg1", "arg with space"]"#).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_args_joined() {
        let expected = OneShot {
            args: Args::Joined(JoinedArgs::from_vec(vec![
                "arg1".into(),
                "arg with space".into(),
            ])),
            ..Default::default()
        };
        let actual = OneShot::parse_toml(r#"args = "arg1 'arg with space'""#).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_status_success() {
        let expected = OneShot {
            status: Some(CommandStatus::Success),
            ..Default::default()
        };
        let actual = OneShot::parse_toml("status = 'success'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_status_code() {
        let expected = OneShot {
            status: Some(CommandStatus::Code(42)),
            ..Default::default()
        };
        let actual = OneShot::parse_toml("status.code = 42").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_same_line_count() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World\n";
        let expected = "One\nWorld\nThree";

        let mut actual = input.to_owned();
        replace_lines(&mut actual, line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_grow() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World\nTrees\n";
        let expected = "One\nWorld\nTrees\nThree";

        let mut actual = input.to_owned();
        replace_lines(&mut actual, line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_shrink() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "";
        let expected = "One\nThree";

        let mut actual = input.to_owned();
        replace_lines(&mut actual, line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_no_trailing() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World";
        let expected = "One\nWorld\nThree";

        let mut actual = input.to_owned();
        replace_lines(&mut actual, line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_empty_range() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..2;
        let replacement = "World\n";
        let expected = "One\nWorld\nTwo\nThree";

        let mut actual = input.to_owned();
        replace_lines(&mut actual, line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn overwrite_toml_status_success() {
        let expected = r#"
bin.name = "cmd"
"#;
        let actual = overwrite_toml_status(
            exit_code_to_status(0),
            r#"
bin.name = "cmd"
status = "failed"
"#
            .into(),
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn overwrite_toml_status_failed() {
        let expected = r#"
bin.name = "cmd"
status.code = 1
"#;
        let actual = overwrite_toml_status(
            exit_code_to_status(1),
            r#"
bin.name = "cmd"
"#
            .into(),
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn overwrite_toml_status_keeps_style() {
        let expected = r#"
bin.name = "cmd"
status = { code = 1 } # comment
"#;
        let actual = overwrite_toml_status(
            exit_code_to_status(1),
            r#"
bin.name = "cmd"
status = { code = 2 } # comment
"#
            .into(),
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn overwrite_trycmd_status_success() {
        let expected = r#"
```
$ cmd arg
foo
bar
```
"#;

        let mut actual = r"
```
$ cmd arg
? failed
foo
bar
```
"
        .to_owned();

        let step = &TryCmd::parse_trycmd(&actual).unwrap().steps[0];
        overwrite_trycmd_status(
            Some(exit_code_to_status(0)),
            step,
            &mut step.expected_stdout_source.clone().unwrap(),
            &mut actual,
        )
        .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn overwrite_trycmd_status_failed() {
        let expected = r#"
```
$ cmd arg
? 1
foo
bar
```
"#;

        let mut actual = r"
```
$ cmd arg
? 2
foo
bar
```
"
        .to_owned();

        let step = &TryCmd::parse_trycmd(&actual).unwrap().steps[0];
        overwrite_trycmd_status(
            Some(exit_code_to_status(1)),
            step,
            &mut step.expected_stdout_source.clone().unwrap(),
            &mut actual,
        )
        .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn overwrite_trycmd_status_keeps_style() {
        let expected = r#"
```
$ cmd arg
? success
foo
bar
```
"#;

        let mut actual = r"
```
$ cmd arg
? success
foo
bar
```
"
        .to_owned();

        let step = &TryCmd::parse_trycmd(&actual).unwrap().steps[0];
        overwrite_trycmd_status(
            Some(exit_code_to_status(0)),
            step,
            &mut step.expected_stdout_source.clone().unwrap(),
            &mut actual,
        )
        .unwrap();

        assert_eq!(expected, actual);
    }

    #[cfg(unix)]
    fn exit_code_to_status(code: u8) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw((code as i32) << 8)
    }

    #[cfg(windows)]
    fn exit_code_to_status(code: u8) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code as u32)
    }

    #[test]
    fn exit_code_to_status_works() {
        assert_eq!(exit_code_to_status(42).code(), Some(42));
    }
}
