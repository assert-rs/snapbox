use super::{Filter, NormalizeRedactions, Redactions};
use crate::data::DataInner;
use crate::Data;

/// Adjust `actual` based on `expected`
pub struct NormalizeToExpected<'a> {
    substitutions: Option<&'a Redactions>,
    unordered: bool,
}

impl<'a> NormalizeToExpected<'a> {
    pub fn new() -> Self {
        Self {
            substitutions: None,
            unordered: false,
        }
    }

    /// Make unordered content comparable
    ///
    /// This is done by re-ordering `actual` according to `expected`.
    pub fn unordered(mut self) -> Self {
        self.unordered = true;
        self
    }

    /// Apply built-in redactions.
    ///
    /// Built-in redactions:
    /// - `...` on a line of its own: match multiple complete lines
    /// - `[..]`: match multiple characters within a line
    ///
    /// Built-ins cannot automatically be applied to `actual` but are inferred from `expected`
    pub fn redact(mut self) -> Self {
        static REDACTIONS: Redactions = Redactions::new();
        self.substitutions = Some(&REDACTIONS);
        self
    }

    /// Apply built-in and user [`Redactions`]
    ///
    /// Built-in redactions:
    /// - `...` on a line of its own: match multiple complete lines
    /// - `[..]`: match multiple characters within a line
    ///
    /// Built-ins cannot automatically be applied to `actual` but are inferred from `expected`
    pub fn redact_with(mut self, redactions: &'a Redactions) -> Self {
        self.substitutions = Some(redactions);
        self
    }

    pub fn normalize(&self, actual: Data, expected: &Data) -> Data {
        let actual = if let Some(substitutions) = self.substitutions {
            NormalizeRedactions {
                redactions: substitutions,
            }
            .filter(actual)
        } else {
            actual
        };
        match (self.substitutions, self.unordered) {
            (None, false) => actual,
            (Some(substitutions), false) => {
                normalize_data_to_redactions(actual, expected, substitutions)
            }
            (None, true) => normalize_data_to_unordered(actual, expected),
            (Some(substitutions), true) => {
                normalize_data_to_unordered_redactions(actual, expected, substitutions)
            }
        }
    }
}

impl Default for NormalizeToExpected<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_data_to_unordered(actual: Data, expected: &Data) -> Data {
    let source = actual.source;
    let filters = actual.filters;
    let inner = match (actual.inner, &expected.inner) {
        (DataInner::Error(err), _) => DataInner::Error(err),
        (DataInner::Binary(bin), _) => DataInner::Binary(bin),
        (DataInner::Text(text), _) => {
            if let Some(pattern) = expected.render() {
                let lines = normalize_str_to_unordered(&text, &pattern);
                DataInner::Text(lines)
            } else {
                DataInner::Text(text)
            }
        }
        #[cfg(feature = "json")]
        (DataInner::Json(value), DataInner::Json(exp)) => {
            let mut value = value;
            normalize_value_to_unordered(&mut value, exp);
            DataInner::Json(value)
        }
        #[cfg(feature = "json")]
        (DataInner::JsonLines(value), DataInner::JsonLines(exp)) => {
            let mut value = value;
            normalize_value_to_unordered(&mut value, exp);
            DataInner::JsonLines(value)
        }
        #[cfg(feature = "term-svg")]
        (DataInner::TermSvg(text), DataInner::TermSvg(exp)) => {
            if let (Some((header, body, footer)), Some((_, exp, _))) = (
                crate::data::split_term_svg(&text),
                crate::data::split_term_svg(exp),
            ) {
                let lines = normalize_str_to_unordered(body, exp);
                DataInner::TermSvg(format!("{header}{lines}{footer}"))
            } else {
                DataInner::TermSvg(text)
            }
        }
        // reachable if more than one structured data format is enabled
        #[allow(unreachable_patterns)]
        (inner, _) => inner,
    };
    Data {
        inner,
        source,
        filters,
    }
}

