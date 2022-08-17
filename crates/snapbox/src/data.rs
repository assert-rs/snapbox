/// Test fixture, actual output, or expected result
///
/// This provides conveniences for tracking the intended format (binary vs text).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    inner: DataInner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DataInner {
    Binary(Vec<u8>),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum DataFormat {
    Binary,
    Text,
}

impl Default for DataFormat {
    fn default() -> Self {
        DataFormat::Text
    }
}

impl Data {
    /// Mark the data as binary (no post-processing)
    pub fn binary(raw: impl Into<Vec<u8>>) -> Self {
        Self {
            inner: DataInner::Binary(raw.into()),
        }
    }

    /// Mark the data as text (post-processing)
    pub fn text(raw: impl Into<String>) -> Self {
        Self {
            inner: DataInner::Text(raw.into()),
        }
    }

    /// Empty test data
    pub fn new() -> Self {
        Self::text("")
    }

    /// Load test data from a file
    pub fn read_from(
        path: &std::path::Path,
        data_format: Option<DataFormat>,
    ) -> Result<Self, crate::Error> {
        let data = match data_format {
            Some(df) => match df {
                DataFormat::Binary => {
                    let data = std::fs::read(&path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    Self::binary(data)
                }
                DataFormat::Text => {
                    let data = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    Self::text(data)
                }
            },
            None => {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                Self::binary(data).try_coerce(DataFormat::Text)
            }
        };
        Ok(data)
    }

    /// Overwrite a snapshot
    pub fn write_to(&self, path: &std::path::Path) -> Result<(), crate::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create parent dir for {}: {}", path.display(), e)
            })?;
        }
        std::fs::write(path, self.to_bytes())
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e).into())
    }

    /// Post-process text
    ///
    /// See [utils][crate::utils]
    pub fn normalize(self, op: impl Normalize) -> Self {
        op.normalize(self)
    }

    /// Coerce to a string
    ///
    /// Note: this will **not** do a binary-content check
    pub fn make_text(&mut self) -> Result<(), std::str::Utf8Error> {
        *self = Self::text(std::mem::take(self).into_string()?);
        Ok(())
    }

    /// Coerce to a string
    ///
    /// Note: this will **not** do a binary-content check
    pub fn into_string(self) -> Result<String, std::str::Utf8Error> {
        match self.inner {
            DataInner::Binary(data) => {
                let data = String::from_utf8(data).map_err(|e| e.utf8_error())?;
                Ok(data)
            }
            DataInner::Text(data) => Ok(data),
        }
    }

    /// Return the underlying `str`
    ///
    /// Note: this will not inspect binary data for being a valid `str`.
    pub fn as_str(&self) -> Option<&str> {
        match &self.inner {
            DataInner::Binary(_) => None,
            DataInner::Text(data) => Some(data.as_str()),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self.inner {
            DataInner::Binary(data) => data.clone(),
            DataInner::Text(data) => data.clone().into_bytes(),
        }
    }

    pub fn try_coerce(self, format: DataFormat) -> Self {
        match format {
            DataFormat::Binary => Self::binary(self.to_bytes()),
            DataFormat::Text => match self.inner {
                DataInner::Binary(data) => {
                    if is_binary(&data) {
                        Self::binary(data)
                    } else {
                        match String::from_utf8(data) {
                            Ok(data) => Self::text(data),
                            Err(err) => {
                                let data = err.into_bytes();
                                Self::binary(data)
                            }
                        }
                    }
                }
                DataInner::Text(data) => Self::text(data),
            },
        }
    }

    /// Outputs the current `DataFormat` of the underlying data
    pub fn format(&self) -> DataFormat {
        match &self.inner {
            DataInner::Binary(_) => DataFormat::Binary,
            DataInner::Text(_) => DataFormat::Text,
        }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            DataInner::Binary(data) => String::from_utf8_lossy(data).fmt(f),
            DataInner::Text(data) => data.fmt(f),
        }
    }
}

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

pub trait Normalize {
    fn normalize(&self, data: Data) -> Data;
}

pub struct NormalizeNewlines;
impl Normalize for NormalizeNewlines {
    fn normalize(&self, data: Data) -> Data {
        match data.inner {
            DataInner::Binary(bin) => Data::binary(bin),
            DataInner::Text(text) => {
                let lines = crate::utils::normalize_lines(&text);
                Data::text(lines)
            }
        }
    }
}

pub struct NormalizePaths;
impl Normalize for NormalizePaths {
    fn normalize(&self, data: Data) -> Data {
        match data.inner {
            DataInner::Binary(bin) => Data::binary(bin),
            DataInner::Text(text) => {
                let lines = crate::utils::normalize_paths(&text);
                Data::text(lines)
            }
        }
    }
}

pub struct NormalizeMatches<'a> {
    substitutions: &'a crate::Substitutions,
    pattern: &'a Data,
}

impl<'a> NormalizeMatches<'a> {
    pub fn new(substitutions: &'a crate::Substitutions, pattern: &'a Data) -> Self {
        NormalizeMatches {
            substitutions,
            pattern,
        }
    }
}

impl Normalize for NormalizeMatches<'_> {
    fn normalize(&self, data: Data) -> Data {
        match data.inner {
            DataInner::Binary(bin) => Data::binary(bin),
            DataInner::Text(text) => {
                let lines = self
                    .substitutions
                    .normalize(&text, self.pattern.as_str().unwrap());
                Data::text(lines)
            }
        }
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
