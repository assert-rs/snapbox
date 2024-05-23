#[cfg(feature = "dir")]
use crate::filter::{Filter as _, FilterNewlines, FilterPaths, NormalizeToExpected};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathDiff {
    Failure(crate::assert::Error),
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
    /// Report differences between `actual_root` and `pattern_root`
    ///
    /// Note: Requires feature flag `path`
    #[cfg(feature = "dir")]
    pub fn subset_eq_iter(
        pattern_root: impl Into<std::path::PathBuf>,
        actual_root: impl Into<std::path::PathBuf>,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), Self>> {
        let pattern_root = pattern_root.into();
        let actual_root = actual_root.into();
        Self::subset_eq_iter_inner(pattern_root, actual_root)
    }

    #[cfg(feature = "dir")]
    pub(crate) fn subset_eq_iter_inner(
        expected_root: std::path::PathBuf,
        actual_root: std::path::PathBuf,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), Self>> {
        let walker = crate::dir::Walk::new(&expected_root);
        walker.map(move |r| {
            let expected_path = r.map_err(|e| Self::Failure(e.to_string().into()))?;
            let rel = expected_path.strip_prefix(&expected_root).unwrap();
            let actual_path = actual_root.join(rel);

            let expected_type = FileType::from_path(&expected_path);
            let actual_type = FileType::from_path(&actual_path);
            if expected_type != actual_type {
                return Err(Self::TypeMismatch {
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
                        return Err(Self::LinkMismatch {
                            expected_path,
                            actual_path,
                            expected_target: expected_target.unwrap(),
                            actual_target: actual_target.unwrap(),
                        });
                    }
                }
                FileType::File => {
                    let mut actual =
                        crate::Data::try_read_from(&actual_path, None).map_err(Self::Failure)?;

                    let expected =
                        FilterNewlines.filter(crate::Data::read_from(&expected_path, None));

                    actual = FilterNewlines.filter(actual.coerce_to(expected.intended_format()));

                    if expected != actual {
                        return Err(Self::ContentMismatch {
                            expected_path,
                            actual_path,
                            expected_content: expected,
                            actual_content: actual,
                        });
                    }
                }
                FileType::Dir | FileType::Unknown | FileType::Missing => {}
            }

            Ok((expected_path, actual_path))
        })
    }

    /// Report differences between `actual_root` and `pattern_root`
    ///
    /// Note: Requires feature flag `path`
    #[cfg(feature = "dir")]
    pub fn subset_matches_iter(
        pattern_root: impl Into<std::path::PathBuf>,
        actual_root: impl Into<std::path::PathBuf>,
        substitutions: &crate::Redactions,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), Self>> + '_ {
        let pattern_root = pattern_root.into();
        let actual_root = actual_root.into();
        Self::subset_matches_iter_inner(pattern_root, actual_root, substitutions, true)
    }

    #[cfg(feature = "dir")]
    pub(crate) fn subset_matches_iter_inner(
        expected_root: std::path::PathBuf,
        actual_root: std::path::PathBuf,
        substitutions: &crate::Redactions,
        normalize_paths: bool,
    ) -> impl Iterator<Item = Result<(std::path::PathBuf, std::path::PathBuf), Self>> + '_ {
        let walker = crate::dir::Walk::new(&expected_root);
        walker.map(move |r| {
            let expected_path = r.map_err(|e| Self::Failure(e.to_string().into()))?;
            let rel = expected_path.strip_prefix(&expected_root).unwrap();
            let actual_path = actual_root.join(rel);

            let expected_type = FileType::from_path(&expected_path);
            let actual_type = FileType::from_path(&actual_path);
            if expected_type != actual_type {
                return Err(Self::TypeMismatch {
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
                        return Err(Self::LinkMismatch {
                            expected_path,
                            actual_path,
                            expected_target: expected_target.unwrap(),
                            actual_target: actual_target.unwrap(),
                        });
                    }
                }
                FileType::File => {
                    let mut actual =
                        crate::Data::try_read_from(&actual_path, None).map_err(Self::Failure)?;

                    let expected =
                        FilterNewlines.filter(crate::Data::read_from(&expected_path, None));

                    actual = actual.coerce_to(expected.intended_format());
                    if normalize_paths {
                        actual = FilterPaths.filter(actual);
                    }
                    actual = NormalizeToExpected::new()
                        .redact_with(substitutions)
                        .normalize(FilterNewlines.filter(actual), &expected);

                    if expected != actual {
                        return Err(Self::ContentMismatch {
                            expected_path,
                            actual_path,
                            expected_content: expected,
                            actual_content: actual,
                        });
                    }
                }
                FileType::Dir | FileType::Unknown | FileType::Missing => {}
            }

            Ok((expected_path, actual_path))
        })
    }
}

