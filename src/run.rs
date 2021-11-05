use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TryCmd {
    pub(crate) bin: Option<Bin>,
    pub(crate) args: Option<Vec<String>>,
    pub(crate) cwd: Option<std::path::PathBuf>,
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

        if let Some(cwd) = run.cwd.take() {
            run.cwd = Some(
                path.parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join(cwd),
            );
        }

        Ok(run)
    }

    pub(crate) fn to_command(&self) -> Result<std::process::Command, String> {
        let bin = self.bin()?;
        if !bin.exists() {
            return Err(format!("Bin doesn't exist: {}", bin.display()));
        }

        let mut cmd = std::process::Command::new(bin);
        if let Some(args) = self.args.as_deref() {
            cmd.args(args);
        }
        if let Some(cwd) = self.cwd.as_deref() {
            cmd.current_dir(cwd);
        }
        self.env.apply(&mut cmd);

        Ok(cmd)
    }

    pub(crate) fn to_output(&self, stdin: Option<Vec<u8>>) -> Result<std::process::Output, String> {
        let mut cmd = self.to_command()?;
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let child = cmd.spawn().map_err(|e| e.to_string())?;
        crate::wait_with_input_output(child, stdin, self.timeout).map_err(|e| e.to_string())
    }

    pub(crate) fn bin(&self) -> Result<std::path::PathBuf, String> {
        match &self.bin {
            Some(Bin::Path(path)) => Ok(path.clone()),
            Some(Bin::Name(name)) => Ok(crate::cargo_bin(name)),
            None => Err(String::from("No bin specified")),
        }
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
        let args = iter.collect();
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

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Env {
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Bin {
    Path(std::path::PathBuf),
    Name(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CommandStatus {
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
            args: Some(vec![]),
            ..Default::default()
        };
        let actual = TryCmd::parse_trycmd("cmd").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_trycmd_command_line() {
        let expected = TryCmd {
            bin: Some(Bin::Name("cmd".into())),
            args: Some(vec!["arg1".into(), "arg with space".into()]),
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
