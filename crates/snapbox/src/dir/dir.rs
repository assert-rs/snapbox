use super::FileType;

/// Collection of files
#[cfg(feature = "dir")] // for documentation purposes only
pub trait Dir {
    /// Initialize a test fixture directory `root`
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error>;
}

impl Dir for InMemoryDir {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.write_to(root)
    }
}

impl Dir for std::path::Path {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        let src = super::resolve_dir(self).map_err(|e| format!("{e}: {}", self.display()))?;
        for (relpath, entry) in PathIter::binary_iter(src.as_path()) {
            let dest = root.join(relpath);
            entry.write_to(&dest)?;
        }
        Ok(())
    }
}

impl Dir for &'_ std::path::Path {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        (*self).write_to(root)
    }
}

impl Dir for &'_ std::path::PathBuf {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.as_path().write_to(root)
    }
}

impl Dir for std::path::PathBuf {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.as_path().write_to(root)
    }
}

impl Dir for str {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to(root)
    }
}

impl Dir for &'_ str {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        (*self).write_to(root)
    }
}

impl Dir for &'_ String {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.as_str().write_to(root)
    }
}

impl Dir for String {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.as_str().write_to(root)
    }
}

impl Dir for std::ffi::OsStr {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        std::path::Path::new(self).write_to(root)
    }
}

impl Dir for &'_ std::ffi::OsStr {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        (*self).write_to(root)
    }
}

impl Dir for &'_ std::ffi::OsString {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.as_os_str().write_to(root)
    }
}

impl Dir for std::ffi::OsString {
    fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        self.as_os_str().write_to(root)
    }
}

#[cfg(feature = "dir")] // for documentation purposes only
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMemoryDir {
    content: std::collections::BTreeMap<std::path::PathBuf, DirEntry>,
}

impl InMemoryDir {
    /// Initialize a test fixture directory `root`
    pub fn write_to(&self, root: &std::path::Path) -> Result<(), crate::assert::Error> {
        for (relpath, entry) in &self.content {
            let dest = root.join(relpath);
            entry.write_to(&dest)?;
        }
        Ok(())
    }
}

pub type InMemoryDirIter = std::collections::btree_map::IntoIter<std::path::PathBuf, DirEntry>;

impl<P, E> FromIterator<(P, E)> for InMemoryDir
where
    P: Into<std::path::PathBuf>,
    E: Into<DirEntry>,
{
    fn from_iter<I: IntoIterator<Item = (P, E)>>(it: I) -> Self {
        let mut content: std::collections::BTreeMap<std::path::PathBuf, DirEntry> =
            Default::default();
        for (mut p, e) in it.into_iter().map(|(p, e)| (p.into(), e.into())) {
            if let Ok(rel) = p.strip_prefix("/") {
                p = rel.to_owned();
            }
            assert!(p.is_relative(), "{}", p.display());
            let mut ancestors = p.ancestors();
            ancestors.next(); // skip self
            for ancestor in ancestors {
                match content.entry(ancestor.to_owned()) {
                    std::collections::btree_map::Entry::Occupied(entry) => {
                        if entry.get().file_type() != FileType::File {
                            panic!(
                                "`{}` assumes `{}` is a dir but its a {:?}",
                                p.display(),
                                ancestor.display(),
                                entry.get().file_type()
                            );
                        }
                    }
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(DirEntry::Dir);
                    }
                }
            }
            match content.entry(p) {
                std::collections::btree_map::Entry::Occupied(entry) => {
                    panic!(
                        "`{}` is assumed to be empty but its a {:?}",
                        entry.key().display(),
                        entry.get().file_type()
                    );
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(e);
                }
            }
        }
        Self { content }
    }
}

/// Note: Ignores `.keep` files
#[cfg(feature = "dir")] // for documentation purposes only
pub struct PathIter {
    root: std::path::PathBuf,
    binary: bool,
    inner: walkdir::IntoIter,
}

impl PathIter {
    fn binary_iter(root: &std::path::Path) -> Self {
        let binary = true;
        Self::iter_(root, binary)
    }

    fn iter_(root: &std::path::Path, binary: bool) -> PathIter {
        PathIter {
            root: root.to_owned(),
            binary,
            inner: walkdir::WalkDir::new(root).into_iter(),
        }
    }
}

