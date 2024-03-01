mod format;
mod normalize;
mod runtime;
mod source;
#[cfg(test)]
mod tests;

pub use format::DataFormat;
pub use normalize::Normalize;
pub use normalize::NormalizeMatches;
pub use normalize::NormalizeNewlines;
pub use normalize::NormalizePaths;
pub use source::DataSource;
pub use source::Inline;
pub use source::Position;

pub trait ToDebug {
    fn to_debug(&self) -> Data;
}

impl<D: std::fmt::Debug> ToDebug for D {
    fn to_debug(&self) -> Data {
        Data::text(format!("{:#?}\n", self))
    }
}

/// Declare an expected value for an assert from a file
///
/// This is relative to the source file the macro is run from
///
/// ```
/// # #[cfg(feature = "json")] {
/// # use snapbox::file;
/// file!["./test_data/bar.json"];
/// file!["./test_data/bar.json": Text];  // do textual rather than structural comparisons
/// file![_];
/// file![_: Json];  // ensure its treated as json since a type can't be inferred
/// # }
/// ```
#[macro_export]
macro_rules! file {
    [_] => {{
        let path = $crate::data::generate_snapshot_path($crate::fn_path!(), None);
        $crate::Data::read_from(&path, None)
    }};
    [_ : $type:ident] => {{
        let format = $crate::data::DataFormat:: $type;
        let path = $crate::data::generate_snapshot_path($crate::fn_path!(), Some(format));
        $crate::Data::read_from(&path, Some($crate::data::DataFormat:: $type))
    }};
    [$path:literal] => {{
        let mut path = $crate::current_dir!();
        path.push($path);
        $crate::Data::read_from(&path, None)
    }};
    [$path:literal : $type:ident] => {{
        let mut path = $crate::current_dir!();
        path.push($path);
        $crate::Data::read_from(&path, Some($crate::data::DataFormat:: $type))
    }};
}

/// Declare an expected value from within Rust source
///
/// ```
/// # use snapbox::str;
/// str![["
///     Foo { value: 92 }
/// "]];
/// str![r#"{"Foo": 92}"#];
/// ```
///
/// Leading indentation is stripped.
///
/// See [`Inline::is`] for declaring the data to be of a certain format.
#[macro_export]
macro_rules! str {
    [$data:literal] => { $crate::str![[$data]] };
    [[$data:literal]] => {{
        let position = $crate::data::Position {
            file: $crate::path::current_rs!(),
            line: line!(),
            column: column!(),
        };
        let inline = $crate::data::Inline {
            position,
            data: $data,
            indent: true,
        };
        inline
    }};
    [] => { $crate::str![[""]] };
    [[]] => { $crate::str![[""]] };
}

/// Test fixture, actual output, or expected result
///
/// This provides conveniences for tracking the intended format (binary vs text).
#[derive(Clone, Debug)]
pub struct Data {
    inner: DataInner,
    source: Option<DataSource>,
}

#[derive(Clone, Debug)]
pub(crate) enum DataInner {
    Error(DataError),
    Binary(Vec<u8>),
    Text(String),
    #[cfg(feature = "json")]
    Json(serde_json::Value),
    #[cfg(feature = "term-svg")]
    TermSvg(String),
}

impl Data {
    /// Mark the data as binary (no post-processing)
    pub fn binary(raw: impl Into<Vec<u8>>) -> Self {
        DataInner::Binary(raw.into()).into()
    }

    /// Mark the data as text (post-processing)
    pub fn text(raw: impl Into<String>) -> Self {
        DataInner::Text(raw.into()).into()
    }

    #[cfg(feature = "json")]
    pub fn json(raw: impl Into<serde_json::Value>) -> Self {
        DataInner::Json(raw.into()).into()
    }

    fn error(raw: impl Into<crate::Error>, intended: DataFormat) -> Self {
        DataError {
            error: raw.into(),
            intended,
        }
        .into()
    }

    /// Empty test data
    pub fn new() -> Self {
        Self::text("")
    }

    fn with_source(mut self, source: impl Into<DataSource>) -> Self {
        self.source = Some(source.into());
        self
    }

    fn with_path(self, path: impl Into<std::path::PathBuf>) -> Self {
        self.with_source(path.into())
    }