#[cfg(feature = "structured-data")]
fn normalize_value_to_unordered(actual: &mut serde_json::Value, expected: &serde_json::Value) {
    use serde_json::Value::{Array, Object, String};

    match (actual, expected) {
        (String(act), String(exp)) => {
            *act = normalize_str_to_unordered(act, exp);
        }
        (Array(act), Array(exp)) => {
            let mut actual_values = std::mem::take(act);
            let mut expected_values = exp.clone();
            expected_values.retain(|expected_value| {
                let mut matched = false;
                actual_values.retain(|actual_value| {
                    if !matched && actual_value == expected_value {
                        matched = true;
                        false
                    } else {
                        true
                    }
                });
                if matched {
                    act.push(expected_value.clone());
                }
                !matched
            });
            for actual_value in actual_values {
                act.push(actual_value);
            }
        }
        (Object(act), Object(exp)) => {
            for (actual_key, mut actual_value) in std::mem::replace(act, serde_json::Map::new()) {
                if let Some(expected_value) = exp.get(&actual_key) {
                    normalize_value_to_unordered(&mut actual_value, expected_value);
                }
                act.insert(actual_key, actual_value);
            }
        }
        (_, _) => {}
    }
}

fn normalize_str_to_unordered(actual: &str, expected: &str) -> String {
    if actual == expected {
        return actual.to_owned();
    }

    let mut normalized: Vec<&str> = Vec::new();
    let mut actual_lines: Vec<_> = crate::utils::LinesWithTerminator::new(actual).collect();
    let mut expected_lines: Vec<_> = crate::utils::LinesWithTerminator::new(expected).collect();
    expected_lines.retain(|expected_line| {
        let mut matched = false;
        actual_lines.retain(|actual_line| {
            if !matched && actual_line == expected_line {
                matched = true;
                false
            } else {
                true
            }
        });
        if matched {
            normalized.push(expected_line);
        }
        !matched
    });
    for actual_line in &actual_lines {
        normalized.push(actual_line);
    }

    normalized.join("")
}

#[cfg(feature = "structured-data")]
const KEY_WILDCARD: &str = "...";
#[cfg(feature = "structured-data")]
const VALUE_WILDCARD: &str = "{...}";

fn normalize_data_to_unordered_redactions(
    actual: Data,
    expected: &Data,
    substitutions: &Redactions,
) -> Data {
    let source = actual.source;
    let filters = actual.filters;
    let inner = match (actual.inner, &expected.inner) {
        (DataInner::Error(err), _) => DataInner::Error(err),
        (DataInner::Binary(bin), _) => DataInner::Binary(bin),
        (DataInner::Text(text), _) => {
            if let Some(pattern) = expected.render() {
                let lines = normalize_str_to_unordered_redactions(&text, &pattern, substitutions);
                DataInner::Text(lines)
            } else {
                DataInner::Text(text)
            }
        }
        #[cfg(feature = "json")]
        (DataInner::Json(value), DataInner::Json(exp)) => {
            let mut value = value;
            normalize_value_to_unordered_redactions(&mut value, exp, substitutions);
            DataInner::Json(value)
        }
        #[cfg(feature = "json")]
        (DataInner::JsonLines(value), DataInner::JsonLines(exp)) => {
            let mut value = value;
            normalize_value_to_unordered_redactions(&mut value, exp, substitutions);
            DataInner::JsonLines(value)
        }
        #[cfg(feature = "term-svg")]
        (DataInner::TermSvg(text), DataInner::TermSvg(exp)) => {
            if let (Some((header, body, footer)), Some((_, exp, _))) = (
                crate::data::split_term_svg(&text),
                crate::data::split_term_svg(exp),
            ) {
                let lines = normalize_str_to_unordered_redactions(body, exp, substitutions);
                DataInner::TermSvg(format!("{header}{lines}{footer}"))
            } else {
                DataInner::TermSvg(text)
            }
        }
        // reachable if more than one structured data format is enabled
        #[allow(unreachable_patterns)]
        (inner, _) => inner,
    };
    Data {
        inner,
        source,
        filters,
    }
}

