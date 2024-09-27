//! `actual` and `expected` [`Data`] for testing code

mod filters;
mod format;
mod runtime;
mod source;
#[cfg(test)]
mod tests;

pub use format::DataFormat;
pub use source::DataSource;
pub use source::Inline;
#[doc(hidden)]
pub use source::Position;

use filters::FilterSet;

/// Capture the pretty debug representation of a value
///
/// Note: this is fairly brittle as debug representations are not generally subject to semver
/// guarantees.
///
/// ```rust,no_run
/// use snapbox::ToDebug as _;
///
/// fn some_function() -> usize {
///     // ...
/// # 5
/// }
///
/// let actual = some_function();
/// let expected = snapbox::str![["5"]];
/// snapbox::assert_data_eq!(actual.to_debug(), expected);
/// ```
pub trait ToDebug {
    fn to_debug(&self) -> Data;
}

impl<D: std::fmt::Debug> ToDebug for D {
    fn to_debug(&self) -> Data {
        Data::text(format!("{self:#?}\n"))
    }
}

/// Capture the serde representation of a value
///
/// # Examples
///
/// ```rust,no_run
/// use snapbox::IntoJson as _;
///
/// fn some_function() -> usize {
///     // ...
/// # 5
/// }
///
/// let actual = some_function();
/// let expected = snapbox::str![["5"]];
/// snapbox::assert_data_eq!(actual.into_json(), expected);
/// ```
#[cfg(feature = "json")]
pub trait IntoJson {
    fn into_json(self) -> Data;
}

#[cfg(feature = "json")]
impl<S: serde::Serialize> IntoJson for S {
    fn into_json(self) -> Data {
        match serde_json::to_value(self) {
            Ok(value) => Data::json(value),
            Err(err) => Data::error(err.to_string(), DataFormat::Json),
        }
    }
}

/// Convert to [`Data`] with modifiers for `expected` data
#[allow(clippy::wrong_self_convention)]
pub trait IntoData: Sized {
    /// Remove default [`filters`][crate::filter] from this `expected` result
    fn raw(self) -> Data {
        self.into_data().raw()
    }

    /// Treat lines and json arrays as unordered
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    /// use snapbox::assert_data_eq;
    ///
    /// let actual = str![[r#"["world", "hello"]"#]]
    ///     .is(snapbox::data::DataFormat::Json)
    ///     .unordered();
    /// let expected = str![[r#"["hello", "world"]"#]]
    ///     .is(snapbox::data::DataFormat::Json)
    ///     .unordered();
    /// assert_data_eq!(actual, expected);
    /// # }
    /// ```
    fn unordered(self) -> Data {
        self.into_data().unordered()
    }

    /// Initialize as [`format`][DataFormat] or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .is(snapbox::data::DataFormat::Json);
    /// assert_eq!(expected.format(), snapbox::data::DataFormat::Json);
    /// # }
    /// ```
    fn is(self, format: DataFormat) -> Data {
        self.into_data().is(format)
    }

    /// Initialize as json or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .is_json();
    /// assert_eq!(expected.format(), snapbox::data::DataFormat::Json);
    /// # }
    /// ```
    #[cfg(feature = "json")]
    fn is_json(self) -> Data {
        self.is(DataFormat::Json)
    }

    #[cfg(feature = "json")]
    #[deprecated(since = "0.6.13", note = "Replaced with `IntoData::is_json`")]
    fn json(self) -> Data {
        self.is_json()
    }

    /// Initialize as json lines or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .is_jsonlines();
    /// assert_eq!(expected.format(), snapbox::data::DataFormat::JsonLines);
    /// # }
    /// ```
    #[cfg(feature = "json")]
    fn is_jsonlines(self) -> Data {
        self.is(DataFormat::JsonLines)
    }

    #[cfg(feature = "json")]
    #[deprecated(since = "0.6.13", note = "Replaced with `IntoData::is_jsonlines`")]
    fn json_lines(self) -> Data {
        self.is_jsonlines()
    }

    /// Initialize as Term SVG
    ///
    /// This is generally used for `expected` data
    #[cfg(feature = "term-svg")]
    fn is_termsvg(self) -> Data {
        self.is(DataFormat::TermSvg)
    }

