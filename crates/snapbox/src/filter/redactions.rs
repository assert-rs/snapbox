use std::borrow::Cow;

/// Match pattern expressions, see [`Assert`][crate::Assert]
///
/// Built-in placeholders:
/// - `...` on a line of its own: match multiple complete lines
/// - `[..]`: match multiple characters within a line
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Redactions {
    vars: std::collections::BTreeMap<&'static str, std::collections::BTreeSet<RedactedValueInner>>,
    unused: std::collections::BTreeSet<RedactedValueInner>,
}

impl Redactions {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_exe() -> Self {
        let mut redactions = Self::new();
        redactions
            .insert("[EXE]", std::env::consts::EXE_SUFFIX)
            .unwrap();
        redactions
    }

    /// Insert an additional match pattern
    ///
    /// `placeholder` must be enclosed in `[` and `]`.
    ///
    /// ```rust
    /// let mut subst = snapbox::Redactions::new();
    /// subst.insert("[EXE]", std::env::consts::EXE_SUFFIX);
    /// ```
    pub fn insert(
        &mut self,
        placeholder: &'static str,
        value: impl Into<RedactedValue>,
    ) -> crate::assert::Result<()> {
        let placeholder = validate_placeholder(placeholder)?;
        let value = value.into();
        if let Some(inner) = value.inner {
            self.vars.entry(placeholder).or_default().insert(inner);
        } else {
            self.unused.insert(RedactedValueInner::Str(placeholder));
        }
        Ok(())
    }

    /// Insert additional match patterns
    ///
    /// placeholders must be enclosed in `[` and `]`.
    pub fn extend(
        &mut self,
        vars: impl IntoIterator<Item = (&'static str, impl Into<RedactedValue>)>,
    ) -> crate::assert::Result<()> {
        for (placeholder, value) in vars {
            self.insert(placeholder, value)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, placeholder: &'static str) -> crate::assert::Result<()> {
        let placeholder = validate_placeholder(placeholder)?;
        self.vars.remove(placeholder);
        Ok(())
    }

    /// Apply match pattern to `input`
    ///
    /// If `pattern` matches `input`, then `pattern` is returned.
    ///
    /// Otherwise, `input`, with as many patterns replaced as possible, will be returned.
    ///
    /// ```rust
    /// let subst = snapbox::Redactions::new();
    /// let output = subst.normalize("Hello World!", "Hello [..]!");
    /// assert_eq!(output, "Hello [..]!");
    /// ```
    pub fn normalize(&self, input: &str, pattern: &str) -> String {
        normalize(input, pattern, self)
    }

    fn substitute<'v>(&self, input: &'v str) -> Cow<'v, str> {
        let mut input = input.to_owned();
        replace_many(
            &mut input,
            self.vars
                .iter()
                .flat_map(|(var, replaces)| replaces.iter().map(|replace| (replace, *var))),
        );
        Cow::Owned(input)
    }

    fn clear<'v>(&self, pattern: &'v str) -> Cow<'v, str> {
        if !self.unused.is_empty() && pattern.contains('[') {
            let mut pattern = pattern.to_owned();
            replace_many(&mut pattern, self.unused.iter().map(|var| (var, "")));
            Cow::Owned(pattern)
        } else {
            Cow::Borrowed(pattern)
        }
    }
}