#[cfg(feature = "structured-data")]
fn normalize_value_to_unordered_redactions(
    actual: &mut serde_json::Value,
    expected: &serde_json::Value,
    substitutions: &Redactions,
) {
    use serde_json::Value::{Array, Object, String};

    match (actual, expected) {
        (act, String(exp)) if exp == VALUE_WILDCARD => {
            *act = serde_json::json!(VALUE_WILDCARD);
        }
        (String(act), String(exp)) => {
            *act = normalize_str_to_unordered_redactions(act, exp, substitutions);
        }
        (Array(act), Array(exp)) => {
            *act = normalize_array_to_unordered_redactions(act, exp, substitutions);
        }
        (Object(act), Object(exp)) => {
            let has_key_wildcard =
                exp.get(KEY_WILDCARD).and_then(|v| v.as_str()) == Some(VALUE_WILDCARD);
            for (actual_key, mut actual_value) in std::mem::replace(act, serde_json::Map::new()) {
                if let Some(expected_value) = exp.get(&actual_key) {
                    normalize_value_to_unordered_redactions(
                        &mut actual_value,
                        expected_value,
                        substitutions,
                    );
                } else if has_key_wildcard {
                    continue;
                }
                act.insert(actual_key, actual_value);
            }
            if has_key_wildcard {
                act.insert(KEY_WILDCARD.to_owned(), String(VALUE_WILDCARD.to_owned()));
            }
        }
        (_, _) => {}
    }
}

#[cfg(feature = "structured-data")]
fn normalize_array_to_unordered_redactions(
    actual: &[serde_json::Value],
    expected: &[serde_json::Value],
    substitutions: &Redactions,
) -> Vec<serde_json::Value> {
    if actual == expected {
        return actual.to_owned();
    }

    let mut normalized: Vec<serde_json::Value> = Vec::new();
    let mut actual_values = actual.to_owned();
    let mut expected_values = expected.to_owned();
    let mut elided = false;
    expected_values.retain(|expected_value| {
        let mut matched = false;
        if expected_value == VALUE_WILDCARD {
            matched = true;
            elided = true;
        } else {
            actual_values.retain(|actual_value| {
                let mut normalized_actual_value = actual_value.clone();
                normalize_value_to_unordered_redactions(
                    &mut normalized_actual_value,
                    expected_value,
                    substitutions,
                );
                if !matched && normalized_actual_value == *expected_value {
                    matched = true;
                    false
                } else {
                    true
                }
            });
        }
        if matched {
            normalized.push(expected_value.clone());
        }
        !matched
    });
    if !elided {
        for actual_value in actual_values {
            normalized.push(actual_value);
        }
    }

    normalized
}

fn normalize_str_to_unordered_redactions(
    actual: &str,
    expected: &str,
    substitutions: &Redactions,
) -> String {
    if actual == expected {
        return actual.to_owned();
    }

    let mut normalized: Vec<&str> = Vec::new();
    let mut actual_lines: Vec<_> = crate::utils::LinesWithTerminator::new(actual).collect();
    let mut expected_lines: Vec<_> = crate::utils::LinesWithTerminator::new(expected).collect();
    let mut elided = false;
    expected_lines.retain(|expected_line| {
        let mut matched = false;
        if is_line_elide(expected_line) {
            matched = true;
            elided = true;
        } else {
            actual_lines.retain(|actual_line| {
                if !matched && line_matches(actual_line, expected_line, substitutions) {
                    matched = true;
                    false
                } else {
                    true
                }
            });
        }
        if matched {
            normalized.push(expected_line);
        }
        !matched
    });
    if !elided {
        for actual_line in &actual_lines {
            normalized.push(actual_line);
        }
    }

    normalized.join("")
}

