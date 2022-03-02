#[derive(Debug)]
pub(crate) struct FilesystemContext(FilesystemContextInner);

#[derive(Debug)]
pub(crate) enum FilesystemContextInner {
    None,
    Path(std::path::PathBuf),
    #[cfg(feature = "filesystem")]
    SandboxPath(std::path::PathBuf),
    #[cfg(feature = "filesystem")]
    SandboxTemp {
        temp: tempfile::TempDir,
        path: std::path::PathBuf,
    },
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
                    let mut context = Self::sandbox_at(&target)?;
                    if let Some(cwd) = cwd {
                        context = context.with_fixture(cwd)?;
                    }
                    Ok(context)
                }
                crate::Mode::Fail | crate::Mode::Overwrite => {
                    let mut context = Self::sandbox_temp()?;
                    if let Some(cwd) = cwd {
                        context = context.with_fixture(cwd)?;
                    }
                    Ok(context)
                }
            }
            #[cfg(not(feature = "filesystem"))]
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Sandboxing is disabled",
            ))
        } else {
            Ok(cwd.map(|p| Self::live(p)).unwrap_or_else(Self::none))
        }
    }

    pub(crate) fn none() -> Self {
        Self(FilesystemContextInner::None)
    }

    pub(crate) fn live(target: &std::path::Path) -> Self {
        Self(FilesystemContextInner::Path(target.to_owned()))
    }

    #[cfg(feature = "filesystem")]
    pub(crate) fn sandbox_temp() -> Result<Self, std::io::Error> {
        let temp = tempfile::tempdir()?;
        // We need to get the `/private` prefix on Mac so variable substitutions work
        // correctly
        let path = canonicalize(temp.path())?;
        Ok(Self(FilesystemContextInner::SandboxTemp { temp, path }))
    }

    #[cfg(feature = "filesystem")]
    pub(crate) fn sandbox_at(target: &std::path::Path) -> Result<Self, std::io::Error> {
        let _ = std::fs::remove_dir_all(&target);
        std::fs::create_dir_all(&target)?;
        Ok(Self(FilesystemContextInner::SandboxPath(target.to_owned())))
    }

    #[cfg(feature = "filesystem")]
    pub(crate) fn with_fixture(
        self,
        template_root: &std::path::Path,
    ) -> Result<Self, std::io::Error> {
        match &self.0 {
            FilesystemContextInner::None | FilesystemContextInner::Path(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Sandboxing is disabled",
                ));
            }
            FilesystemContextInner::SandboxPath(path)
            | FilesystemContextInner::SandboxTemp { path, .. } => {
                debug!(
                    "Initializing {} from {}",
                    path.display(),
                    template_root.display()
                );
                copy_dir(template_root, &path)?;
            }
        }

        Ok(self)
    }

    pub(crate) fn is_sandbox(&self) -> bool {
        match &self.0 {
            FilesystemContextInner::None | FilesystemContextInner::Path(_) => false,
            #[cfg(feature = "filesystem")]
            FilesystemContextInner::SandboxPath(_) => true,
            #[cfg(feature = "filesystem")]
            FilesystemContextInner::SandboxTemp { .. } => true,
        }
    }

    pub(crate) fn path(&self) -> Option<&std::path::Path> {
        match &self.0 {
            FilesystemContextInner::None => None,
            FilesystemContextInner::Path(path) => Some(path.as_path()),
            #[cfg(feature = "filesystem")]
            FilesystemContextInner::SandboxPath(path) => Some(path.as_path()),
            #[cfg(feature = "filesystem")]
            FilesystemContextInner::SandboxTemp { path, .. } => Some(path.as_path()),
        }
    }

    pub(crate) fn close(self) -> Result<(), std::io::Error> {
        match self.0 {
            FilesystemContextInner::None | FilesystemContextInner::Path(_) => Ok(()),
            FilesystemContextInner::SandboxPath(_) => Ok(()),
            #[cfg(feature = "filesystem")]
            FilesystemContextInner::SandboxTemp { temp, .. } => temp.close(),
        }
    }
}

impl Default for FilesystemContext {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(feature = "filesystem")]
pub(crate) struct Iterate {
    inner: walkdir::IntoIter,
}

#[cfg(feature = "filesystem")]
impl Iterate {
    pub(crate) fn new(path: &std::path::Path) -> Self {
        Self {
            inner: walkdir::WalkDir::new(path).into_iter(),
        }
    }
}

#[cfg(feature = "filesystem")]
impl Iterator for Iterate {
    type Item = Result<std::path::PathBuf, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.inner.next().map(|e| {
            e.map(walkdir::DirEntry::into_path)
                .map_err(std::io::Error::from)
        }) {
            if entry.as_ref().ok().and_then(|e| e.file_name())
                != Some(std::ffi::OsStr::new(".keep"))
            {
                return Some(entry);
            }
        }
        None
    }
}

#[cfg(not(feature = "filesystem"))]
pub(crate) struct Iterate {}

#[cfg(not(feature = "filesystem"))]
impl Iterate {
    pub(crate) fn new(_path: &std::path::Path) -> Self {
        Self {}
    }
}

#[cfg(not(feature = "filesystem"))]
impl Iterator for Iterate {
    type Item = Result<std::path::PathBuf, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[cfg(feature = "filesystem")]
fn copy_dir(source: &std::path::Path, dest: &std::path::Path) -> Result<(), std::io::Error> {
    let source = canonicalize(source)?;
    let dest = canonicalize(dest)?;

    for current in Iterate::new(&source) {
        let current = current?;
        let rel = current.strip_prefix(&source).unwrap();
        let target = dest.join(rel);

        shallow_copy(&current, &target)?;
    }

    Ok(())
}

pub(crate) fn shallow_copy(
    source: &std::path::Path,
    dest: &std::path::Path,
) -> Result<(), std::io::Error> {
    let meta = source.symlink_metadata()?;
    if meta.is_dir() {
        std::fs::create_dir_all(dest)?;
    } else if meta.is_file() {
        std::fs::copy(source, dest)?;
    } else if let Ok(target) = std::fs::read_link(source) {
        symlink_to_file(dest, &target)?;
    }

    Ok(())
}

#[cfg(windows)]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(windows))]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(target, link)
}

pub(crate) fn resolve_dir(path: std::path::PathBuf) -> Result<std::path::PathBuf, std::io::Error> {
    let meta = std::fs::symlink_metadata(&path)?;
    if meta.is_dir() {
        canonicalize(&path)
    } else if meta.is_file() {
        // Git might checkout symlinks as files
        let target = std::fs::read_to_string(&path)?;
        let target_path = path.parent().unwrap().join(target);
        resolve_dir(target_path)
    } else {
        canonicalize(&path)
    }
}

fn canonicalize(path: &std::path::Path) -> Result<std::path::PathBuf, std::io::Error> {
    #[cfg(feature = "filesystem")]
    {
        dunce::canonicalize(path)
    }
    #[cfg(not(feature = "filesystem"))]
    {
        // Hope for the best
        Ok(strip_trailing_slash(path).to_owned())
    }
}

pub(crate) fn strip_trailing_slash(path: &std::path::Path) -> &std::path::Path {
    path.components().as_path()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn strips_trailing_slash() {
        let path = std::path::Path::new("/foo/bar/");
        let rendered = path.display().to_string();
        assert_eq!(rendered.as_bytes()[rendered.len() - 1], b'/');

        let stripped = strip_trailing_slash(path);
        let rendered = stripped.display().to_string();
        assert_eq!(rendered.as_bytes()[rendered.len() - 1], b'r');
    }
}