    /// Load `expected` data from a file
    pub fn read_from(path: &std::path::Path, data_format: Option<DataFormat>) -> Self {
        match Self::try_read_from(path, data_format) {
            Ok(data) => data,
            Err(err) => Self::error(err, data_format.unwrap_or_else(|| DataFormat::from(path)))
                .with_path(path),
        }
    }

    /// Load `expected` data from a file
    pub fn try_read_from(
        path: &std::path::Path,
        data_format: Option<DataFormat>,
    ) -> Result<Self, crate::Error> {
        let data =
            std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let data = Self::binary(data);
        let data = match data_format {
            Some(df) => data.is(df),
            None => {
                let inferred_format = DataFormat::from(path);
                match inferred_format {
                    #[cfg(feature = "json")]
                    DataFormat::Json => data.coerce_to(inferred_format),
                    #[cfg(feature = "term-svg")]
                    DataFormat::TermSvg => {
                        let data = data.coerce_to(DataFormat::Text);
                        data.is(inferred_format)
                    }
                    _ => data.coerce_to(DataFormat::Text),
                }
            }
        };
        Ok(data.with_path(path))
    }

    /// Location the data came from
    pub fn source(&self) -> Option<&DataSource> {
        self.source.as_ref()
    }

    /// Overwrite a snapshot
    pub fn write_to(&self, source: &DataSource) -> Result<(), crate::Error> {
        match &source.inner {
            source::DataSourceInner::Path(p) => self.write_to_path(p),
            source::DataSourceInner::Inline(p) => runtime::get()
                .write(self, p)
                .map_err(|err| err.to_string().into()),
        }
    }

