//! Filter `actual` or `expected` [`Data`]
//!
//! This can be done for
//! - Making snapshots consistent across platforms or conditional compilation
//! - Focusing snapshots on the characteristics of the data being tested

#[cfg(test)]
mod test;

use crate::data::DataInner;
use crate::Data;

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
        let inner = match data.inner {
            DataInner::Error(err) => DataInner::Error(err),
            DataInner::Binary(bin) => DataInner::Binary(bin),
            DataInner::Text(text) => {
                let lines = crate::utils::normalize_lines(&text);
                DataInner::Text(lines)
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                normalize_value(&mut value, crate::utils::normalize_lines);
                DataInner::Json(value)
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                let lines = crate::utils::normalize_lines(&text);
                DataInner::TermSvg(lines)
            }
        };
        Data { inner, source }
    }
}

pub struct FilterPaths;
impl Filter for FilterPaths {
    fn normalize(&self, data: Data) -> Data {
        let source = data.source;
        let inner = match data.inner {
            DataInner::Error(err) => DataInner::Error(err),
            DataInner::Binary(bin) => DataInner::Binary(bin),
            DataInner::Text(text) => {
                let lines = crate::utils::normalize_paths(&text);
                DataInner::Text(lines)
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                normalize_value(&mut value, crate::utils::normalize_paths);
                DataInner::Json(value)
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                let lines = crate::utils::normalize_paths(&text);
                DataInner::TermSvg(lines)
            }
        };
        Data { inner, source }
    }
}

pub struct FilterMatches<'a> {
    substitutions: &'a crate::Substitutions,
    pattern: &'a Data,
}

impl<'a> FilterMatches<'a> {
    pub fn new(substitutions: &'a crate::Substitutions, pattern: &'a Data) -> Self {
        FilterMatches {
            substitutions,
            pattern,
        }
    }
}

impl Filter for FilterMatches<'_> {
    fn normalize(&self, data: Data) -> Data {
        let source = data.source;
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
        Data { inner, source }
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