fn normalize_data_to_redactions(actual: Data, expected: &Data, substitutions: &Redactions) -> Data {
    let source = actual.source;
    let filters = actual.filters;
    let inner = match (actual.inner, &expected.inner) {
        (DataInner::Error(err), _) => DataInner::Error(err),
        (DataInner::Binary(bin), _) => DataInner::Binary(bin),
        (DataInner::Text(text), _) => {
            if let Some(pattern) = expected.render() {
                let lines = normalize_str_to_redactions(&text, &pattern, substitutions);
                DataInner::Text(lines)
            } else {
                DataInner::Text(text)
            }
        }
        #[cfg(feature = "json")]
        (DataInner::Json(value), DataInner::Json(exp)) => {
            let mut value = value;
            normalize_value_to_redactions(&mut value, exp, substitutions);
            DataInner::Json(value)
        }
        #[cfg(feature = "json")]
        (DataInner::JsonLines(value), DataInner::JsonLines(exp)) => {
            let mut value = value;
            normalize_value_to_redactions(&mut value, exp, substitutions);
            DataInner::JsonLines(value)
        }
        #[cfg(feature = "term-svg")]
        (DataInner::TermSvg(text), DataInner::TermSvg(exp)) => {
            if let (Some((header, body, footer)), Some((_, exp, _))) = (
                crate::data::split_term_svg(&text),
                crate::data::split_term_svg(exp),
            ) {
                let lines = normalize_str_to_redactions(body, exp, substitutions);
                DataInner::TermSvg(format!("{header}{lines}{footer}"))
            } else {
                DataInner::TermSvg(text)
            }
        }
        // reachable if more than one structured data format is enabled
        #[allow(unreachable_patterns)]
        (inner, _) => inner,
    };
    Data {
        inner,
        source,
        filters,
    }
}

#[cfg(feature = "structured-data")]
fn normalize_value_to_redactions(
    actual: &mut serde_json::Value,
    expected: &serde_json::Value,
    substitutions: &Redactions,
) {
    use serde_json::Value::{Array, Object, String};

    match (actual, expected) {
        (act, String(exp)) if exp == VALUE_WILDCARD => {
            *act = serde_json::json!(VALUE_WILDCARD);
        }
        (String(act), String(exp)) => {
            *act = normalize_str_to_redactions(act, exp, substitutions);
        }
        (Array(act), Array(exp)) => {
            *act = normalize_array_to_redactions(act, exp, substitutions);
        }
        (Object(act), Object(exp)) => {
            let has_key_wildcard =
                exp.get(KEY_WILDCARD).and_then(|v| v.as_str()) == Some(VALUE_WILDCARD);
            for (actual_key, mut actual_value) in std::mem::replace(act, serde_json::Map::new()) {
                if let Some(expected_value) = exp.get(&actual_key) {
                    normalize_value_to_redactions(&mut actual_value, expected_value, substitutions);
                } else if has_key_wildcard {
                    continue;
                }
                act.insert(actual_key, actual_value);
            }
            if has_key_wildcard {
                act.insert(KEY_WILDCARD.to_owned(), String(VALUE_WILDCARD.to_owned()));
            }
        }
        (_, _) => {}
    }
}

#[cfg(feature = "structured-data")]
fn normalize_array_to_redactions(
    actual: &[serde_json::Value],
    expected: &[serde_json::Value],
    redactions: &Redactions,
) -> Vec<serde_json::Value> {
    if actual == expected {
        return actual.to_vec();
    }

    let mut normalized: Vec<serde_json::Value> = Vec::new();
    let mut actual_index = 0;
    let mut expected = expected.iter().peekable();
    while let Some(expected_elem) = expected.next() {
        if expected_elem == VALUE_WILDCARD {
            let Some(next_expected_elem) = expected.peek() else {
                // Stop as elide consumes to end
                normalized.push(expected_elem.clone());
                actual_index = actual.len();
                break;
            };
            let Some(index_offset) = actual[actual_index..].iter().position(|next_actual_elem| {
                let mut next_actual_elem = next_actual_elem.clone();
                normalize_value_to_redactions(
                    &mut next_actual_elem,
                    next_expected_elem,
                    redactions,
                );
                next_actual_elem == **next_expected_elem
            }) else {
                // Give up as we can't find where the elide ends
                break;
            };
            normalized.push(expected_elem.clone());
            actual_index += index_offset;
        } else {
            let Some(actual_elem) = actual.get(actual_index) else {
                // Give up as we have no more content to check
                break;
            };

            actual_index += 1;
            let mut normalized_elem = actual_elem.clone();
            normalize_value_to_redactions(&mut normalized_elem, expected_elem, redactions);
            normalized.push(normalized_elem);
        }
    }

    normalized.extend(actual[actual_index..].iter().cloned());
    normalized
}

