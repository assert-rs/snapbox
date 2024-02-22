#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSource {
    pub(crate) inner: DataSourceInner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DataSourceInner {
    Path(std::path::PathBuf),
    Inline(Inline),
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
            DataSourceInner::Path(value) => Some(value.as_ref()),
            _ => None,
        }
    }

    pub fn is_inline(&self) -> bool {
        self.as_inline().is_some()
    }

    pub fn as_inline(&self) -> Option<&Inline> {
        match &self.inner {
            DataSourceInner::Inline(value) => Some(value),
            _ => None,
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

impl From<Inline> for DataSource {
    fn from(inline: Inline) -> Self {
        Self {
            inner: DataSourceInner::Inline(inline),
        }
    }
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            DataSourceInner::Path(value) => crate::path::display_relpath(value).fmt(f),
            DataSourceInner::Inline(value) => value.fmt(f),
        }
    }
}

/// Data from within Rust source code
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    #[doc(hidden)]
    pub position: Position,
    #[doc(hidden)]
    pub data: &'static str,
    #[doc(hidden)]
    pub indent: bool,
}

impl Inline {
    /// Indent to quote-level when overwriting the string literal (default)
    pub fn indent(mut self, yes: bool) -> Self {
        self.indent = yes;
        self
    }

    /// Initialize `Self` as [`format`][crate::data::DataFormat] or [`Error`][crate::data::DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    pub fn is(self, format: super::DataFormat) -> super::Data {
        let data: super::Data = self.into();
        data.is(format)
    }

    /// Deprecated, replaced with [`Inline::is`]
    #[deprecated(since = "0.5.2", note = "Replaced with `Inline::is`")]
    pub fn coerce_to(self, format: super::DataFormat) -> super::Data {
        let data: super::Data = self.into();
        data.coerce_to(format)
    }

    fn trimmed(&self) -> String {
        if !self.data.contains('\n') {
            return self.data.to_string();
        }
        trim_indent(self.data)
    }
}

fn trim_indent(mut text: &str) -> String {
    if text.starts_with('\n') {
        text = &text[1..];
    }
    let indent = text
        .lines()
        .filter(|it| !it.trim().is_empty())
        .map(|it| it.len() - it.trim_start().len())
        .min()
        .unwrap_or(0);

    crate::utils::LinesWithTerminator::new(text)
        .map(|line| {
            if line.len() <= indent {
                line.trim_start_matches(' ')
            } else {
                &line[indent..]
            }
        })
        .collect()
}

impl std::fmt::Display for Inline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.position.fmt(f)
    }
}

impl From<Inline> for super::Data {
    fn from(inline: Inline) -> Self {
        let trimmed = inline.trimmed();
        super::Data::text(trimmed).with_source(inline)
    }
}

/// Position within Rust source code, see [`Inline`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    #[doc(hidden)]
    pub file: std::path::PathBuf,
    #[doc(hidden)]
    pub line: u32,
    #[doc(hidden)]
    pub column: u32,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}
