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
    #[cfg(feature = "json")]
    Json(serde_json::Value),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, Default)]
pub enum DataFormat {
    Binary,
    #[default]
    Text,
    #[cfg(feature = "json")]
    Json,
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

    #[cfg(feature = "json")]
    pub fn json(raw: impl Into<serde_json::Value>) -> Self {
        Self {
            inner: DataInner::Json(raw.into()),
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

    /// Return the underlying `String`
    ///
    /// Note: this will not inspect binary data for being a valid `String`.
    pub fn render(&self) -> Option<String> {
        match &self.inner {
            DataInner::Binary(_) => None,
            DataInner::Text(data) => Some(data.to_owned()),
            #[cfg(feature = "json")]
            DataInner::Json(value) => Some(serde_json::to_string_pretty(value).unwrap()),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self.inner {
            DataInner::Binary(data) => data.clone(),
            DataInner::Text(data) => data.clone().into_bytes(),
            #[cfg(feature = "json")]
            DataInner::Json(value) => serde_json::to_vec_pretty(value).unwrap(),
        }
    }

    pub fn try_coerce(self, format: DataFormat) -> Self {
        match (self.inner, format) {
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
            (inner, DataFormat::Binary) => Self::binary(Self { inner }.to_bytes()),
            // This variant is already covered unless structured data is enabled
            #[cfg(feature = "structured-data")]
            (inner, DataFormat::Text) => {
                let remake = Self { inner };
                if let Some(str) = remake.render() {
                    Self::text(str)
                } else {
                    remake
                }
            }
        }
    }

    /// Outputs the current `DataFormat` of the underlying data
    pub fn format(&self) -> DataFormat {
        match &self.inner {
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
            DataInner::Binary(data) => String::from_utf8_lossy(data).fmt(f),
            DataInner::Text(data) => data.fmt(f),
            #[cfg(feature = "json")]
            DataInner::Json(data) => serde_json::to_string_pretty(data).unwrap().fmt(f),
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
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                normalize_value(&mut value, crate::utils::normalize_lines);
                Data::json(value)
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
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                normalize_value(&mut value, crate::utils::normalize_paths);
                Data::json(value)
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
                    .normalize(&text, &self.pattern.render().unwrap());
                Data::text(lines)
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                if let DataInner::Json(exp) = &self.pattern.inner {
                    normalize_value_matches(&mut value, exp, self.substitutions);
                }
                Data::json(value)
            }
        }
    }
}

#[cfg(feature = "structured-data")]
fn normalize_value(value: &mut serde_json::Value, op: fn(&str) -> String) {
    match value {
        serde_json::Value::String(str) => {
            *str = op(str);
        }
        serde_json::Value::Array(arr) => {
            for value in arr.iter_mut() {
                normalize_value(value, op)
            }
        }
        serde_json::Value::Object(obj) => {
            for (_, value) in obj.iter_mut() {
                normalize_value(value, op)
            }
        }
        _ => {}
    }
}

#[cfg(feature = "structured-data")]
fn normalize_value_matches(
    actual: &mut serde_json::Value,
    expected: &serde_json::Value,
    substitutions: &crate::Substitutions,
) {
    use serde_json::Value::*;
    match (actual, expected) {
        // "{...}" is a wildcard
        (act, String(exp)) if exp == "{...}" => {
            *act = serde_json::json!("{...}");
        }
        (String(act), String(exp)) => {
            *act = substitutions.normalize(act, exp);
        }
        (Array(act), Array(exp)) => {
            let wildcard = String("{...}".to_string());
            let mut sections = exp.split(|e| e == &wildcard).peekable();
            let mut processed = 0;
            while let Some(expected_subset) = sections.next() {
                // Process all values in the current section
                if !expected_subset.is_empty() {
                    let actual_subset = &mut act[processed..processed + expected_subset.len()];
                    for (a, e) in actual_subset.iter_mut().zip(expected_subset) {
                        normalize_value_matches(a, e, substitutions);
                    }
                    processed += expected_subset.len();
                }

                if let Some(next_section) = sections.peek() {
                    // If the next section has nothing in it, replace from processed to end with
                    // a single "{...}"
                    if next_section.is_empty() {
                        act.splice(processed.., vec![wildcard.clone()]);
                        processed += 1;
                    } else {
                        let first = next_section.first().unwrap();
                        // Replace everything up until the value we are looking for with
                        // a single "{...}".
                        if let Some(index) = act.iter().position(|v| v == first) {
                            act.splice(processed..index, vec![wildcard.clone()]);
                            processed += 1;
                        } else {
                            // If we cannot find the value we are looking for return early
                            break;
                        }
                    }
                }
            }
        }
        (Object(act), Object(exp)) => {
            for (a, e) in act.iter_mut().zip(exp).filter(|(a, e)| a.0 == e.0) {
                normalize_value_matches(a.1, e.1, substitutions)
            }
        }
        (_, _) => {}
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
    #[cfg(feature = "json")]
    use serde_json::json;

    // Tests for checking to_bytes and render produce the same results
    #[test]
    fn text_to_bytes_render() {
        let d = Data::text(String::from("test"));
        let bytes = d.to_bytes();
        let bytes = String::from_utf8(bytes).unwrap();
        let rendered = d.render().unwrap();
        assert_eq!(bytes, rendered);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_to_bytes_render() {
        let d = Data::json(json!({"name": "John\\Doe\r\n"}));
        let bytes = d.to_bytes();
        let bytes = String::from_utf8(bytes).unwrap();
        let rendered = d.render().unwrap();
        assert_eq!(bytes, rendered);
    }

    // Tests for checking all types are coercible to each other and
    // for when the coercion should fail
    #[test]
    fn binary_to_text() {
        let binary = String::from("test").into_bytes();
        let d = Data::binary(binary);
        let text = d.try_coerce(DataFormat::Text);
        assert_eq!(DataFormat::Text, text.format())
    }

    #[test]
    fn binary_to_text_not_utf8() {
        let binary = b"\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00".to_vec();
        let d = Data::binary(binary);
        let d = d.try_coerce(DataFormat::Text);
        assert_ne!(DataFormat::Text, d.format());
        assert_eq!(DataFormat::Binary, d.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn binary_to_json() {
        let value = json!({"name": "John\\Doe\r\n"});
        let binary = serde_json::to_vec_pretty(&value).unwrap();
        let d = Data::binary(binary);
        let json = d.try_coerce(DataFormat::Json);
        assert_eq!(DataFormat::Json, json.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn binary_to_json_not_utf8() {
        let binary = b"\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00".to_vec();
        let d = Data::binary(binary);
        let d = d.try_coerce(DataFormat::Json);
        assert_ne!(DataFormat::Json, d.format());
        assert_eq!(DataFormat::Binary, d.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn binary_to_json_not_json() {
        let binary = String::from("test").into_bytes();
        let d = Data::binary(binary);
        let d = d.try_coerce(DataFormat::Json);
        assert_ne!(DataFormat::Json, d.format());
        assert_eq!(DataFormat::Binary, d.format());
    }

    #[test]
    fn text_to_binary() {
        let text = String::from("test");
        let d = Data::text(text);
        let binary = d.try_coerce(DataFormat::Binary);
        assert_eq!(DataFormat::Binary, binary.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn text_to_json() {
        let value = json!({"name": "John\\Doe\r\n"});
        let text = serde_json::to_string_pretty(&value).unwrap();
        let d = Data::text(text);
        let json = d.try_coerce(DataFormat::Json);
        assert_eq!(DataFormat::Json, json.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn text_to_json_not_json() {
        let text = String::from("test");
        let d = Data::text(text);
        let json = d.try_coerce(DataFormat::Json);
        assert_eq!(DataFormat::Text, json.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_to_binary() {
        let value = json!({"name": "John\\Doe\r\n"});
        let d = Data::json(value);
        let binary = d.try_coerce(DataFormat::Binary);
        assert_eq!(DataFormat::Binary, binary.format());
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_to_text() {
        let value = json!({"name": "John\\Doe\r\n"});
        let d = Data::json(value);
        let text = d.try_coerce(DataFormat::Text);
        assert_eq!(DataFormat::Text, text.format());
    }

    // Tests for coercible conversions create the same output as to_bytes/render
    //
    // render does not need to be checked against bin -> text since render
    // outputs None for binary
    #[test]
    fn text_to_bin_coerce_equals_to_bytes() {
        let text = String::from("test");
        let d = Data::text(text);
        let binary = d.clone().try_coerce(DataFormat::Binary);
        assert_eq!(Data::binary(d.to_bytes()), binary);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_to_bin_coerce_equals_to_bytes() {
        let json = json!({"name": "John\\Doe\r\n"});
        let d = Data::json(json);
        let binary = d.clone().try_coerce(DataFormat::Binary);
        assert_eq!(Data::binary(d.to_bytes()), binary);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_to_text_coerce_equals_render() {
        let json = json!({"name": "John\\Doe\r\n"});
        let d = Data::json(json);
        let text = d.clone().try_coerce(DataFormat::Text);
        assert_eq!(Data::text(d.render().unwrap()), text);
    }

    // Tests for normalization on json
    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_paths_and_lines() {
        let json = json!({"name": "John\\Doe\r\n"});
        let data = Data::json(json);
        let data = data.normalize(NormalizePaths);
        assert_eq!(Data::json(json!({"name": "John/Doe\r\n"})), data);
        let data = data.normalize(NormalizeNewlines);
        assert_eq!(Data::json(json!({"name": "John/Doe\n"})), data);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_obj_paths_and_lines() {
        let json = json!({
            "person": {
                "name": "John\\Doe\r\n",
                "nickname": "Jo\\hn\r\n",
            }
        });
        let data = Data::json(json);
        let data = data.normalize(NormalizePaths);
        let assert = json!({
            "person": {
                "name": "John/Doe\r\n",
                "nickname": "Jo/hn\r\n",
            }
        });
        assert_eq!(Data::json(assert), data);
        let data = data.normalize(NormalizeNewlines);
        let assert = json!({
            "person": {
                "name": "John/Doe\n",
                "nickname": "Jo/hn\n",
            }
        });
        assert_eq!(Data::json(assert), data);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_array_paths_and_lines() {
        let json = json!({"people": ["John\\Doe\r\n", "Jo\\hn\r\n"]});
        let data = Data::json(json);
        let data = data.normalize(NormalizePaths);
        let paths = json!({"people": ["John/Doe\r\n", "Jo/hn\r\n"]});
        assert_eq!(Data::json(paths), data);
        let data = data.normalize(NormalizeNewlines);
        let new_lines = json!({"people": ["John/Doe\n", "Jo/hn\n"]});
        assert_eq!(Data::json(new_lines), data);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_array_obj_paths_and_lines() {
        let json = json!({
            "people": [
                {
                    "name": "John\\Doe\r\n",
                    "nickname": "Jo\\hn\r\n",
                }
            ]
        });
        let data = Data::json(json);
        let data = data.normalize(NormalizePaths);
        let paths = json!({
            "people": [
                {
                    "name": "John/Doe\r\n",
                    "nickname": "Jo/hn\r\n",
                }
            ]
        });
        assert_eq!(Data::json(paths), data);
        let data = data.normalize(NormalizeNewlines);
        let new_lines = json!({
            "people": [
                {
                    "name": "John/Doe\n",
                    "nickname": "Jo/hn\n",
                }
            ]
        });
        assert_eq!(Data::json(new_lines), data);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_matches_string() {
        let exp = json!({"name": "{...}"});
        let expected = Data::json(exp);
        let actual = json!({"name": "JohnDoe"});
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_matches_array() {
        let exp = json!({"people": "{...}"});
        let expected = Data::json(exp);
        let actual = json!({
            "people": [
                {
                    "name": "JohnDoe",
                    "nickname": "John",
                }
            ]
        });
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_matches_obj() {
        let exp = json!({"people": "{...}"});
        let expected = Data::json(exp);
        let actual = json!({
            "people": {
                "name": "JohnDoe",
                "nickname": "John",
            }
        });
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_matches_diff_order_array() {
        let exp = json!({
            "people": ["John", "Jane"]
        });
        let expected = Data::json(exp);
        let actual = json!({
            "people": ["Jane", "John"]
        });
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_ne!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_wildcard_object_first() {
        let exp = json!({
            "people": [
                "{...}",
                {
                    "name": "three",
                    "nickname": "3",
                }
            ]
        });
        let expected = Data::json(exp);
        let actual = json!({
            "people": [
                {
                    "name": "one",
                    "nickname": "1",
                },
                {
                    "name": "two",
                    "nickname": "2",
                },
                {
                    "name": "three",
                    "nickname": "3",
                }
            ]
        });
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_wildcard_array_first() {
        let exp = json!([
            "{...}",
            {
                "name": "three",
                "nickname": "3",
            }
        ]);
        let expected = Data::json(exp);
        let actual = json!([
            {
                "name": "one",
                "nickname": "1",
            },
            {
                "name": "two",
                "nickname": "2",
            },
            {
                "name": "three",
                "nickname": "3",
            }
        ]);
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_wildcard_array_first_last() {
        let exp = json!([
            "{...}",
            {
                "name": "two",
                "nickname": "2",
            },
            "{...}"
        ]);
        let expected = Data::json(exp);
        let actual = json!([
            {
                "name": "one",
                "nickname": "1",
            },
            {
                "name": "two",
                "nickname": "2",
            },
            {
                "name": "three",
                "nickname": "3",
            },
            {
                "name": "four",
                "nickname": "4",
            }
        ]);
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_wildcard_array_middle_last() {
        let exp = json!([
            {
                "name": "one",
                "nickname": "1",
            },
            "{...}",
            {
                "name": "three",
                "nickname": "3",
            },
            "{...}"
        ]);
        let expected = Data::json(exp);
        let actual = json!([
            {
                "name": "one",
                "nickname": "1",
            },
            {
                "name": "two",
                "nickname": "2",
            },
            {
                "name": "three",
                "nickname": "3",
            },
            {
                "name": "four",
                "nickname": "4",
            },
            {
                "name": "five",
                "nickname": "5",
            }
        ]);
        let actual = Data::json(actual).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
            assert_eq!(exp, act);
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_normalize_wildcard_array_middle_last_early_return() {
        let exp = json!([
            {
                "name": "one",
                "nickname": "1",
            },
            "{...}",
            {
                "name": "three",
                "nickname": "3",
            },
            "{...}"
        ]);
        let expected = Data::json(exp);
        let actual = json!([
            {
                "name": "one",
                "nickname": "1",
            },
            {
                "name": "two",
                "nickname": "2",
            },
            {
                "name": "four",
                "nickname": "4",
            },
            {
                "name": "five",
                "nickname": "5",
            }
        ]);
        let actual_normalized = Data::json(actual.clone()).normalize(NormalizeMatches {
            substitutions: &Default::default(),
            pattern: &expected,
        });
        if let DataInner::Json(act) = actual_normalized.inner {
            assert_eq!(act, actual);
        }
    }
}
