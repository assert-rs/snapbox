/// Recursively walk a path
///
/// Note: Ignores `.keep` files
#[cfg(feature = "dir")]
pub struct Walk {
    inner: walkdir::IntoIter,
}

#[cfg(feature = "dir")]
impl Walk {
    pub fn new(path: &std::path::Path) -> Self {
        Self {
            inner: walkdir::WalkDir::new(path).into_iter(),
        }
    }
}

#[cfg(feature = "dir")]
impl Iterator for Walk {
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

/// Copy a template into a [`DirRoot`][super::DirRoot]
///
/// Note: Generally you'll use [`DirRoot::with_template`][super::DirRoot::with_template] instead.
///
/// Note: Ignores `.keep` files
#[cfg(feature = "dir")]
pub fn copy_template(
    source: impl AsRef<std::path::Path>,
    dest: impl AsRef<std::path::Path>,
) -> Result<(), crate::assert::Error> {
    let source = source.as_ref();
    let dest = dest.as_ref();
    let source = canonicalize(source)
        .map_err(|e| format!("Failed to canonicalize {}: {}", source.display(), e))?;
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create {}: {}", dest.display(), e))?;
    let dest = canonicalize(dest)
        .map_err(|e| format!("Failed to canonicalize {}: {}", dest.display(), e))?;

    for current in Walk::new(&source) {
        let current = current.map_err(|e| e.to_string())?;
        let rel = current.strip_prefix(&source).unwrap();
        let target = dest.join(rel);

        shallow_copy(&current, &target)?;
    }

    Ok(())
}

/// Copy a file system entry, without recursing
pub(crate) fn shallow_copy(
    source: &std::path::Path,
    dest: &std::path::Path,
) -> Result<(), crate::assert::Error> {
    let meta = source
        .symlink_metadata()
        .map_err(|e| format!("Failed to read metadata from {}: {}", source.display(), e))?;
    if meta.is_dir() {
        std::fs::create_dir_all(dest)
            .map_err(|e| format!("Failed to create {}: {}", dest.display(), e))?;
    } else if meta.is_file() {
        std::fs::copy(source, dest).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {}",
                source.display(),
                dest.display(),
                e
            )
        })?;
        // Avoid a mtime check race where:
        // - Copy files
        // - Test checks mtime
        // - Test writes
        // - Test checks mtime
        //
        // If all of this happens too close to each other, then the second mtime check will think
        // nothing was written by the test.
        //
        // Instead of just setting 1s in the past, we'll just respect the existing mtime.
        copy_stats(&meta, dest).map_err(|e| {
            format!(
                "Failed to copy {} metadata to {}: {}",
                source.display(),
                dest.display(),
                e
            )
        })?;
    } else if let Ok(target) = std::fs::read_link(source) {
        symlink_to_file(dest, &target)
            .map_err(|e| format!("Failed to create symlink {}: {}", dest.display(), e))?;
    }

    Ok(())
}

#[cfg(feature = "dir")]
fn copy_stats(
    source_meta: &std::fs::Metadata,
    dest: &std::path::Path,
) -> Result<(), std::io::Error> {
    let src_mtime = filetime::FileTime::from_last_modification_time(source_meta);
    filetime::set_file_mtime(dest, src_mtime)?;

    Ok(())
}

#[cfg(not(feature = "dir"))]
fn copy_stats(
    _source_meta: &std::fs::Metadata,
    _dest: &std::path::Path,
) -> Result<(), std::io::Error> {
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

pub fn resolve_dir(
    path: impl AsRef<std::path::Path>,
) -> Result<std::path::PathBuf, std::io::Error> {
    let path = path.as_ref();
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        canonicalize(path)
    } else if meta.is_file() {
        // Git might checkout symlinks as files
        let target = std::fs::read_to_string(path)?;
        let target_path = path.parent().unwrap().join(target);
        resolve_dir(target_path)
    } else {
        canonicalize(path)
    }
}

pub(crate) fn canonicalize(path: &std::path::Path) -> Result<std::path::PathBuf, std::io::Error> {
    #[cfg(feature = "dir")]
    {
        dunce::canonicalize(path)
    }
    #[cfg(not(feature = "dir"))]
    {
        // Hope for the best
        Ok(strip_trailing_slash(path).to_owned())
    }
}

pub fn strip_trailing_slash(path: &std::path::Path) -> &std::path::Path {
    path.components().as_path()
}

/// Normalize a path, removing things like `.` and `..`.
///
/// CAUTION: This does not resolve symlinks (unlike
/// [`std::fs::canonicalize`]). This may cause incorrect or surprising
/// behavior at times. This should be used carefully. Unfortunately,
/// [`std::fs::canonicalize`] can be hard to use correctly, since it can often
/// fail, or on Windows returns annoying device paths. This is a problem Cargo
/// needs to improve on.
pub(crate) fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    use std::path::Component;

    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        std::path::PathBuf::from(c.as_os_str())
    } else {
        std::path::PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if ret.ends_with(Component::ParentDir) {
                    ret.push(Component::ParentDir);
                } else {
                    let popped = ret.pop();
                    if !popped && !ret.has_root() {
                        ret.push(Component::ParentDir);
                    }
                }
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

pub(crate) fn display_relpath(path: impl AsRef<std::path::Path>) -> String {
    let path = path.as_ref();
    let relpath = if let Ok(cwd) = std::env::current_dir() {
        match path.strip_prefix(cwd) {
            Ok(path) => path,
            Err(_) => path,
        }
    } else {
        path
    };
    relpath.display().to_string()
}