fn normalize_str_to_redactions(actual: &str, expected: &str, redactions: &Redactions) -> String {
    if actual == expected {
        return actual.to_owned();
    }

    let mut normalized: Vec<&str> = Vec::new();
    let mut actual_index = 0;
    let actual_lines: Vec<_> = crate::utils::LinesWithTerminator::new(actual).collect();
    let mut expected_lines = crate::utils::LinesWithTerminator::new(expected).peekable();
    while let Some(expected_line) = expected_lines.next() {
        if is_line_elide(expected_line) {
            let Some(next_expected_line) = expected_lines.peek() else {
                // Stop as elide consumes to end
                normalized.push(expected_line);
                actual_index = actual_lines.len();
                break;
            };
            let Some(index_offset) =
                actual_lines[actual_index..]
                    .iter()
                    .position(|next_actual_line| {
                        line_matches(next_actual_line, next_expected_line, redactions)
                    })
            else {
                // Give up as we can't find where the elide ends
                break;
            };
            normalized.push(expected_line);
            actual_index += index_offset;
        } else {
            let Some(actual_line) = actual_lines.get(actual_index) else {
                // Give up as we have no more content to check
                break;
            };

            if line_matches(actual_line, expected_line, redactions) {
                actual_index += 1;
                normalized.push(expected_line);
            } else {
                // Skip this line and keep processing
                actual_index += 1;
                normalized.push(actual_line);
            }
        }
    }

    normalized.extend(actual_lines[actual_index..].iter().copied());
    normalized.join("")
}

fn is_line_elide(line: &str) -> bool {
    line == "...\n" || line == "..."
}

fn line_matches(mut actual: &str, expected: &str, redactions: &Redactions) -> bool {
    if actual == expected {
        return true;
    }

    let expected = redactions.clear_unused(expected);
    let mut sections = expected.split("[..]").peekable();
    while let Some(section) = sections.next() {
        if let Some(remainder) = actual.strip_prefix(section) {
            if let Some(next_section) = sections.peek() {
                if next_section.is_empty() {
                    actual = "";
                } else if let Some(restart_index) = remainder.find(next_section) {
                    actual = &remainder[restart_index..];
                }
            } else {
                return remainder.is_empty();
            }
        } else {
            return false;
        }
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn str_normalize_redactions_line_matches_cases() {
        let cases = [
            ("", "", true),
            ("", "[..]", true),
            ("hello", "hello", true),
            ("hello", "goodbye", false),
            ("hello", "[..]", true),
            ("hello", "he[..]", true),
            ("hello", "go[..]", false),
            ("hello", "[..]o", true),
            ("hello", "[..]e", false),
            ("hello", "he[..]o", true),
            ("hello", "he[..]e", false),
            ("hello", "go[..]o", false),
            ("hello", "go[..]e", false),
            (
                "hello world, goodbye moon",
                "hello [..], goodbye [..]",
                true,
            ),
            (
                "hello world, goodbye moon",
                "goodbye [..], goodbye [..]",
                false,
            ),
            (
                "hello world, goodbye moon",
                "goodbye [..], hello [..]",
                false,
            ),
            ("hello world, goodbye moon", "hello [..], [..] moon", true),
            (
                "hello world, goodbye moon",
                "goodbye [..], [..] moon",
                false,
            ),
            ("hello world, goodbye moon", "hello [..], [..] world", false),
        ];
        for (line, pattern, expected) in cases {
            let actual = line_matches(line, pattern, &Redactions::new());
            assert_eq!(expected, actual, "line={line:?}  pattern={pattern:?}");
        }
    }
}
