use crate::Action;

#[derive(Debug)]
pub struct PathFixture(PathFixtureInner);

#[derive(Debug)]
enum PathFixtureInner {
    None,
    Immutable(std::path::PathBuf),
    #[cfg(feature = "path")]
    MutablePath(std::path::PathBuf),
    #[cfg(feature = "path")]
    MutableTemp {
        temp: tempfile::TempDir,
        path: std::path::PathBuf,
    },
}

impl PathFixture {
    pub fn none() -> Self {
        Self(PathFixtureInner::None)
    }

    pub fn immutable(target: &std::path::Path) -> Self {
        Self(PathFixtureInner::Immutable(target.to_owned()))
    }

    #[cfg(feature = "path")]
    pub fn mutable_temp() -> Result<Self, crate::Error> {
        let temp = tempfile::tempdir().map_err(|e| e.to_string())?;
        // We need to get the `/private` prefix on Mac so variable substitutions work
        // correctly
        let path = canonicalize(temp.path())
            .map_err(|e| format!("Failed to canonicalize {}: {}", temp.path().display(), e))?;
        Ok(Self(PathFixtureInner::MutableTemp { temp, path }))
    }

    #[cfg(feature = "path")]
    pub fn mutable_at(target: &std::path::Path) -> Result<Self, crate::Error> {
        let _ = std::fs::remove_dir_all(&target);
        std::fs::create_dir_all(&target)
            .map_err(|e| format!("Failed to create {}: {}", target.display(), e))?;
        Ok(Self(PathFixtureInner::MutablePath(target.to_owned())))
    }

    #[cfg(feature = "path")]
    pub fn with_template(self, template_root: &std::path::Path) -> Result<Self, crate::Error> {
        match &self.0 {
            PathFixtureInner::None | PathFixtureInner::Immutable(_) => {
                return Err("Sandboxing is disabled".into());
            }
            PathFixtureInner::MutablePath(path) | PathFixtureInner::MutableTemp { path, .. } => {
                crate::debug!(
                    "Initializing {} from {}",
                    path.display(),
                    template_root.display()
                );
                copy_template(template_root, path)?;
            }
        }

        Ok(self)
    }

    pub fn is_mutable(&self) -> bool {
        match &self.0 {
            PathFixtureInner::None | PathFixtureInner::Immutable(_) => false,
            #[cfg(feature = "path")]
            PathFixtureInner::MutablePath(_) => true,
            #[cfg(feature = "path")]
            PathFixtureInner::MutableTemp { .. } => true,
        }
    }

