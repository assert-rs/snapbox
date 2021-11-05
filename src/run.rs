use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct TryCmd {
    pub(crate) bin: Option<Bin>,
    pub(crate) args: Option<Vec<String>>,
    pub(crate) env: Option<Env>,
    pub(crate) status: Option<CommandStatus>,
    pub(crate) binary: bool,
    #[serde(deserialize_with = "humantime_serde::deserialize")]
    pub(crate) timeout: Option<std::time::Duration>,
}

impl TryCmd {
    pub(crate) fn load(path: &std::path::Path) -> Result<Self, String> {
        if let Some(ext) = path.extension() {
            if ext == std::ffi::OsStr::new("toml") {
                let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
                toml::from_str(&raw).map_err(|e| e.to_string())
            } else if ext == std::ffi::OsStr::new("trycmd") {
                let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
                raw.parse()
            } else {
                Err(format!("Unsupported extension: {}", ext.to_string_lossy()))
            }
        } else {
            Err("No extension".into())
        }
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
        if let Some(env) = self.env.as_ref() {
            env.apply(&mut cmd);
        }

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
}

impl std::str::FromStr for TryCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct Env {
    #[serde(default = "inherit_default")]
    pub(crate) inherit: bool,
    pub(crate) add: BTreeMap<String, String>,
    pub(crate) remove: Vec<String>,
}

impl Env {
    pub(crate) fn apply(&self, command: &mut std::process::Command) {
        if !self.inherit {
            command.env_clear();
        }
        for remove in &self.remove {
            command.env_remove(&remove);
        }
        command.envs(&self.add);
    }
}

fn inherit_default() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) enum Bin {
    Path(std::path::PathBuf),
    Name(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize)]
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
