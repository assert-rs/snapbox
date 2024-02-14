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

impl From<&'_ std::path::Path> for DataSource {
    fn from(value: &'_ std::path::Path) -> Self {
        Self::path(value)
    }
}

impl From<std::path::PathBuf> for DataSource {
    fn from(value: std::path::PathBuf) -> Self {
        Self::path(value)
    }
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            DataSourceInner::Path(path) => crate::path::display_relpath(path).fmt(f),
        }
    }
}