impl Iterator for PathIter {
    type Item = (std::path::PathBuf, DirEntry);

    fn next(&mut self) -> Option<Self::Item> {
        for raw in self.inner.by_ref() {
            let entry = match raw {
                Ok(raw) => {
                    if raw.file_type().is_file()
                        && raw.path().file_name() == Some(std::ffi::OsStr::new(".keep"))
                    {
                        crate::debug!("ignoring {}, `.keep` file", raw.path().display());
                        continue;
                    }

                    let Ok(path) = raw.path().strip_prefix(&self.root) else {
                        crate::debug!(
                            "ignoring {}, out of root {}",
                            raw.path().display(),
                            self.root.display()
                        );
                        continue;
                    };
                    let entry = match DirEntry::try_from_path(raw.path(), self.binary) {
                        Ok(entry) => entry,
                        Err(err) => DirEntry::error(err),
                    };
                    (path.to_owned(), entry)
                }
                Err(err) => {
                    let Some(path) = err.path() else {
                        crate::debug!("ignoring error {err}");
                        continue;
                    };
                    let Ok(path) = path.strip_prefix(&self.root) else {
                        crate::debug!(
                            "ignoring {}, out of root {}",
                            path.display(),
                            self.root.display()
                        );
                        continue;
                    };
                    (path.to_owned(), DirEntry::error(err.to_string().into()))
                }
            };
            return Some(entry);
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DirEntry {
    Dir,
    File(crate::Data),
    Symlink(std::path::PathBuf),
}

impl DirEntry {
    fn error(err: crate::assert::Error) -> Self {
        DirEntry::File(crate::Data::error(err, crate::data::DataFormat::Error))
    }

    fn try_from_path(path: &std::path::Path, binary: bool) -> Result<Self, crate::assert::Error> {
        let metadata = path
            .symlink_metadata()
            .map_err(|e| format!("{e}: {}", path.display()))?;
        let entry = if metadata.is_dir() {
            DirEntry::Dir
        } else if metadata.is_file() {
            let data = if binary {
                crate::Data::binary(
                    std::fs::read(path).map_err(|e| format!("{e}: {}", path.display()))?,
                )
            } else {
                crate::Data::try_read_from(path, None)
                    .map_err(|e| format!("{e}: {}", path.display()))?
            };
            DirEntry::File(data)
        } else if metadata.is_symlink() {
            DirEntry::Symlink(
                path.read_link()
                    .map_err(|e| format!("{e}: {}", path.display()))?,
            )
        } else {
            return Err(crate::assert::Error::new("unknown file type"));
        };
        Ok(entry)
    }

    pub fn write_to(&self, path: &std::path::Path) -> Result<(), crate::assert::Error> {
        match self {
            DirEntry::Dir => {
                std::fs::create_dir_all(path).map_err(|e| format!("{e}: {}", path.display()))?
            }
            DirEntry::File(content) => {
                std::fs::write(path, content.to_bytes()?)
                    .map_err(|e| format!("{e}: {}", path.display()))?;
                // Avoid a mtime check race where:
                // - Copy files
                // - Test checks mtime
                // - Test writes
                // - Test checks mtime
                //
                // If all of this happens too close to each other, then the second mtime check will think
                // nothing was written by the test.
                //
                // Instead of just setting 1s in the past, we'll use a reproducible mtime
                filetime::set_file_mtime(path, filetime::FileTime::zero())
                    .map_err(|e| format!("{e}: {}", path.display()))?;
            }
            DirEntry::Symlink(target) => {
                symlink_to_file(path, target).map_err(|e| format!("{e}: {}", path.display()))?;
            }
        }
        Ok(())
    }

    pub fn file_type(&self) -> FileType {
        match self {
            Self::Dir => FileType::Dir,
            Self::File(_) => FileType::File,
            Self::Symlink(_) => FileType::Symlink,
        }
    }
}

impl<D> From<D> for DirEntry
where
    D: crate::data::IntoData,
{
    fn from(data: D) -> Self {
        Self::File(data.into_data())
    }
}

#[cfg(windows)]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(windows))]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(target, link)
}
