#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    inner: String,
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn new(inner: String) -> Self {
        Self { inner }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for Error {}

impl<'s> From<&'s str> for Error {
    fn from(other: &'s str) -> Self {
        Self::new(other.into())
    }
}

impl<'s> From<&'s String> for Error {
    fn from(other: &'s String) -> Self {
        Self::new(other.clone())
    }
}

impl From<String> for Error {
    fn from(other: String) -> Self {
        Self::new(other)
    }
}
