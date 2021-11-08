/// `cmd.toml` Schema
///
/// [`TryCmd`] is the top-level item in the `cmd.toml` files.
use std::collections::BTreeMap;

/// Top-level data in `cmd.toml` files
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TryCmd {
    pub(crate) bin: Option<Bin>,
    pub(crate) args: Option<Args>,
    #[serde(default)]
    pub(crate) fs: Filesystem,
    #[serde(default)]
    pub(crate) env: Env,
    pub(crate) status: Option<CommandStatus>,
    #[serde(default)]
    pub(crate) binary: bool,
    #[serde(default)]
    #[serde(deserialize_with = "humantime_serde::deserialize")]
    pub(crate) timeout: Option<std::time::Duration>,
}

impl TryCmd {
    pub(crate) fn load(path: &std::path::Path) -> Result<Self, String> {
        let mut run = if let Some(ext) = path.extension() {
            if ext == std::ffi::OsStr::new("toml") {
                let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
                Self::parse_toml(&raw)
            } else if ext == std::ffi::OsStr::new("trycmd") {
                let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
                Self::parse_trycmd(&raw)
            } else {
                Err(format!("Unsupported extension: {}", ext.to_string_lossy()))
            }
        } else {
            Err("No extension".into())
        }?;

        if let Some(cwd) = run.fs.cwd.take() {
            run.fs.cwd = Some(
                path.parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join(cwd),
            );
        }
        if run.fs.base.is_none() {
            let base_path = path.with_extension("in");
            if base_path.exists() {
                run.fs.base = Some(base_path);
            } else if run.fs.cwd.is_some() {
                run.fs.base = run.fs.cwd.clone();
            }
        }
        if run.fs.cwd.is_none() {
            run.fs.cwd = run.fs.base.clone();
        }
        if run.fs.sandbox.is_none() {
            run.fs.sandbox = Some(path.with_extension("out").exists());
        }

        Ok(run)
    }

    pub(crate) fn to_command(
        &self,
        base: Option<&std::path::Path>,
    ) -> Result<std::process::Command, String> {
        let bin = match &self.bin {
            Some(Bin::Path(path)) => Ok(path.clone()),
            Some(Bin::Name(name)) => Err(format!("Unknown bin.name = {}", name)),
            Some(Bin::Error(err)) => Err(err.clone().into_string()),
            None => Err(String::from("No bin specified")),
        }?;
        if !bin.exists() {
            return Err(format!("Bin doesn't exist: {}", bin.display()));
        }

        let mut cmd = std::process::Command::new(bin);
        if let Some(args) = self.args.as_deref() {
            cmd.args(args);
        }
        if let Some(base) = base {
            let base = if let (Some(orig_cwd), Some(orig_base)) =
                (self.fs.cwd.as_deref(), self.fs.base.as_deref())
            {
                let rel_cwd = orig_cwd.strip_prefix(orig_base).map_err(|_| {
                    format!(
                        "fs.cwd ({}) must be within fs.base ({})",
                        orig_cwd.display(),
                        orig_base.display()
                    )
                })?;
                base.join(rel_cwd)
            } else {
                base.to_owned()
            };
            cmd.current_dir(base);
        }
        self.env.apply(&mut cmd);

        Ok(cmd)
    }

    pub(crate) fn to_output(
        &self,
        stdin: Option<Vec<u8>>,
        cwd: Option<&std::path::Path>,
    ) -> Result<std::process::Output, String> {
        let mut cmd = self.to_command(cwd)?;
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let child = cmd.spawn().map_err(|e| e.to_string())?;
        crate::wait_with_input_output(child, stdin, self.timeout).map_err(|e| e.to_string())
    }

    pub(crate) fn status(&self) -> CommandStatus {
        self.status.unwrap_or_default()
    }

    fn parse_toml(s: &str) -> Result<Self, String> {
        toml_edit::de::from_str(s).map_err(|e| e.to_string())
    }

    fn parse_trycmd(s: &str) -> Result<Self, String> {
        let mut iter = shlex::Shlex::new(s.trim());
        let bin = iter
            .next()
            .ok_or_else(|| String::from("No bin specified"))?;
        let args = Args::Split(iter.collect());
        Ok(Self {
            bin: Some(Bin::Name(bin)),
            args: Some(args),
            ..Default::default()
        })
    }
}

impl std::str::FromStr for TryCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_trycmd(s)
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

/// Describe command's the filesystem context
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

    pub(crate) fn apply(&self, command: &mut std::process::Command) {
        if !self.inherit() {
            command.env_clear();
        }
        for remove in &self.remove {
            command.env_remove(&remove);
        }
        command.envs(&self.add);
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
pub enum CommandStatus {
    Pass,
    Fail,
    Interrupted,
    Skip,
    Code(i32),
}

impl Default for CommandStatus {
    fn default() -> Self {
        CommandStatus::Pass
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_trycmd_command() {
        let expected = TryCmd {
            bin: Some(Bin::Name("cmd".into())),
            args: Some(Args::default()),
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd("cmd").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_command_line() {
        let expected = TryCmd {
            bin: Some(Bin::Name("cmd".into())),
            args: Some(Args::Split(vec!["arg1".into(), "arg with space".into()])),
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd("cmd arg1 'arg with space'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_minimal() {
        let expected = TryCmd {
            ..Default::default()
        };
        let actual = TryCmd::parse_toml("").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_minimal_env() {
        let expected = TryCmd {
            ..Default::default()
        };
        let actual = TryCmd::parse_toml("[env]").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_bin_name() {
        let expected = TryCmd {
            bin: Some(Bin::Name("cmd".into())),
            ..Default::default()
        };
        let actual = TryCmd::parse_toml("bin.name = 'cmd'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_bin_path() {
        let expected = TryCmd {
            bin: Some(Bin::Path("/usr/bin/cmd".into())),
            ..Default::default()
        };
        let actual = TryCmd::parse_toml("bin.path = '/usr/bin/cmd'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_args_split() {
        let expected = TryCmd {
            args: Some(Args::Split(vec!["arg1".into(), "arg with space".into()])),
            ..Default::default()
        };
        let actual = TryCmd::parse_toml(r#"args = ["arg1", "arg with space"]"#).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_args_joined() {
        let expected = TryCmd {
            args: Some(Args::Joined(JoinedArgs::from_vec(vec![
                "arg1".into(),
                "arg with space".into(),
            ]))),
            ..Default::default()
        };
        let actual = TryCmd::parse_toml(r#"args = "arg1 'arg with space'""#).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_status_success() {
        let expected = TryCmd {
            status: Some(CommandStatus::Pass),
            ..Default::default()
        };
        let actual = TryCmd::parse_toml("status = 'pass'").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_toml_status_code() {
        let expected = TryCmd {
            status: Some(CommandStatus::Code(42)),
            ..Default::default()
        };
        let actual = TryCmd::parse_toml("status.code = 42").unwrap();
        assert_eq!(expected, actual);
    }
}