    pub fn path(&self) -> Option<&std::path::Path> {
        match &self.0 {
            PathFixtureInner::None => None,
            PathFixtureInner::Immutable(path) => Some(path.as_path()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutablePath(path) => Some(path.as_path()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutableTemp { path, .. } => Some(path.as_path()),
        }
    }

    /// Explicitly close to report errors
    pub fn close(self) -> Result<(), std::io::Error> {
        match self.0 {
            PathFixtureInner::None | PathFixtureInner::Immutable(_) => Ok(()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutablePath(_) => Ok(()),
            #[cfg(feature = "path")]
            PathFixtureInner::MutableTemp { temp, .. } => temp.close(),
        }
    }
}

impl Default for PathFixture {
    fn default() -> Self {
        Self::none()
    }
}

pub fn path_assert() -> PathAssert {
    Default::default()
}

pub struct PathAssert {
    action: Action,
    substitutions: crate::Substitutions,
    palette: crate::report::Palette,
}

/// # Customize Behavior
impl PathAssert {
    /// Override the color palette
    pub fn palette(mut self, palette: crate::report::Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Read the failure action from an environment variable
    pub fn action_env(mut self, var_name: &str) -> Self {
        let action = Action::with_env_var(var_name);
        self.action = action.unwrap_or(self.action);
        self
    }

    /// Override the failure action
    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    /// Override the default [`Substitutions`][crate::Substitutions]
    pub fn substitutions(mut self, substitutions: crate::Substitutions) -> Self {
        self.substitutions = substitutions;
        self
    }
}

/// # Building Blocks
impl PathAssert {
    pub fn subset_eq_iter<'s>(
        &'s self,
        actual_root: impl Into<std::path::PathBuf>,
        pattern_root: impl Into<std::path::PathBuf>,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), PathDiff>> + 's {
        let actual_root = actual_root.into();
        let pattern_root = pattern_root.into();
        self.subset_eq_iter_inner(actual_root, pattern_root)
    }

    fn subset_eq_iter_inner<'s>(
        &'s self,
        actual_root: std::path::PathBuf,
        expected_root: std::path::PathBuf,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), PathDiff>> + 's {
        let walker = Walk::new(&expected_root);
        walker.map(move |r| {
            let expected_path = r.map_err(|e| PathDiff::Failure(e.to_string().into()))?;
            let rel = expected_path.strip_prefix(&expected_root).unwrap();
            let actual_path = actual_root.join(rel);

            let expected_type = FileType::from_path(&expected_path);
            let actual_type = FileType::from_path(&actual_path);
            if expected_type != actual_type {
                return Err(PathDiff::TypeMismatch {
                    expected_path,
                    actual_path,
                    expected_type,
                    actual_type,
                });
            }

            match expected_type {
                FileType::Symlink => {
                    let expected_target = std::fs::read_link(&expected_path).ok();
                    let actual_target = std::fs::read_link(&actual_path).ok();
                    if expected_target != actual_target {
                        return Err(PathDiff::LinkMismatch {
                            expected_path,
                            actual_path,
                            expected_target: expected_target.unwrap(),
                            actual_target: actual_target.unwrap(),
                        });
                    }
                }
                FileType::File => {
                    let mut actual =
                        crate::Data::read_from(&actual_path, None).map_err(PathDiff::Failure)?;

                    let expected = crate::Data::read_from(&expected_path, None)
                        .map(|d| d.map_text(crate::utils::normalize_lines))
                        .map_err(PathDiff::Failure)?;
                    if expected.as_str().is_some() {
                        actual = actual.try_text().map_text(crate::utils::normalize_lines);
                    }

                    if expected != actual {
                        return Err(PathDiff::ContentMismatch {
                            expected_path,
                            actual_path,
                            expected_content: expected,
                            actual_content: actual,
                        });
                    }
                }
                FileType::Dir | FileType::Unknown | FileType::Missing => {}
            }

            Ok((actual_path, expected_path))
        })
    }

    pub fn subset_matches_iter<'s>(
        &'s self,
        actual_root: impl Into<std::path::PathBuf>,
        pattern_root: impl Into<std::path::PathBuf>,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), PathDiff>> + 's {
        let actual_root = actual_root.into();
        let pattern_root = pattern_root.into();
        self.subset_matches_iter_inner(actual_root, pattern_root)
    }

    fn subset_matches_iter_inner<'s>(
        &'s self,
        actual_root: std::path::PathBuf,
        expected_root: std::path::PathBuf,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), PathDiff>> + 's {
        let walker = Walk::new(&expected_root);
        walker.map(move |r| {
            let expected_path = r.map_err(|e| PathDiff::Failure(e.to_string().into()))?;
            let rel = expected_path.strip_prefix(&expected_root).unwrap();
            let actual_path = actual_root.join(rel);

            let expected_type = FileType::from_path(&expected_path);
            let actual_type = FileType::from_path(&actual_path);
            if expected_type != actual_type {
                return Err(PathDiff::TypeMismatch {
                    expected_path,
                    actual_path,
                    expected_type,
                    actual_type,
                });
            }

            match expected_type {
                FileType::Symlink => {
                    let expected_target = std::fs::read_link(&expected_path).ok();
                    let actual_target = std::fs::read_link(&actual_path).ok();
                    if expected_target != actual_target {
                        return Err(PathDiff::LinkMismatch {
                            expected_path,
                            actual_path,
                            expected_target: expected_target.unwrap(),
                            actual_target: actual_target.unwrap(),
                        });
                    }
                }
                FileType::File => {
                    let mut actual =
                        crate::Data::read_from(&actual_path, None).map_err(PathDiff::Failure)?;

                    let expected = crate::Data::read_from(&expected_path, None)
                        .map(|d| d.map_text(crate::utils::normalize_lines))
                        .map_err(PathDiff::Failure)?;
                    if let Some(expected) = expected.as_str() {
                        actual = actual
                            .try_text()
                            .map_text(crate::utils::normalize_text)
                            .map_text(|t| self.substitutions.normalize(t, expected));
                    }

                    if expected != actual {
                        return Err(PathDiff::ContentMismatch {
                            expected_path,
                            actual_path,
                            expected_content: expected,
                            actual_content: actual,
                        });
                    }
                }
                FileType::Dir | FileType::Unknown | FileType::Missing => {}
            }

            Ok((actual_path, expected_path))
        })
    }
}

