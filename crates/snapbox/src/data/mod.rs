mod format;
mod normalize;
mod source;
#[cfg(test)]
mod tests;

pub use format::DataFormat;
pub use normalize::Normalize;
pub use normalize::NormalizeMatches;
pub use normalize::NormalizeNewlines;
pub use normalize::NormalizePaths;
pub use source::DataSource;

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
        let stem = ::std::path::Path::new(::std::file!()).file_stem().unwrap();
        let rel_path = ::std::format!("snapshots/{}-{}.txt", stem.to_str().unwrap(), line!());
        let mut path = $crate::current_dir!();
        path.push(rel_path);
        $crate::Data::read_from(&path, None)
    }};
    [_ : $type:ident] => {{
        let stem = ::std::path::Path::new(::std::file!()).file_stem().unwrap();
        let ext = $crate::data::DataFormat:: $type.ext();
        let rel_path = ::std::format!("snapshots/{}-{}.{ext}", stem.to_str().unwrap(), line!());
        let mut path = $crate::current_dir!();
        path.push(rel_path);
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

/// Test fixture, actual output, or expected result
///
/// This provides conveniences for tracking the intended format (binary vs text).
#[derive(Clone, Debug)]
pub struct Data {
    inner: DataInner,
    source: Option<DataSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DataInner {
    Error(crate::Error),
    Binary(Vec<u8>),
    Text(String),
    #[cfg(feature = "json")]
    Json(serde_json::Value),
}

impl Data {
    /// Mark the data as binary (no post-processing)
    pub fn binary(raw: impl Into<Vec<u8>>) -> Self {
        Self {
            inner: DataInner::Binary(raw.into()),
            source: None,
        }
    }

    /// Mark the data as text (post-processing)
    pub fn text(raw: impl Into<String>) -> Self {
        Self {
            inner: DataInner::Text(raw.into()),
            source: None,
        }
    }

    #[cfg(feature = "json")]
    pub fn json(raw: impl Into<serde_json::Value>) -> Self {
        Self {
            inner: DataInner::Json(raw.into()),
            source: None,
        }
    }

    fn error(raw: impl Into<crate::Error>) -> Self {
        Self {
            inner: DataInner::Error(raw.into()),
            source: None,
        }
    }

    /// Empty test data
    pub fn new() -> Self {
        Self::text("")
    }

    fn with_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.source = Some(DataSource::path(path));
        self
    }

    /// Load test data from a file
    pub fn read_from(path: &std::path::Path, data_format: Option<DataFormat>) -> Self {
        match Self::try_read_from(path, data_format) {
            Ok(data) => data,
            Err(err) => Self::error(err),
        }
    }

    /// Load test data from a file
    pub fn try_read_from(
        path: &std::path::Path,
        data_format: Option<DataFormat>,
    ) -> Result<Self, crate::Error> {
        let data = match data_format {
            Some(df) => match df {
                DataFormat::Error => Self::error("unknown error"),
                DataFormat::Binary => {
                    let data = std::fs::read(path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    Self::binary(data)
                }
                DataFormat::Text => {
                    let data = std::fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    Self::text(data)
                }
                #[cfg(feature = "json")]
                DataFormat::Json => {
                    let data = std::fs::read_to_string(path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    Self::json(serde_json::from_str::<serde_json::Value>(&data).unwrap())
                }
            },
            None => {
                let data = std::fs::read(path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                let data = Self::binary(data);
                match path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                {
                    #[cfg(feature = "json")]
                    "json" => data.try_coerce(DataFormat::Json),
                    _ => data.try_coerce(DataFormat::Text),
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
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::Error> {
        match &self.inner {
            DataInner::Error(err) => Err(err.clone()),
            DataInner::Binary(data) => Ok(data.clone()),
            DataInner::Text(data) => Ok(data.clone().into_bytes()),
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                serde_json::to_vec_pretty(value).map_err(|err| format!("{err}").into())
            }
        }
    }

    pub fn try_coerce(self, format: DataFormat) -> Self {
        let mut data = match (self.inner, format) {
            (DataInner::Error(inner), _) => Self::error(inner),
            (inner, DataFormat::Error) => Self {
                inner,
                source: None,
            },
            (DataInner::Binary(inner), DataFormat::Binary) => Self::binary(inner),
            (DataInner::Text(inner), DataFormat::Text) => Self::text(inner),
            #[cfg(feature = "json")]
            (DataInner::Json(inner), DataFormat::Json) => Self::json(inner),
            (DataInner::Binary(inner), _) => {
                if is_binary(&inner) {
                    Self::binary(inner)
                } else {
                    match String::from_utf8(inner) {
                        Ok(str) => {
                            let coerced = Self::text(str).try_coerce(format);
                            // if the Text cannot be coerced into the correct format
                            // reset it back to Binary
                            if coerced.format() != format {
                                coerced.try_coerce(DataFormat::Binary)
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
            (inner, DataFormat::Binary) => Self::binary(
                Self {
                    inner,
                    source: None,
                }
                .to_bytes()
                .expect("error case handled"),
            ),
            // This variant is already covered unless structured data is enabled
            #[cfg(feature = "structured-data")]
            (inner, DataFormat::Text) => {
                let remake = Self {
                    inner,
                    source: None,
                };
                if let Some(str) = remake.render() {
                    Self::text(str)
                } else {
                    remake
                }
            }
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
        }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            DataInner::Error(err) => err.fmt(f),
            DataInner::Binary(data) => String::from_utf8_lossy(data).fmt(f),
            DataInner::Text(data) => data.fmt(f),
            #[cfg(feature = "json")]
            DataInner::Json(data) => serde_json::to_string_pretty(data).unwrap().fmt(f),
        }
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        self.inner == other.inner
    }
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
