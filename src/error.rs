#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    inner: String,
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn new(inner: String) -> Self {
        Self { inner }
    }

    pub(crate) fn into_string(self) -> String {
        self.inner
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for Error {}
