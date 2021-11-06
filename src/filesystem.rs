pub(crate) enum FilesystemContext {
    Default,
    Path(std::path::PathBuf),
    #[cfg(feature = "filesystem")]
    Temp(tempfile::TempDir),
}

impl FilesystemContext {
    #[cfg_attr(not(feature = "filesystem"), allow(unused_variables))]
    pub(crate) fn new(
        path: &std::path::Path,
        cwd: Option<&std::path::Path>,
        sandbox: bool,
        mode: &crate::Mode,
    ) -> Result<Self, std::io::Error> {
        if sandbox {
            #[cfg(feature = "filesystem")]
            match mode {
                crate::Mode::Dump(root) => {
                    let target = root.join(path.with_extension("out").file_name().unwrap());
                    let _ = std::fs::remove_dir_all(&target);
                    std::fs::create_dir_all(&target)?;
                    if let Some(cwd) = cwd {
                        copy_dir(cwd, &target)?;
                    }
                    Ok(Self::Path(target))
                }
                crate::Mode::Fail | crate::Mode::Overwrite => {
                    let temp = tempfile::tempdir()?;
                    if let Some(cwd) = cwd {
                        copy_dir(cwd, temp.path())?;
                    }
                    Ok(Self::Temp(temp))
                }
            }
            #[cfg(not(feature = "filesystem"))]
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "sandboxing is disabled",
            ))
        } else {
            Ok(cwd.map(|p| Self::Path(p.to_owned())).unwrap_or_default())
        }
    }

    pub(crate) fn path(&self) -> Option<&std::path::Path> {
        match self {
            Self::Default => None,
            Self::Path(path) => Some(path.as_path()),
            #[cfg(feature = "filesystem")]
            Self::Temp(temp) => Some(temp.path()),
        }
    }

    pub(crate) fn close(self) -> Result<(), std::io::Error> {
        match self {
            Self::Default | Self::Path(_) => Ok(()),
            #[cfg(feature = "filesystem")]
            Self::Temp(temp) => temp.close(),
        }
    }
}

impl Default for FilesystemContext {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(feature = "filesystem")]
fn copy_dir(source: &std::path::Path, dest: &std::path::Path) -> Result<(), std::io::Error> {
    let source = source.canonicalize()?;
    let dest = dest.canonicalize()?;

    for entry in walkdir::WalkDir::new(&source) {
        let entry = entry?;
        let rel_dest = entry.path().strip_prefix(&source).unwrap();
        let dest = dest.join(rel_dest);
        let source = entry.path();

        let meta = source.symlink_metadata()?;
        if meta.is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if meta.is_file() {
            std::fs::copy(source, &dest)?;
        } else if let Ok(target) = std::fs::read_link(source) {
            symlink_to_file(&dest, &target)?;
        }
    }

    Ok(())
}

#[cfg(feature = "filesystem")]
#[cfg(windows)]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(feature = "filesystem")]
#[cfg(not(windows))]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(target, link)
}
