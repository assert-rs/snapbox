//! Filter `actual` or `expected` [`Data`]
//!
//! This can be done for
//! - Making snapshots consistent across platforms or conditional compilation
//! - Focusing snapshots on the characteristics of the data being tested

mod pattern;
mod redactions;
#[cfg(test)]
mod test;
#[cfg(test)]
mod test_redactions;
#[cfg(test)]
mod test_unordered_redactions;

use crate::Data;
use crate::data::DataValue;

pub use pattern::NormalizeToExpected;
pub use redactions::RedactedValue;
pub use redactions::Redactions;

pub trait Filter {
    fn filter(&self, data: Data) -> Data;
}

pub struct FilterNewlines;
impl Filter for FilterNewlines {
    fn filter(&self, data: Data) -> Data {
        let source = data.source;
        let filters = data.filters;
        let inner = match data.value {
            DataValue::Error(err) => DataValue::Error(err),
            DataValue::Binary(bin) => DataValue::Binary(bin),
            DataValue::Text(text) => {
                let lines = normalize_lines(&text);
                DataValue::Text(lines)
            }
            #[cfg(feature = "json")]
            DataValue::Json(value) => {
                let mut value = value;
                normalize_json_string(&mut value, &normalize_lines);
                DataValue::Json(value)
            }
            #[cfg(feature = "json")]
            DataValue::JsonLines(value) => {
                let mut value = value;
                normalize_json_string(&mut value, &normalize_lines);
                DataValue::JsonLines(value)
            }
            #[cfg(feature = "term-svg")]
            DataValue::TermSvg(text) => {
                let lines = normalize_lines(&text);
                DataValue::TermSvg(lines)
            }
        };
        Data {
            value: inner,
            source,
            filters,
        }
    }
}

/// Normalize line endings
pub fn normalize_lines(data: &str) -> String {
    normalize_lines_chars(data.chars()).collect()
}

fn normalize_lines_chars(data: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    normalize_line_endings::normalized(data)
}

pub struct FilterPaths;
impl Filter for FilterPaths {
    fn filter(&self, data: Data) -> Data {
        let source = data.source;
        let filters = data.filters;
        let inner = match data.value {
            DataValue::Error(err) => DataValue::Error(err),
            DataValue::Binary(bin) => DataValue::Binary(bin),
            DataValue::Text(text) => {
                let lines = normalize_paths(&text);
                DataValue::Text(lines)
            }
            #[cfg(feature = "json")]
            DataValue::Json(value) => {
                let mut value = value;
                normalize_json_string(&mut value, &normalize_paths);
                DataValue::Json(value)
            }
            #[cfg(feature = "json")]
            DataValue::JsonLines(value) => {
                let mut value = value;
                normalize_json_string(&mut value, &normalize_paths);
                DataValue::JsonLines(value)
            }
            #[cfg(feature = "term-svg")]
            DataValue::TermSvg(text) => {
                let lines = normalize_paths(&text);
                DataValue::TermSvg(lines)
            }
        };
        Data {
            value: inner,
            source,
            filters,
        }
    }
}

/// Normalize path separators
///
/// [`std::path::MAIN_SEPARATOR`] can vary by platform, so make it consistent
///
/// Note: this cannot distinguish between when a character is being used as a path separator or not
/// and can "normalize" unrelated data
pub fn normalize_paths(data: &str) -> String {
    normalize_paths_chars(data.chars()).collect()
}

fn normalize_paths_chars(data: impl Iterator<Item = char>) -> impl Iterator<Item = char> {
    data.map(|c| if c == '\\' { '/' } else { c })
}

struct NormalizeRedactions<'r> {
    redactions: &'r Redactions,
}
impl Filter for NormalizeRedactions<'_> {
    fn filter(&self, data: Data) -> Data {
        let source = data.source;
        let filters = data.filters;
        let inner = match data.value {
            DataValue::Error(err) => DataValue::Error(err),
            DataValue::Binary(bin) => DataValue::Binary(bin),
            DataValue::Text(text) => {
                let lines = self.redactions.redact(&text);
                DataValue::Text(lines)
            }
            #[cfg(feature = "json")]
            DataValue::Json(value) => {
                let mut value = value;
                normalize_json_string(&mut value, &|s| self.redactions.redact(s));
                DataValue::Json(value)
            }
            #[cfg(feature = "json")]
            DataValue::JsonLines(value) => {
                let mut value = value;
                normalize_json_string(&mut value, &|s| self.redactions.redact(s));
                DataValue::JsonLines(value)
            }
            #[cfg(feature = "term-svg")]
            DataValue::TermSvg(text) => {
                let lines = self.redactions.redact(&text);
                DataValue::TermSvg(lines)
            }
        };
        Data {
            value: inner,
            source,
            filters,
        }
    }
}

#[cfg(feature = "structured-data")]
fn normalize_json_string(value: &mut serde_json::Value, op: &dyn Fn(&str) -> String) {
    match value {
        serde_json::Value::String(str) => {
            *str = op(str);
        }
        serde_json::Value::Array(arr) => {
            for value in arr.iter_mut() {
                normalize_json_string(value, op);
            }
        }
        serde_json::Value::Object(obj) => {
            for (key, mut value) in std::mem::replace(obj, serde_json::Map::new()) {
                let key = op(&key);
                normalize_json_string(&mut value, op);
                obj.insert(key, value);
            }
        }
        _ => {}
    }
}