    /// Overwrite a snapshot
    pub fn write_to_path(&self, path: &std::path::Path) -> Result<(), crate::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create parent dir for {}: {}", path.display(), e)
            })?;
        }
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e).into())
    }

    /// Post-process text
    ///
    /// See [utils][crate::utils]
    pub fn normalize(self, op: impl Normalize) -> Self {
        op.normalize(self)
    }

    /// Return the underlying `String`
    ///
    /// Note: this will not inspect binary data for being a valid `String`.
    pub fn render(&self) -> Option<String> {
        match &self.inner {
            DataInner::Error(_) => None,
            DataInner::Binary(_) => None,
            DataInner::Text(data) => Some(data.to_owned()),
            #[cfg(feature = "json")]
            DataInner::Json(value) => Some(serde_json::to_string_pretty(value).unwrap()),
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => Some(data.to_owned()),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::Error> {
        match &self.inner {
            DataInner::Error(err) => Err(err.error.clone()),
            DataInner::Binary(data) => Ok(data.clone()),
            DataInner::Text(data) => Ok(data.clone().into_bytes()),
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                serde_json::to_vec_pretty(value).map_err(|err| format!("{err}").into())
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => Ok(data.clone().into_bytes()),
        }
    }

    /// Initialize `Self` as [`format`][DataFormat] or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    pub fn is(self, format: DataFormat) -> Self {
        match self.try_is(format) {
            Ok(new) => new,
            Err(err) => Self::error(err, format),
        }
    }

    fn try_is(self, format: DataFormat) -> Result<Self, crate::Error> {
        let original = self.format();
        let source = self.source;
        let inner = match (self.inner, format) {
            (DataInner::Error(inner), _) => DataInner::Error(inner),
            (DataInner::Binary(inner), DataFormat::Binary) => DataInner::Binary(inner),
            (DataInner::Text(inner), DataFormat::Text) => DataInner::Text(inner),
            #[cfg(feature = "json")]
            (DataInner::Json(inner), DataFormat::Json) => DataInner::Json(inner),
            #[cfg(feature = "term-svg")]
            (DataInner::TermSvg(inner), DataFormat::TermSvg) => DataInner::TermSvg(inner),
            (DataInner::Binary(inner), _) => {
                let inner = String::from_utf8(inner).map_err(|_err| "invalid UTF-8".to_owned())?;
                Self::text(inner).try_is(format)?.inner
            }
            #[cfg(feature = "json")]
            (DataInner::Text(inner), DataFormat::Json) => {
                let inner = serde_json::from_str::<serde_json::Value>(&inner)
                    .map_err(|err| err.to_string())?;
                DataInner::Json(inner)
            }
            #[cfg(feature = "term-svg")]
            (DataInner::Text(inner), DataFormat::TermSvg) => DataInner::TermSvg(inner),
            (inner, DataFormat::Binary) => {
                let remake: Self = inner.into();
                DataInner::Binary(remake.to_bytes().expect("error case handled"))
            }
            // This variant is already covered unless structured data is enabled
            #[cfg(feature = "structured-data")]
            (inner, DataFormat::Text) => {
                if let Some(str) = Data::from(inner).render() {
                    DataInner::Text(str)
                } else {
                    return Err(format!("cannot convert {original:?} to {format:?}").into());
                }
            }
            (_, _) => return Err(format!("cannot convert {original:?} to {format:?}").into()),
        };
        Ok(Self { inner, source })
    }

    /// Convert `Self` to [`format`][DataFormat] if possible
    ///
    /// This is generally used on `actual` data to make it match `expected`
    pub fn coerce_to(self, format: DataFormat) -> Self {
        let mut data = match (self.inner, format) {
            (DataInner::Error(inner), _) => inner.into(),
            (inner, DataFormat::Error) => inner.into(),
            (DataInner::Binary(inner), DataFormat::Binary) => Self::binary(inner),
            (DataInner::Text(inner), DataFormat::Text) => Self::text(inner),
            #[cfg(feature = "json")]
            (DataInner::Json(inner), DataFormat::Json) => Self::json(inner),
            #[cfg(feature = "term-svg")]
            (DataInner::TermSvg(inner), DataFormat::TermSvg) => inner.into(),
            (DataInner::Binary(inner), _) => {
                if is_binary(&inner) {
                    Self::binary(inner)
                } else {
                    match String::from_utf8(inner) {
                        Ok(str) => {
                            let coerced = Self::text(str).coerce_to(format);
                            // if the Text cannot be coerced into the correct format
                            // reset it back to Binary
                            if coerced.format() != format {
                                coerced.coerce_to(DataFormat::Binary)
                            } else {
                                coerced
                            }
                        }
                        Err(err) => {
                            let bin = err.into_bytes();
                            Self::binary(bin)
                        }
                    }
                }
            }
            #[cfg(feature = "json")]
            (DataInner::Text(inner), DataFormat::Json) => {
                match serde_json::from_str::<serde_json::Value>(&inner) {
                    Ok(json) => Self::json(json),
                    Err(_) => Self::text(inner),
                }
            }
            #[cfg(feature = "term-svg")]
            (DataInner::Text(inner), DataFormat::TermSvg) => {
                DataInner::TermSvg(anstyle_svg::Term::new().render_svg(&inner)).into()
            }
            (inner, DataFormat::Binary) => {
                let remake: Self = inner.into();
                Self::binary(remake.to_bytes().expect("error case handled"))
            }
            // This variant is already covered unless structured data is enabled
            #[cfg(feature = "structured-data")]
            (inner, DataFormat::Text) => {
                let remake: Self = inner.into();
                if let Some(str) = remake.render() {
                    Self::text(str)
                } else {
                    remake
                }
            }
            // reachable if more than one structured data format is enabled
            #[allow(unreachable_patterns)]
            #[cfg(feature = "json")]
            (inner, DataFormat::Json) => inner.into(),
            // reachable if more than one structured data format is enabled
            #[allow(unreachable_patterns)]
            #[cfg(feature = "term-svg")]
            (inner, DataFormat::TermSvg) => inner.into(),
        };
        data.source = self.source;
        data
    }

    /// Outputs the current `DataFormat` of the underlying data
    pub fn format(&self) -> DataFormat {
        match &self.inner {
            DataInner::Error(_) => DataFormat::Error,
            DataInner::Binary(_) => DataFormat::Binary,
            DataInner::Text(_) => DataFormat::Text,
            #[cfg(feature = "json")]
            DataInner::Json(_) => DataFormat::Json,
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(_) => DataFormat::TermSvg,
        }
    }

    pub(crate) fn intended_format(&self) -> DataFormat {
        match &self.inner {
            DataInner::Error(DataError { intended, .. }) => *intended,
            DataInner::Binary(_) => DataFormat::Binary,
            DataInner::Text(_) => DataFormat::Text,
            #[cfg(feature = "json")]
            DataInner::Json(_) => DataFormat::Json,
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(_) => DataFormat::TermSvg,
        }
    }

    pub(crate) fn relevant(&self) -> Option<&str> {
        match &self.inner {
            DataInner::Error(_) => None,
            DataInner::Binary(_) => None,
            DataInner::Text(_) => None,
            #[cfg(feature = "json")]
            DataInner::Json(_) => None,
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => text_elem(data),
        }
    }
}