impl Default for PathAssert {
    fn default() -> Self {
        Self {
            action: Action::Verify,
            substitutions: crate::Substitutions::with_exe(),
            palette: crate::report::Palette::auto(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathDiff {
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

impl PathDiff {
    pub fn overwrite(&self) -> Result<(), crate::Error> {
        match self {
            // Not passing the error up because users most likely want to treat a processing error
            // differently than an overwrite error
            Self::Failure(_err) => Ok(()),
            Self::TypeMismatch {
                expected_path,
                actual_path,
                expected_type: _,
                actual_type: _,
            } => shallow_copy(&expected_path, &actual_path),
            Self::LinkMismatch {
                expected_path,
                actual_path,
                expected_target: _,
                actual_target: _,
            } => shallow_copy(&expected_path, &actual_path),
            Self::ContentMismatch {
                expected_path,
                actual_path: _,
                expected_content: _,
                actual_content,
            } => actual_content.write_to(&expected_path),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileType {
    Dir,
    File,
    Symlink,
    Unknown,
    Missing,
}

impl FileType {
    pub fn from_path(path: &std::path::Path) -> Self {
        let meta = path.symlink_metadata();
        match meta {
            Ok(meta) => {
                if meta.is_dir() {
                    Self::Dir
                } else if meta.is_file() {
                    Self::File
                } else {
                    let target = std::fs::read_link(path).ok();
                    if target.is_some() {
                        Self::Symlink
                    } else {
                        Self::Unknown
                    }
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Self::Missing,
                _ => Self::Unknown,
            },
        }
    }
}

impl FileType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dir => "dir",
            Self::File => "file",
            Self::Symlink => "symlink",
            Self::Unknown => "unknown",
            Self::Missing => "missing",
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

/// Recursively walk a path
///
/// Note: Ignores `.keep` files
#[cfg(feature = "path")]
pub struct Walk {
    inner: walkdir::IntoIter,
}

#[cfg(feature = "path")]
impl Walk {
    pub fn new(path: &std::path::Path) -> Self {
        Self {
            inner: walkdir::WalkDir::new(path).into_iter(),
        }
    }
}

#[cfg(feature = "path")]
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

#[cfg(not(feature = "path"))]
pub struct Walk {}

#[cfg(not(feature = "path"))]
impl Walk {
    pub fn new(_path: &std::path::Path) -> Self {
        Self {}
    }
}

#[cfg(not(feature = "path"))]
impl Iterator for Walk {
    type Item = Result<std::path::PathBuf, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

/// Copy a template into a [`PathFixture`]
///
/// Note: Generally you'll use [`PathFixture::with_template`] instead.
///
/// Note: Ignores `.keep` files
#[cfg(feature = "path")]
pub fn copy_template(source: &std::path::Path, dest: &std::path::Path) -> Result<(), crate::Error> {
    let source = canonicalize(source)
        .map_err(|e| format!("Failed to canonicalize {}: {}", source.display(), e))?;
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
pub fn shallow_copy(source: &std::path::Path, dest: &std::path::Path) -> Result<(), crate::Error> {
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
    } else if let Ok(target) = std::fs::read_link(source) {
        symlink_to_file(dest, &target)
            .map_err(|e| format!("Failed to create symlink {}: {}", dest.display(), e))?;
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

pub fn resolve_dir(path: std::path::PathBuf) -> Result<std::path::PathBuf, std::io::Error> {
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
    #[cfg(feature = "path")]
    {
        dunce::canonicalize(path)
    }
    #[cfg(not(feature = "path"))]
    {
        // Hope for the best
        Ok(strip_trailing_slash(path).to_owned())
    }
}

pub fn strip_trailing_slash(path: &std::path::Path) -> &std::path::Path {
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

    #[test]
    fn file_type_detect_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        dbg!(&path);
        let actual = FileType::from_path(&path);
        assert_eq!(actual, FileType::File);
    }

    #[test]
    fn file_type_detect_dir() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        dbg!(path);
        let actual = FileType::from_path(path);
        assert_eq!(actual, FileType::Dir);
    }

    #[test]
    fn file_type_detect_missing() {
        let path = std::path::Path::new("this-should-never-exist");
        dbg!(path);
        let actual = FileType::from_path(path);
        assert_eq!(actual, FileType::Missing);
    }
}