    #[cfg(feature = "term-svg")]
    #[deprecated(since = "0.6.13", note = "Replaced with `IntoData::is_termsvg`")]
    fn term_svg(self) -> Data {
        self.is_termsvg()
    }

    /// Override the type this snapshot will be compared against
    ///
    /// Normally, the `actual` data is coerced to [`IntoData::is`].
    /// This allows overriding that so you can store your snapshot in a more readable, diffable
    /// format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .against(snapbox::data::DataFormat::JsonLines);
    /// # }
    /// ```
    fn against(self, format: DataFormat) -> Data {
        self.into_data().against(format)
    }

    /// Initialize as json or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .is_json();
    /// # }
    /// ```
    #[cfg(feature = "json")]
    fn against_json(self) -> Data {
        self.against(DataFormat::Json)
    }

    /// Initialize as json lines or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .against_jsonlines();
    /// # }
    /// ```
    #[cfg(feature = "json")]
    fn against_jsonlines(self) -> Data {
        self.against(DataFormat::JsonLines)
    }

    /// Convert to [`Data`], applying defaults
    fn into_data(self) -> Data;
}

impl IntoData for Data {
    fn into_data(self) -> Data {
        self
    }
}

impl IntoData for &'_ Data {
    fn into_data(self) -> Data {
        self.clone()
    }
}

impl IntoData for Vec<u8> {
    fn into_data(self) -> Data {
        Data::binary(self)
    }
}

impl IntoData for &'_ [u8] {
    fn into_data(self) -> Data {
        self.to_owned().into_data()
    }
}

impl IntoData for String {
    fn into_data(self) -> Data {
        Data::text(self)
    }
}

impl IntoData for &'_ String {
    fn into_data(self) -> Data {
        self.to_owned().into_data()
    }
}

impl IntoData for &'_ str {
    fn into_data(self) -> Data {
        self.to_owned().into_data()
    }
}

impl IntoData for Inline {
    fn into_data(self) -> Data {
        let trimmed = self.trimmed();
        Data::text(trimmed).with_source(self)
    }
}

/// Declare an expected value for an assert from a file
///
/// This is relative to the source file the macro is run from
///
/// Output type: [`Data`]
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
        let mut path = $crate::utils::current_dir!();
        path.push($path);
        $crate::Data::read_from(&path, None)
    }};
    [$path:literal : $type:ident] => {{
        let mut path = $crate::utils::current_dir!();
        path.push($path);
        $crate::Data::read_from(&path, Some($crate::data::DataFormat:: $type))
    }};
}