impl From<DataInner> for Data {
    fn from(inner: DataInner) -> Self {
        Data {
            inner,
            source: None,
        }
    }
}

impl From<DataError> for Data {
    fn from(inner: DataError) -> Self {
        Data {
            inner: DataInner::Error(inner),
            source: None,
        }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            DataInner::Error(data) => data.fmt(f),
            DataInner::Binary(data) => String::from_utf8_lossy(data).fmt(f),
            DataInner::Text(data) => data.fmt(f),
            #[cfg(feature = "json")]
            DataInner::Json(data) => serde_json::to_string_pretty(data).unwrap().fmt(f),
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => data.fmt(f),
        }
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        match (&self.inner, &other.inner) {
            (DataInner::Error(left), DataInner::Error(right)) => left == right,
            (DataInner::Binary(left), DataInner::Binary(right)) => left == right,
            (DataInner::Text(left), DataInner::Text(right)) => left == right,
            #[cfg(feature = "json")]
            (DataInner::Json(left), DataInner::Json(right)) => left == right,
            #[cfg(feature = "term-svg")]
            (DataInner::TermSvg(left), DataInner::TermSvg(right)) => {
                // HACK: avoid including `width` and `height` in the comparison
                let left = text_elem(left.as_str());
                let right = text_elem(right.as_str());
                left == right
            }
            (_, _) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DataError {
    error: crate::Error,
    intended: DataFormat,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

#[cfg(feature = "term-svg")]
fn text_elem(svg: &str) -> Option<&str> {
    let open_elem_start_idx = svg.find("<text")?;
    _ = svg[open_elem_start_idx..].find('>')?;
    let open_elem_line_start_idx = svg[..open_elem_start_idx]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(svg.len());

    let close_elem = "</text>";
    let close_elem_start_idx = svg.rfind(close_elem).unwrap_or(svg.len());
    let close_elem_line_end_idx = svg[close_elem_start_idx..]
        .find('\n')
        .map(|idx| idx + close_elem_start_idx + 1)
        .unwrap_or(svg.len());

    let body = &svg[open_elem_line_start_idx..close_elem_line_end_idx];
    Some(body)
}

impl Eq for Data {}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> From<&'d Data> for Data {
    fn from(other: &'d Data) -> Self {
        other.clone()
    }
}

impl From<Vec<u8>> for Data {
    fn from(other: Vec<u8>) -> Self {
        Self::binary(other)
    }
}

impl<'b> From<&'b [u8]> for Data {
    fn from(other: &'b [u8]) -> Self {
        other.to_owned().into()
    }
}

impl From<String> for Data {
    fn from(other: String) -> Self {
        Self::text(other)
    }
}

impl<'s> From<&'s String> for Data {
    fn from(other: &'s String) -> Self {
        other.clone().into()
    }
}

impl<'s> From<&'s str> for Data {
    fn from(other: &'s str) -> Self {
        other.to_owned().into()
    }
}

#[cfg(feature = "detect-encoding")]
fn is_binary(data: &[u8]) -> bool {
    match content_inspector::inspect(data) {
        content_inspector::ContentType::BINARY |
        // We don't support these
        content_inspector::ContentType::UTF_16LE |
        content_inspector::ContentType::UTF_16BE |
        content_inspector::ContentType::UTF_32LE |
        content_inspector::ContentType::UTF_32BE => {
            true
        },
        content_inspector::ContentType::UTF_8 |
        content_inspector::ContentType::UTF_8_BOM => {
            false
        },
    }
}

#[cfg(not(feature = "detect-encoding"))]
fn is_binary(_data: &[u8]) -> bool {
    false
}

#[doc(hidden)]
pub fn generate_snapshot_path(fn_path: &str, format: Option<DataFormat>) -> std::path::PathBuf {
    use std::fmt::Write as _;

    let fn_path_normalized = fn_path.replace("::", "__");
    let mut path = format!("tests/snapshots/{fn_path_normalized}");
    let count = runtime::get().count(&path);
    if 0 < count {
        write!(&mut path, "@{count}").unwrap();
    }
    path.push('.');
    path.push_str(format.unwrap_or(DataFormat::Text).ext());
    path.into()
}