impl PathDiff {
    pub fn expected_path(&self) -> Option<&std::path::Path> {
        match &self {
            Self::Failure(_msg) => None,
            Self::TypeMismatch {
                expected_path,
                actual_path: _,
                expected_type: _,
                actual_type: _,
            } => Some(expected_path),
            Self::LinkMismatch {
                expected_path,
                actual_path: _,
                expected_target: _,
                actual_target: _,
            } => Some(expected_path),
            Self::ContentMismatch {
                expected_path,
                actual_path: _,
                expected_content: _,
                actual_content: _,
            } => Some(expected_path),
        }
    }

    pub fn write(
        &self,
        f: &mut dyn std::fmt::Write,
        palette: crate::report::Palette,
    ) -> Result<(), std::fmt::Error> {
        match &self {
            Self::Failure(msg) => {
                writeln!(f, "{}", palette.error(msg))?;
            }
            Self::TypeMismatch {
                expected_path,
                actual_path: _actual_path,
                expected_type,
                actual_type,
            } => {
                writeln!(
                    f,
                    "{}: Expected {}, was {}",
                    expected_path.display(),
                    palette.info(expected_type),
                    palette.error(actual_type)
                )?;
            }
            Self::LinkMismatch {
                expected_path,
                actual_path: _actual_path,
                expected_target,
                actual_target,
            } => {
                writeln!(
                    f,
                    "{}: Expected {}, was {}",
                    expected_path.display(),
                    palette.info(expected_target.display()),
                    palette.error(actual_target.display())
                )?;
            }
            Self::ContentMismatch {
                expected_path,
                actual_path,
                expected_content,
                actual_content,
            } => {
                crate::report::write_diff(
                    f,
                    expected_content,
                    actual_content,
                    Some(&expected_path.display()),
                    Some(&actual_path.display()),
                    palette,
                )?;
            }
        }

        Ok(())
    }

    pub fn overwrite(&self) -> Result<(), crate::assert::Error> {
        match self {
            // Not passing the error up because users most likely want to treat a processing error
            // differently than an overwrite error
            Self::Failure(_err) => Ok(()),
            Self::TypeMismatch {
                expected_path,
                actual_path,
                expected_type: _,
                actual_type,
            } => {
                match actual_type {
                    FileType::Dir => {
                        std::fs::remove_dir_all(expected_path).map_err(|e| {
                            format!("Failed to remove {}: {}", expected_path.display(), e)
                        })?;
                    }
                    FileType::File | FileType::Symlink => {
                        std::fs::remove_file(expected_path).map_err(|e| {
                            format!("Failed to remove {}: {}", expected_path.display(), e)
                        })?;
                    }
                    FileType::Unknown | FileType::Missing => {}
                }
                super::shallow_copy(expected_path, actual_path)
            }
            Self::LinkMismatch {
                expected_path,
                actual_path,
                expected_target: _,
                actual_target: _,
            } => super::shallow_copy(expected_path, actual_path),
            Self::ContentMismatch {
                expected_path: _,
                actual_path: _,
                expected_content,
                actual_content,
            } => actual_content.write_to(expected_content.source().unwrap()),
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