/// Declare an expected value from within Rust source
///
/// Output type: [`Inline`], see [`IntoData`] for operations
///
/// ```
/// # use snapbox::str;
/// str![["
///     Foo { value: 92 }
/// "]];
/// str![r#"{"Foo": 92}"#];
/// ```
#[macro_export]
macro_rules! str {
    [$data:literal] => { $crate::str![[$data]] };
    [[$data:literal]] => {{
        let position = $crate::data::Position {
            file: $crate::utils::current_rs!(),
            line: line!(),
            column: column!(),
        };
        let inline = $crate::data::Inline {
            position,
            data: $data,
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
    pub(crate) inner: DataInner,
    pub(crate) source: Option<DataSource>,
    pub(crate) filters: FilterSet,
}

#[derive(Clone, Debug)]
pub(crate) enum DataInner {
    Error(DataError),
    Binary(Vec<u8>),
    Text(String),
    #[cfg(feature = "json")]
    Json(serde_json::Value),
    // Always a `Value::Array` but using `Value` for easier bookkeeping
    #[cfg(feature = "json")]
    JsonLines(serde_json::Value),
    #[cfg(feature = "term-svg")]
    TermSvg(String),
}

/// # Constructors
///
/// See also
/// - [`str!`] for inline snapshots
/// - [`file!`] for external snapshots
/// - [`ToString`] for verifying a `Display` representation
/// - [`ToDebug`] for verifying a debug representation
/// - [`IntoJson`] for verifying the serde representation
/// - [`IntoData`] for modifying `expected`
impl Data {
    /// Mark the data as binary (no post-processing)
    pub fn binary(raw: impl Into<Vec<u8>>) -> Self {
        Self::with_inner(DataInner::Binary(raw.into()))
    }

    /// Mark the data as text (post-processing)
    pub fn text(raw: impl Into<String>) -> Self {
        Self::with_inner(DataInner::Text(raw.into()))
    }

    #[cfg(feature = "json")]
    pub fn json(raw: impl Into<serde_json::Value>) -> Self {
        Self::with_inner(DataInner::Json(raw.into()))
    }

    #[cfg(feature = "json")]
    pub fn jsonlines(raw: impl Into<Vec<serde_json::Value>>) -> Self {
        Self::with_inner(DataInner::JsonLines(serde_json::Value::Array(raw.into())))
    }

    fn error(raw: impl Into<crate::assert::Error>, intended: DataFormat) -> Self {
        Self::with_inner(DataInner::Error(DataError {
            error: raw.into(),
            intended,
        }))
    }

    /// Empty test data
    pub fn new() -> Self {
        Self::text("")
    }

    /// Load `expected` data from a file
    pub fn read_from(path: &std::path::Path, data_format: Option<DataFormat>) -> Self {
        match Self::try_read_from(path, data_format) {
            Ok(data) => data,
            Err(err) => Self::error(err, data_format.unwrap_or_else(|| DataFormat::from(path)))
                .with_path(path),
        }
    }

    /// Remove default [`filters`][crate::filter] from this `expected` result
    pub fn raw(mut self) -> Self {
        self.filters = FilterSet::empty().newlines();
        self
    }

    /// Treat lines and json arrays as unordered
    pub fn unordered(mut self) -> Self {
        self.filters = self.filters.unordered();
        self
    }
}

/// # Assertion frameworks operations
///
/// For example, see [`OutputAssert`][crate::cmd::OutputAssert]
impl Data {
    pub(crate) fn with_inner(inner: DataInner) -> Self {
        Self {
            inner,
            source: None,
            filters: FilterSet::new(),
        }
    }

    fn with_source(mut self, source: impl Into<DataSource>) -> Self {
        self.source = Some(source.into());
        self
    }

    fn with_path(self, path: impl Into<std::path::PathBuf>) -> Self {
        self.with_source(path.into())
    }

    /// Load `expected` data from a file
    pub fn try_read_from(
        path: &std::path::Path,
        data_format: Option<DataFormat>,
    ) -> crate::assert::Result<Self> {
        let data =
            std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let data = Self::binary(data);
        let data = match data_format {
            Some(df) => data.is(df),
            None => {
                let inferred_format = DataFormat::from(path);
                match inferred_format {
                    #[cfg(feature = "json")]
                    DataFormat::Json | DataFormat::JsonLines => data.coerce_to(inferred_format),
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

    /// Overwrite a snapshot
    pub fn write_to(&self, source: &DataSource) -> crate::assert::Result<()> {
        match &source.inner {
            source::DataSourceInner::Path(p) => self.write_to_path(p),
            source::DataSourceInner::Inline(p) => runtime::get()
                .write(self, p)
                .map_err(|err| err.to_string().into()),
        }
    }

    /// Overwrite a snapshot
    pub fn write_to_path(&self, path: &std::path::Path) -> crate::assert::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create parent dir for {}: {}", path.display(), e)
            })?;
        }
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e).into())
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
            DataInner::Json(_) => Some(self.to_string()),
            #[cfg(feature = "json")]
            DataInner::JsonLines(_) => Some(self.to_string()),
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => Some(data.to_owned()),
        }
    }

    pub fn to_bytes(&self) -> crate::assert::Result<Vec<u8>> {
        match &self.inner {
            DataInner::Error(err) => Err(err.error.clone()),
            DataInner::Binary(data) => Ok(data.clone()),
            DataInner::Text(data) => Ok(data.clone().into_bytes()),
            #[cfg(feature = "json")]
            DataInner::Json(_) => Ok(self.to_string().into_bytes()),
            #[cfg(feature = "json")]
            DataInner::JsonLines(_) => Ok(self.to_string().into_bytes()),
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => Ok(data.clone().into_bytes()),
        }
    }

    /// Initialize `Self` as [`format`][DataFormat] or [`Error`][DataFormat::Error]
    ///
    /// This is generally used for `expected` data
    pub fn is(self, format: DataFormat) -> Self {
        let filters = self.filters;
        let source = self.source.clone();
        match self.try_is(format) {
            Ok(new) => new,
            Err(err) => {
                let inner = DataInner::Error(DataError {
                    error: err,
                    intended: format,
                });
                Self {
                    inner,
                    source,
                    filters,
                }
            }
        }
    }

    fn try_is(self, format: DataFormat) -> crate::assert::Result<Self> {
        let original = self.format();
        let source = self.source;
        let filters = self.filters;
        let inner = match (self.inner, format) {
            (DataInner::Error(inner), _) => DataInner::Error(inner),
            (DataInner::Binary(inner), DataFormat::Binary) => DataInner::Binary(inner),
            (DataInner::Text(inner), DataFormat::Text) => DataInner::Text(inner),
            #[cfg(feature = "json")]
            (DataInner::Json(inner), DataFormat::Json) => DataInner::Json(inner),
            #[cfg(feature = "json")]
            (DataInner::JsonLines(inner), DataFormat::JsonLines) => DataInner::JsonLines(inner),
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
            #[cfg(feature = "json")]
            (DataInner::Text(inner), DataFormat::JsonLines) => {
                let inner = parse_jsonlines(&inner).map_err(|err| err.to_string())?;
                DataInner::JsonLines(serde_json::Value::Array(inner))
            }
            #[cfg(feature = "term-svg")]
            (DataInner::Text(inner), DataFormat::TermSvg) => DataInner::TermSvg(inner),
            (inner, DataFormat::Binary) => {
                let remake = Self::with_inner(inner);
                DataInner::Binary(remake.to_bytes().expect("error case handled"))
            }
            // This variant is already covered unless structured data is enabled
            #[cfg(feature = "structured-data")]
            (inner, DataFormat::Text) => {
                if let Some(str) = Self::with_inner(inner).render() {
                    DataInner::Text(str)
                } else {
                    return Err(format!("cannot convert {original:?} to {format:?}").into());
                }
            }
            (_, _) => return Err(format!("cannot convert {original:?} to {format:?}").into()),
        };
        Ok(Self {
            inner,
            source,
            filters,
        })
    }

    /// Override the type this snapshot will be compared against
    ///
    /// Normally, the `actual` data is coerced to [`Data::is`].
    /// This allows overriding that so you can store your snapshot in a more readable, diffable
    /// format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "json")] {
    /// use snapbox::prelude::*;
    /// use snapbox::str;
    ///
    /// let expected = str![[r#"{"hello": "world"}"#]]
    ///     .is(snapbox::data::DataFormat::Json)
    ///     .against(snapbox::data::DataFormat::JsonLines);
    /// # }
    /// ```
    fn against(mut self, format: DataFormat) -> Data {
        self.filters = self.filters.against(format);
        self
    }

    /// Convert `Self` to [`format`][DataFormat] if possible
    ///
    /// This is generally used on `actual` data to make it match `expected`
    pub fn coerce_to(self, format: DataFormat) -> Self {
        let source = self.source;
        let filters = self.filters;
        let inner = match (self.inner, format) {
            (DataInner::Error(inner), _) => DataInner::Error(inner),
            (inner, DataFormat::Error) => inner,
            (DataInner::Binary(inner), DataFormat::Binary) => DataInner::Binary(inner),
            (DataInner::Text(inner), DataFormat::Text) => DataInner::Text(inner),
            #[cfg(feature = "json")]
            (DataInner::Json(inner), DataFormat::Json) => DataInner::Json(inner),
            #[cfg(feature = "json")]
            (DataInner::JsonLines(inner), DataFormat::JsonLines) => DataInner::JsonLines(inner),
            #[cfg(feature = "json")]
            (DataInner::JsonLines(inner), DataFormat::Json) => DataInner::Json(inner),
            #[cfg(feature = "json")]
            (DataInner::Json(inner), DataFormat::JsonLines) => DataInner::JsonLines(inner),
            #[cfg(feature = "term-svg")]
            (DataInner::TermSvg(inner), DataFormat::TermSvg) => DataInner::TermSvg(inner),
            (DataInner::Binary(inner), _) => {
                if is_binary(&inner) {
                    DataInner::Binary(inner)
                } else {
                    match String::from_utf8(inner) {
                        Ok(str) => {
                            let coerced = Self::text(str).coerce_to(format);
                            // if the Text cannot be coerced into the correct format
                            // reset it back to Binary
                            let coerced = if coerced.format() != format {
                                coerced.coerce_to(DataFormat::Binary)
                            } else {
                                coerced
                            };
                            coerced.inner
                        }
                        Err(err) => {
                            let bin = err.into_bytes();
                            DataInner::Binary(bin)
                        }
                    }
                }
            }
            #[cfg(feature = "json")]
            (DataInner::Text(inner), DataFormat::Json) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&inner) {
                    DataInner::Json(json)
                } else {
                    DataInner::Text(inner)
                }
            }
            #[cfg(feature = "json")]
            (DataInner::Text(inner), DataFormat::JsonLines) => {
                if let Ok(jsonlines) = parse_jsonlines(&inner) {
                    DataInner::JsonLines(serde_json::Value::Array(jsonlines))
                } else {
                    DataInner::Text(inner)
                }
            }
            #[cfg(feature = "term-svg")]
            (DataInner::Text(inner), DataFormat::TermSvg) => {
                DataInner::TermSvg(anstyle_svg::Term::new().render_svg(&inner))
            }
            (inner, DataFormat::Binary) => {
                let remake = Self::with_inner(inner);
                DataInner::Binary(remake.to_bytes().expect("error case handled"))
            }
            // This variant is already covered unless structured data is enabled
            #[cfg(feature = "structured-data")]
            (inner, DataFormat::Text) => {
                let remake = Self::with_inner(inner);
                if let Some(str) = remake.render() {
                    DataInner::Text(str)
                } else {
                    remake.inner
                }
            }
            // reachable if more than one structured data format is enabled
            #[allow(unreachable_patterns)]
            #[cfg(feature = "json")]
            (inner, DataFormat::Json) => inner,
            // reachable if more than one structured data format is enabled
            #[allow(unreachable_patterns)]
            #[cfg(feature = "json")]
            (inner, DataFormat::JsonLines) => inner,
            // reachable if more than one structured data format is enabled
            #[allow(unreachable_patterns)]
            #[cfg(feature = "term-svg")]
            (inner, DataFormat::TermSvg) => inner,
        };
        Self {
            inner,
            source,
            filters,
        }
    }

    /// Location the data came from
    pub fn source(&self) -> Option<&DataSource> {
        self.source.as_ref()
    }

    /// Outputs the current `DataFormat` of the underlying data
    pub fn format(&self) -> DataFormat {
        match &self.inner {
            DataInner::Error(_) => DataFormat::Error,
            DataInner::Binary(_) => DataFormat::Binary,
            DataInner::Text(_) => DataFormat::Text,
            #[cfg(feature = "json")]
            DataInner::Json(_) => DataFormat::Json,
            #[cfg(feature = "json")]
            DataInner::JsonLines(_) => DataFormat::JsonLines,
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
            #[cfg(feature = "json")]
            DataInner::JsonLines(_) => DataFormat::JsonLines,
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(_) => DataFormat::TermSvg,
        }
    }

    pub(crate) fn against_format(&self) -> DataFormat {
        self.filters
            .get_against()
            .unwrap_or_else(|| self.intended_format())
    }

    pub(crate) fn relevant(&self) -> Option<&str> {
        match &self.inner {
            DataInner::Error(_) => None,
            DataInner::Binary(_) => None,
            DataInner::Text(_) => None,
            #[cfg(feature = "json")]
            DataInner::Json(_) => None,
            #[cfg(feature = "json")]
            DataInner::JsonLines(_) => None,
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(data) => term_svg_body(data),
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
            #[cfg(feature = "json")]
            DataInner::JsonLines(data) => {
                let array = data.as_array().expect("jsonlines is always an array");
                for value in array {
                    writeln!(f, "{}", serde_json::to_string(value).unwrap())?;
                }
                Ok(())
            }
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
            #[cfg(feature = "json")]
            (DataInner::JsonLines(left), DataInner::JsonLines(right)) => left == right,
            #[cfg(feature = "term-svg")]
            (DataInner::TermSvg(left), DataInner::TermSvg(right)) => {
                // HACK: avoid including `width` and `height` in the comparison
                let left = term_svg_body(left.as_str()).unwrap_or(left.as_str());
                let right = term_svg_body(right.as_str()).unwrap_or(right.as_str());
                left == right
            }
            (_, _) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DataError {
    error: crate::assert::Error,
    intended: DataFormat,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

#[cfg(feature = "json")]
fn parse_jsonlines(text: &str) -> Result<Vec<serde_json::Value>, serde_json::Error> {
    let mut lines = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let json = serde_json::from_str::<serde_json::Value>(line)?;
        lines.push(json);
    }
    Ok(lines)
}

#[cfg(feature = "term-svg")]
fn term_svg_body(svg: &str) -> Option<&str> {
    let (_header, body, _footer) = split_term_svg(svg)?;
    Some(body)
}

#[cfg(feature = "term-svg")]
pub(crate) fn split_term_svg(svg: &str) -> Option<(&str, &str, &str)> {
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

    let header = &svg[..open_elem_line_start_idx];
    let body = &svg[open_elem_line_start_idx..close_elem_line_end_idx];
    let footer = &svg[close_elem_line_end_idx..];
    Some((header, body, footer))
}

impl Eq for Data {}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl<'d> From<&'d Data> for Data {
    fn from(other: &'d Data) -> Self {
        other.into_data()
    }
}

impl From<Vec<u8>> for Data {
    fn from(other: Vec<u8>) -> Self {
        other.into_data()
    }
}

impl<'b> From<&'b [u8]> for Data {
    fn from(other: &'b [u8]) -> Self {
        other.into_data()
    }
}

impl From<String> for Data {
    fn from(other: String) -> Self {
        other.into_data()
    }
}

impl<'s> From<&'s String> for Data {
    fn from(other: &'s String) -> Self {
        other.into_data()
    }
}

impl<'s> From<&'s str> for Data {
    fn from(other: &'s str) -> Self {
        other.into_data()
    }
}

impl From<Inline> for Data {
    fn from(other: Inline) -> Self {
        other.into_data()
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

#[cfg(test)]
mod test {
    use super::*;

    #[track_caller]
    fn validate_cases(cases: &[(&str, bool)], input_format: DataFormat) {
        for (input, valid) in cases.iter().copied() {
            let (expected_is_format, expected_coerced_format) = if valid {
                (input_format, input_format)
            } else {
                (DataFormat::Error, DataFormat::Text)
            };

            let actual_is = Data::text(input).is(input_format);
            assert_eq!(
                actual_is.format(),
                expected_is_format,
                "\n{input}\n{actual_is}"
            );

            let actual_coerced = Data::text(input).coerce_to(input_format);
            assert_eq!(
                actual_coerced.format(),
                expected_coerced_format,
                "\n{input}\n{actual_coerced}"
            );

            if valid {
                assert_eq!(actual_is, actual_coerced);

                let rendered = actual_is.render().unwrap();
                let bytes = actual_is.to_bytes().unwrap();
                assert_eq!(rendered, std::str::from_utf8(&bytes).unwrap());

                assert_eq!(Data::text(&rendered).is(input_format), actual_is);
            }
        }
    }

    #[test]
    fn text() {
        let cases = [("", true), ("good", true), ("{}", true), ("\"\"", true)];
        validate_cases(&cases, DataFormat::Text);
    }

    #[cfg(feature = "json")]
    #[test]
    fn json() {
        let cases = [("", false), ("bad", false), ("{}", true), ("\"\"", true)];
        validate_cases(&cases, DataFormat::Json);
    }

    #[cfg(feature = "json")]
    #[test]
    fn jsonlines() {
        let cases = [
            ("", true),
            ("bad", false),
            ("{}", true),
            ("\"\"", true),
            (
                "
{}
{}
", true,
            ),
            (
                "
{}

{}
", true,
            ),
            (
                "
{}
bad
{}
",
                false,
            ),
        ];
        validate_cases(&cases, DataFormat::JsonLines);
    }
}
