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
    pub fn read_from(path: &std::path::Path, binary: Option<bool>) -> Result<Self, crate::Error> {
        let data = match binary {
            Some(true) => {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                Self::binary(data)
            }
            Some(false) => {
                let data = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                Self::text(data)
            }
            None => {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                Self::binary(data).try_text()
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
        std::fs::write(path, self.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e).into())
    }

    /// Update an inline snapshot
    pub fn replace_lines(
        &mut self,
        line_nums: std::ops::Range<usize>,
        text: &str,
    ) -> Result<(), crate::Error> {
        let mut output_lines = String::new();

        let s = self
            .as_str()
            .ok_or("Binary file can't have lines replaced")?;
        for (line_num, line) in crate::utils::LinesWithTerminator::new(s)
            .enumerate()
            .map(|(i, l)| (i + 1, l))
        {
            if line_num == line_nums.start {
                output_lines.push_str(text);
                if !text.is_empty() && !text.ends_with('\n') {
                    output_lines.push('\n');
                }
            }
            if !line_nums.contains(&line_num) {
                output_lines.push_str(line);
            }
        }

        *self = Self::text(output_lines);
        Ok(())
    }

    /// Post-process text
    ///
    /// See [utils][crate::utils]
    pub fn map_text(self, op: impl FnOnce(&str) -> String) -> Self {
        match self.inner {
            DataInner::Binary(data) => Self::binary(data),
            DataInner::Text(data) => Self::text(op(&data)),
        }
    }

    /// Convert from binary to text, if possible
    ///
    /// This will do extra binary file detection if `detect-encoding` feature flag is set
    pub fn try_text(self) -> Self {
        match self.inner {
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
        }
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

    pub fn as_bytes(&self) -> &[u8] {
        match &self.inner {
            DataInner::Binary(data) => data,
            DataInner::Text(data) => data.as_bytes(),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn replace_lines_same_line_count() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World\n";
        let expected = Data::text("One\nWorld\nThree");

        let mut actual = Data::text(input);
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_grow() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World\nTrees\n";
        let expected = Data::text("One\nWorld\nTrees\nThree");

        let mut actual = Data::text(input);
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_shrink() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "";
        let expected = Data::text("One\nThree");

        let mut actual = Data::text(input);
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_no_trailing() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World";
        let expected = Data::text("One\nWorld\nThree");

        let mut actual = Data::text(input);
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_empty_range() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..2;
        let replacement = "World\n";
        let expected = Data::text("One\nWorld\nTwo\nThree");

        let mut actual = Data::text(input);
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }
}
