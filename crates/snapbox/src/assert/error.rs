pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
pub struct Error {
    inner: String,
    backtrace: Option<Backtrace>,
}

impl Error {
    pub fn new(inner: impl std::fmt::Display) -> Self {
        Self::with_string(inner.to_string())
    }

    fn with_string(inner: String) -> Self {
        Self {
            inner,
            backtrace: Backtrace::new(),
        }
    }

    #[track_caller]
    pub(crate) fn panic(self) -> ! {
        panic!("{self}")
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.inner)?;
        if let Some(backtrace) = self.backtrace.as_ref() {
            writeln!(f)?;
            writeln!(f, "Backtrace:")?;
            writeln!(f, "{backtrace}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl<'s> From<&'s str> for Error {
    fn from(other: &'s str) -> Self {
        Self::with_string(other.to_owned())
    }
}

impl<'s> From<&'s String> for Error {
    fn from(other: &'s String) -> Self {
        Self::with_string(other.clone())
    }
}

impl From<String> for Error {
    fn from(other: String) -> Self {
        Self::with_string(other)
    }
}

#[cfg(feature = "debug")]
#[derive(Debug, Clone)]
struct Backtrace(backtrace::Backtrace);

#[cfg(feature = "debug")]
impl Backtrace {
    fn new() -> Option<Self> {
        Some(Self(backtrace::Backtrace::new()))
    }
}

#[cfg(feature = "debug")]
impl std::fmt::Display for Backtrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `backtrace::Backtrace` uses `Debug` instead of `Display`
        write!(f, "{:?}", self.0)
    }
}

#[cfg(not(feature = "debug"))]
#[derive(Debug, Copy, Clone)]
struct Backtrace;

#[cfg(not(feature = "debug"))]
impl Backtrace {
    fn new() -> Option<Self> {
        None
    }
}

#[cfg(not(feature = "debug"))]
impl std::fmt::Display for Backtrace {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
