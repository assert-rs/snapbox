use super::Redactions;
use crate::data::DataInner;
use crate::Data;

/// Adjust `actual` based on `expected`
pub struct NormalizeToExpected<'a> {
    substitutions: Option<&'a crate::Redactions>,
}

impl<'a> NormalizeToExpected<'a> {
    pub fn new() -> Self {
        Self {
            substitutions: None,
        }
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
    pub fn redact_with(mut self, redactions: &'a crate::Redactions) -> Self {
        self.substitutions = Some(redactions);
        self
    }

    pub fn normalize(&self, actual: Data, expected: &Data) -> Data {
        let Some(substitutions) = self.substitutions else {
            return actual;
        };
        normalize_data_to_redactions(actual, expected, substitutions)
    }
}

impl Default for NormalizeToExpected<'_> {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_data_to_redactions(
    actual: Data,
    expected: &Data,
    substitutions: &crate::Redactions,
) -> Data {
    let source = actual.source;
    let filters = actual.filters;
    let inner = match actual.inner {
        DataInner::Error(err) => DataInner::Error(err),
        DataInner::Binary(bin) => DataInner::Binary(bin),
        DataInner::Text(text) => {
            if let Some(pattern) = expected.render() {
                let lines = normalize_str_to_redactions(&text, &pattern, substitutions);
                DataInner::Text(lines)
            } else {
                DataInner::Text(text)
            }
        }
        #[cfg(feature = "json")]
        DataInner::Json(value) => {
            let mut value = value;
            if let DataInner::Json(exp) = &expected.inner {
                normalize_value_to_redactions(&mut value, exp, substitutions);
            }
            DataInner::Json(value)
        }
        #[cfg(feature = "json")]
        DataInner::JsonLines(value) => {
            let mut value = value;
            if let DataInner::Json(exp) = &expected.inner {
                normalize_value_to_redactions(&mut value, exp, substitutions);
            }
            DataInner::JsonLines(value)
        }
        #[cfg(feature = "term-svg")]
        DataInner::TermSvg(text) => {
            if let Some(pattern) = expected.render() {
                let lines = normalize_str_to_redactions(&text, &pattern, substitutions);
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

#[cfg(feature = "structured-data")]
fn normalize_value_to_redactions(
    actual: &mut serde_json::Value,
    expected: &serde_json::Value,
    substitutions: &crate::Redactions,
) {
    use serde_json::Value::*;

    const KEY_WILDCARD: &str = "...";
    const VALUE_WILDCARD: &str = "{...}";

    match (actual, expected) {
        (act, String(exp)) if exp == VALUE_WILDCARD => {
            *act = serde_json::json!(VALUE_WILDCARD);
        }
        (String(act), String(exp)) => {
            *act = normalize_str_to_redactions(act, exp, substitutions);
        }
        (Array(act), Array(exp)) => {
            let mut sections = exp.split(|e| e == VALUE_WILDCARD).peekable();
            let mut processed = 0;
            while let Some(expected_subset) = sections.next() {
                // Process all values in the current section
                if !expected_subset.is_empty() {
                    let actual_subset = &mut act[processed..processed + expected_subset.len()];
                    for (a, e) in actual_subset.iter_mut().zip(expected_subset) {
                        normalize_value_to_redactions(a, e, substitutions);
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
            let has_key_wildcard =
                exp.get(KEY_WILDCARD).and_then(|v| v.as_str()) == Some(VALUE_WILDCARD);
            for (actual_key, mut actual_value) in std::mem::replace(act, serde_json::Map::new()) {
                let actual_key = substitutions.redact(&actual_key);
                if let Some(expected_value) = exp.get(&actual_key) {
                    normalize_value_to_redactions(&mut actual_value, expected_value, substitutions)
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

fn normalize_str_to_redactions(input: &str, pattern: &str, redactions: &Redactions) -> String {
    if input == pattern {
        return input.to_owned();
    }

    let input = redactions.redact(input);

    let mut normalized: Vec<&str> = Vec::new();
    let mut input_index = 0;
    let input_lines: Vec<_> = crate::utils::LinesWithTerminator::new(&input).collect();
    let mut pattern_lines = crate::utils::LinesWithTerminator::new(pattern).peekable();
    'outer: while let Some(pattern_line) = pattern_lines.next() {
        if is_line_elide(pattern_line) {
            if let Some(next_pattern_line) = pattern_lines.peek() {
                for (index_offset, next_input_line) in
                    input_lines[input_index..].iter().copied().enumerate()
                {
                    if line_matches(next_input_line, next_pattern_line, redactions) {
                        normalized.push(pattern_line);
                        input_index += index_offset;
                        continue 'outer;
                    }
                }
                // Give up doing further normalization
                break;
            } else {
                // Give up doing further normalization
                normalized.push(pattern_line);
                // captured rest so don't copy remaining lines over
                input_index = input_lines.len();
                break;
            }
        } else {
            let Some(input_line) = input_lines.get(input_index) else {
                // Give up doing further normalization
                break;
            };

            if line_matches(input_line, pattern_line, redactions) {
                input_index += 1;
                normalized.push(pattern_line);
            } else {
                // Give up doing further normalization
                break;
            }
        }
    }

    normalized.extend(input_lines[input_index..].iter().copied());
    normalized.join("")
}

fn is_line_elide(line: &str) -> bool {
    line == "...\n" || line == "..."
}

fn line_matches(mut input: &str, pattern: &str, redactions: &Redactions) -> bool {
    if input == pattern {
        return true;
    }

    let pattern = redactions.clear(pattern);
    let mut sections = pattern.split("[..]").peekable();
    while let Some(section) = sections.next() {
        if let Some(remainder) = input.strip_prefix(section) {
            if let Some(next_section) = sections.peek() {
                if next_section.is_empty() {
                    input = "";
                } else if let Some(restart_index) = remainder.find(next_section) {
                    input = &remainder[restart_index..];
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
    use std::path::PathBuf;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn str_normalize_redactions_empty() {
        let input = "";
        let pattern = "";
        let expected = "";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_literals_match() {
        let input = "Hello\nWorld";
        let pattern = "Hello\nWorld";
        let expected = "Hello\nWorld";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_pattern_shorter() {
        let input = "Hello\nWorld";
        let pattern = "Hello\n";
        let expected = "Hello\nWorld";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_input_shorter() {
        let input = "Hello\n";
        let pattern = "Hello\nWorld";
        let expected = "Hello\n";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_all_different() {
        let input = "Hello\nWorld";
        let pattern = "Goodbye\nMoon";
        let expected = "Hello\nWorld";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_middles_diverge() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\nMoon\nGoodbye";
        let expected = "Hello\nWorld\nGoodbye";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_elide_delimited_with_sub() {
        let input = "Hello World\nHow are you?\nGoodbye World";
        let pattern = "Hello [..]\n...\nGoodbye [..]";
        let expected = "Hello [..]\n...\nGoodbye [..]";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_leading_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "...\nGoodbye";
        let expected = "...\nGoodbye";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_trailing_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...";
        let expected = "Hello\n...";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_middle_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...\nGoodbye";
        let expected = "Hello\n...\nGoodbye";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_post_elide_diverge() {
        let input = "Hello\nSun\nAnd\nWorld";
        let pattern = "Hello\n...\nMoon";
        let expected = "Hello\nSun\nAnd\nWorld";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_post_diverge_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nMoon\nGoodbye\n...";
        let expected = "Hello\nWorld\nGoodbye\nSir";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

    #[test]
    fn str_normalize_redactions_inline_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nW[..]d\nGoodbye\nSir";
        let expected = "Hello\nW[..]d\nGoodbye\nSir";
        let actual = NormalizeToExpected::new()
            .redact()
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, expected.into_data());
    }

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
            assert_eq!(expected, actual, "line={:?}  pattern={:?}", line, pattern);
        }
    }

    #[test]
    fn str_normalize_redactions_user_literal() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert("[OBJECT]", "world").unwrap();
        let actual = NormalizeToExpected::new()
            .redact_with(&sub)
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, pattern.into_data());
    }

    #[test]
    fn str_normalize_redactions_user_path() {
        let input = "input: /home/epage";
        let pattern = "input: [HOME]";
        let mut sub = Redactions::new();
        let sep = std::path::MAIN_SEPARATOR.to_string();
        let redacted = PathBuf::from(sep).join("home").join("epage");
        sub.insert("[HOME]", redacted).unwrap();
        let actual = NormalizeToExpected::new()
            .redact_with(&sub)
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, pattern.into_data());
    }

    #[test]
    fn str_normalize_redactions_user_overlapping_path() {
        let input = "\
a: /home/epage
b: /home/epage/snapbox";
        let pattern = "\
a: [A]
b: [B]";
        let mut sub = Redactions::new();
        let sep = std::path::MAIN_SEPARATOR.to_string();
        let redacted = PathBuf::from(&sep).join("home").join("epage");
        sub.insert("[A]", redacted).unwrap();
        let redacted = PathBuf::from(sep)
            .join("home")
            .join("epage")
            .join("snapbox");
        sub.insert("[B]", redacted).unwrap();
        let actual = NormalizeToExpected::new()
            .redact_with(&sub)
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, pattern.into_data());
    }

    #[test]
    fn str_normalize_redactions_user_disabled() {
        let input = "cargo";
        let pattern = "cargo[EXE]";
        let mut sub = Redactions::new();
        sub.insert("[EXE]", "").unwrap();
        let actual = NormalizeToExpected::new()
            .redact_with(&sub)
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, pattern.into_data());
    }

    #[test]
    #[cfg(feature = "regex")]
    fn str_normalize_redactions_user_regex_unnamed() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert("[OBJECT]", regex::Regex::new("world").unwrap())
            .unwrap();
        let actual = NormalizeToExpected::new()
            .redact_with(&sub)
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, pattern.into_data());
    }

    #[test]
    #[cfg(feature = "regex")]
    fn str_normalize_redactions_user_regex_named() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert(
            "[OBJECT]",
            regex::Regex::new("(?<redacted>world)!").unwrap(),
        )
        .unwrap();
        let actual = NormalizeToExpected::new()
            .redact_with(&sub)
            .normalize(input.into(), &pattern.into());
        assert_eq!(actual, pattern.into_data());
    }
}
