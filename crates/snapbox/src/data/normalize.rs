use super::Data;
use super::DataInner;

pub trait Normalize {
    fn normalize(&self, data: Data) -> Data;
}

pub struct NormalizeNewlines;
impl Normalize for NormalizeNewlines {
    fn normalize(&self, data: Data) -> Data {
        let mut new = match data.inner {
            DataInner::Error(err) => err.into(),
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
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                let lines = crate::utils::normalize_lines(&text);
                DataInner::TermSvg(lines).into()
            }
        };
        new.source = data.source;
        new
    }
}

pub struct NormalizePaths;
impl Normalize for NormalizePaths {
    fn normalize(&self, data: Data) -> Data {
        let mut new = match data.inner {
            DataInner::Error(err) => err.into(),
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
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                let lines = crate::utils::normalize_paths(&text);
                DataInner::TermSvg(lines).into()
            }
        };
        new.source = data.source;
        new
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
        let mut new = match data.inner {
            DataInner::Error(err) => err.into(),
            DataInner::Binary(bin) => Data::binary(bin),
            DataInner::Text(text) => {
                if let Some(pattern) = self.pattern.render() {
                    let lines = self.substitutions.normalize(&text, &pattern);
                    Data::text(lines)
                } else {
                    DataInner::Text(text).into()
                }
            }
            #[cfg(feature = "json")]
            DataInner::Json(value) => {
                let mut value = value;
                if let DataInner::Json(exp) = &self.pattern.inner {
                    normalize_value_matches(&mut value, exp, self.substitutions);
                }
                Data::json(value)
            }
            #[cfg(feature = "term-svg")]
            DataInner::TermSvg(text) => {
                if let Some(pattern) = self.pattern.render() {
                    let lines = self.substitutions.normalize(&text, &pattern);
                    DataInner::TermSvg(lines).into()
                } else {
                    DataInner::TermSvg(text).into()
                }
            }
        };
        new.source = data.source;
        new
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
