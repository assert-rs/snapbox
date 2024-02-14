#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSource {
    pub(crate) inner: DataSourceInner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DataSourceInner {
    Path(std::path::PathBuf),
}

impl DataSource {
    pub fn path(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            inner: DataSourceInner::Path(path.into()),
        }
    }

    pub fn is_path(&self) -> bool {
        self.as_path().is_some()
    }

    pub fn as_path(&self) -> Option<&std::path::Path> {
        match &self.inner {
            DataSourceInner::Path(path) => Some(path.as_ref()),
        }
    }
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            DataSourceInner::Path(path) => crate::path::display_relpath(path).fmt(f),
        }
    }
}