#[derive(Clone)]
pub struct RedactedValue {
    inner: Option<RedactedValueInner>,
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
enum RedactedValueInner {
    Str(&'static str),
    String(String),
}

impl RedactedValueInner {
    fn find_in(&self, buffer: &str) -> Option<std::ops::Range<usize>> {
        match self {
            Self::Str(s) => buffer.find(s).map(|offset| offset..(offset + s.len())),
            Self::String(s) => buffer.find(s).map(|offset| offset..(offset + s.len())),
        }
    }
}

impl<C> From<C> for RedactedValue
where
    C: Into<Cow<'static, str>>,
{
    fn from(inner: C) -> Self {
        let inner = inner.into();
        if inner.is_empty() {
            Self { inner: None }
        } else {
            Self {
                inner: Some(RedactedValueInner::String(crate::filter::normalize_paths(
                    &crate::filter::normalize_lines(&inner),
                ))),
            }
        }
    }
}

/// Replacements is `(from, to)`
fn replace_many<'a>(
    buffer: &mut String,
    replacements: impl IntoIterator<Item = (&'a RedactedValueInner, &'a str)>,
) {
    for (var, replace) in replacements {
        let mut index = 0;
        while let Some(offset) = var.find_in(&buffer[index..]) {
            let old_range = (index + offset.start)..(index + offset.end);
            buffer.replace_range(old_range, replace);
            index += offset.start + replace.len();
        }
    }
}

fn validate_placeholder(placeholder: &'static str) -> crate::assert::Result<&'static str> {
    if !placeholder.starts_with('[') || !placeholder.ends_with(']') {
        return Err(format!("Key `{}` is not enclosed in []", placeholder).into());
    }

    if placeholder[1..(placeholder.len() - 1)]
        .find(|c: char| !c.is_ascii_uppercase())
        .is_some()
    {
        return Err(format!("Key `{}` can only be A-Z but ", placeholder).into());
    }

    Ok(placeholder)
}

fn normalize(input: &str, pattern: &str, redactions: &Redactions) -> String {
    if input == pattern {
        return input.to_owned();
    }

    let input = redactions.substitute(input);

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
    use super::*;

    #[test]
    fn empty() {
        let input = "";
        let pattern = "";
        let expected = "";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn literals_match() {
        let input = "Hello\nWorld";
        let pattern = "Hello\nWorld";
        let expected = "Hello\nWorld";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn pattern_shorter() {
        let input = "Hello\nWorld";
        let pattern = "Hello\n";
        let expected = "Hello\nWorld";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn input_shorter() {
        let input = "Hello\n";
        let pattern = "Hello\nWorld";
        let expected = "Hello\n";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn all_different() {
        let input = "Hello\nWorld";
        let pattern = "Goodbye\nMoon";
        let expected = "Hello\nWorld";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn middles_diverge() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\nMoon\nGoodbye";
        let expected = "Hello\nWorld\nGoodbye";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn elide_delimited_with_sub() {
        let input = "Hello World\nHow are you?\nGoodbye World";
        let pattern = "Hello [..]\n...\nGoodbye [..]";
        let expected = "Hello [..]\n...\nGoodbye [..]";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn leading_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "...\nGoodbye";
        let expected = "...\nGoodbye";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn trailing_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...";
        let expected = "Hello\n...";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn middle_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...\nGoodbye";
        let expected = "Hello\n...\nGoodbye";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_elide_diverge() {
        let input = "Hello\nSun\nAnd\nWorld";
        let pattern = "Hello\n...\nMoon";
        let expected = "Hello\nSun\nAnd\nWorld";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_diverge_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nMoon\nGoodbye\n...";
        let expected = "Hello\nWorld\nGoodbye\nSir";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn inline_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nW[..]d\nGoodbye\nSir";
        let expected = "Hello\nW[..]d\nGoodbye\nSir";
        let actual = normalize(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn line_matches_cases() {
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
    fn test_validate_placeholder() {
        let cases = [
            ("[HELLO", false),
            ("HELLO]", false),
            ("[HELLO]", true),
            ("[hello]", false),
            ("[HE  O]", false),
        ];
        for (placeholder, expected) in cases {
            let actual = validate_placeholder(placeholder).is_ok();
            assert_eq!(expected, actual, "placeholder={:?}", placeholder);
        }
    }

    #[test]
    fn substitute_literal() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert("[OBJECT]", "world").unwrap();
        let actual = normalize(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }

    #[test]
    fn substitute_disabled() {
        let input = "cargo";
        let pattern = "cargo[EXE]";
        let mut sub = Redactions::new();
        sub.insert("[EXE]", "").unwrap();
        let actual = normalize(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }
}
