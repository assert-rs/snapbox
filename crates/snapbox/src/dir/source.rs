#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirSource {
    inner: DirSourceInner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DirSourceInner {
    InMemory,
    Path(std::path::PathBuf),
}

impl DirSource {
    pub(crate) fn inmemory() -> Self {
        Self {
            inner: DirSourceInner::InMemory,
        }
    }

    pub fn is_inmemory(&self) -> bool {
        matches!(self.inner, DirSourceInner::InMemory)
    }

    pub fn path(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            inner: DirSourceInner::Path(path.into()),
        }
    }

    pub fn is_path(&self) -> bool {
        self.as_path().is_some()
    }

    pub fn as_path(&self) -> Option<&std::path::Path> {
        match &self.inner {
            DirSourceInner::Path(value) => Some(value.as_ref()),
            _ => None,
        }
    }
}
