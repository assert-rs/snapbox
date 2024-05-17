//! Filter `actual` or `expected` [`Data`]
//!
//! This can be done for
//! - Making snapshots consistent across platforms or conditional compilation
//! - Focusing snapshots on the characteristics of the data being tested

mod redactions;
#[cfg(test)]
mod test;

use crate::data::DataInner;
use crate::Data;

pub use redactions::RedactedValue;
pub use redactions::Redactions;

pub trait Filter {
    #[deprecated(since = "0.5.11", note = "Replaced with `Filter::filter`")]
    fn normalize(&self, data: Data) -> Data;
    fn filter(&self, data: Data) -> Data {
        #[allow(deprecated)]
        self.normalize(data)
    }
}

pub struct FilterNewlines;
impl Filter for FilterNewlines {
    fn normalize(&self, data: Data) -> Data {
        let source = data.source;
        let filters = data.filters;
        let inner = match data.inner {
            DataInner::Error(err) => DataInner::Error(err),
            DataInner::Binary(bin) => DataInner::Binary(bin),
            DataInner::Text(text) => {
                let lines = normalize_lines(&text);
                DataInner::Text(lines)
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                normalize_value(&mut value, normalize_lines);
                DataInner::Json(value)
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                let lines = normalize_lines(&text);
                DataInner::TermSvg(lines)
            }
        };
        Data {
            inner,
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
    fn normalize(&self, data: Data) -> Data {
        let source = data.source;
        let filters = data.filters;
        let inner = match data.inner {
            DataInner::Error(err) => DataInner::Error(err),
            DataInner::Binary(bin) => DataInner::Binary(bin),
            DataInner::Text(text) => {
                let lines = normalize_paths(&text);
                DataInner::Text(lines)
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                normalize_value(&mut value, normalize_paths);
                DataInner::Json(value)
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                let lines = normalize_paths(&text);
                DataInner::TermSvg(lines)
            }
        };
        Data {
            inner,
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

pub struct FilterRedactions<'a> {
    substitutions: &'a crate::Redactions,
    pattern: &'a Data,
}

impl<'a> FilterRedactions<'a> {
    pub fn new(substitutions: &'a crate::Redactions, pattern: &'a Data) -> Self {
        FilterRedactions {
            substitutions,
            pattern,
        }
    }
}

impl Filter for FilterRedactions<'_> {
    fn normalize(&self, data: Data) -> Data {
        let source = data.source;
        let filters = data.filters;
        let inner = match data.inner {
            DataInner::Error(err) => DataInner::Error(err),
            DataInner::Binary(bin) => DataInner::Binary(bin),
            DataInner::Text(text) => {
                if let Some(pattern) = self.pattern.render() {
                    let lines = self.substitutions.normalize(&text, &pattern);
                    DataInner::Text(lines)
                } else {
                    DataInner::Text(text)
                }
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                if let DataInner::Json(exp) = &self.pattern.inner {
                    normalize_value_matches(&mut value, exp, self.substitutions);
                }
                DataInner::Json(value)
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                if let Some(pattern) = self.pattern.render() {
                    let lines = self.substitutions.normalize(&text, &pattern);
                    DataInner::TermSvg(lines)
                } else {
                    DataInner::TermSvg(text)
                }
            }
        };
        Data {
            inner,
            source,
            filters,
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
    substitutions: &crate::Redactions,
) {
    use serde_json::Value::*;

    const VALUE_WILDCARD: &str = "{...}";

    match (actual, expected) {
        (act, String(exp)) if exp == VALUE_WILDCARD => {
            *act = serde_json::json!(VALUE_WILDCARD);
        }
        (String(act), String(exp)) => {
            *act = substitutions.normalize(act, exp);
        }
        (Array(act), Array(exp)) => {
            let mut sections = exp.split(|e| e == VALUE_WILDCARD).peekable();
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
                        act.splice(processed.., vec![String(VALUE_WILDCARD.to_owned())]);
                        processed += 1;
                    } else {
                        let first = next_section.first().unwrap();
                        // Replace everything up until the value we are looking for with
                        // a single "{...}".
                        if let Some(index) = act.iter().position(|v| v == first) {
                            act.splice(processed..index, vec![String(VALUE_WILDCARD.to_owned())]);
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
