use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

/// Replace data with placeholders
///
/// This can be used for:
/// - Handling test-run dependent data like temp directories or elapsed time
/// - Making special characters more obvious (e.g. redacting a tab a `[TAB]`)
/// - Normalizing platform-specific data like [`std::env::consts::EXE_SUFFIX`]
///
/// # Examples
///
/// ```rust
/// let mut subst = snapbox::Redactions::new();
/// subst.insert("[LOCATION]", "World");
/// assert_eq!(subst.redact("Hello World!"), "Hello [LOCATION]!");
/// ```
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Redactions {
    vars: Option<
        std::collections::BTreeMap<RedactedValueInner, std::collections::BTreeSet<&'static str>>,
    >,
    unused: Option<std::collections::BTreeSet<RedactedValueInner>>,
}

impl Redactions {
    pub const fn new() -> Self {
        Self {
            vars: None,
            unused: None,
        }
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
    ///
    /// With the `regex` feature, you can define patterns using regexes.
    /// You can choose to replace a subset of the regex by giving it the named capture group
    /// `redacted`.
    ///
    /// ```rust
    /// # #[cfg(feature = "regex")] {
    /// let mut subst = snapbox::Redactions::new();
    /// subst.insert("[OBJECT]", regex::Regex::new("(?<redacted>(world|moon))").unwrap());
    /// assert_eq!(subst.redact("Hello world!"), "Hello [OBJECT]!");
    /// assert_eq!(subst.redact("Hello moon!"), "Hello [OBJECT]!");
    /// assert_eq!(subst.redact("Hello other!"), "Hello other!");
    /// # }
    /// ```
    pub fn insert(
        &mut self,
        placeholder: &'static str,
        value: impl Into<RedactedValue>,
    ) -> crate::assert::Result<()> {
        let placeholder = validate_placeholder(placeholder)?;
        let value = value.into();
        if let Some(value) = value.inner {
            self.vars
                .get_or_insert(std::collections::BTreeMap::new())
                .entry(value)
                .or_default()
                .insert(placeholder);
        } else {
            self.unused
                .get_or_insert(std::collections::BTreeSet::new())
                .insert(RedactedValueInner::Str(placeholder));
        }
        Ok(())
    }

    /// Insert additional match patterns
    ///
    /// Placeholders must be enclosed in `[` and `]`.
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
        self.vars
            .get_or_insert(std::collections::BTreeMap::new())
            .retain(|_value, placeholders| {
                placeholders.retain(|p| *p != placeholder);
                !placeholders.is_empty()
            });
        Ok(())
    }

    /// Apply redaction only, no pattern-dependent globs
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut subst = snapbox::Redactions::new();
    /// subst.insert("[LOCATION]", "World");
    /// let output = subst.redact("Hello World!");
    /// assert_eq!(output, "Hello [LOCATION]!");
    /// ```
    pub fn redact(&self, input: &str) -> String {
        let mut input = input.to_owned();
        replace_many(
            &mut input,
            self.vars
                .iter()
                .flatten()
                .flat_map(|(value, placeholders)| {
                    placeholders
                        .iter()
                        .map(move |placeholder| (value, *placeholder))
                }),
        );
        input
    }

    /// Clear unused redactions from expected data
    ///
    /// Some redactions can be conditionally present, like redacting [`std::env::consts::EXE_SUFFIX`].
    /// When the redaction is not present, it needs to be removed from the expected data so it can
    /// be matched against the actual data.
    pub fn clear_unused<'v>(&self, pattern: &'v str) -> Cow<'v, str> {
        if !self.unused.as_ref().map(|s| s.is_empty()).unwrap_or(false) && pattern.contains('[') {
            let mut pattern = pattern.to_owned();
            replace_many(
                &mut pattern,
                self.unused.iter().flatten().map(|var| (var, "")),
            );
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

#[derive(Clone, Debug)]
enum RedactedValueInner {
    Str(&'static str),
    String(String),
    Path {
        native: String,
        normalized: String,
    },
    #[cfg(feature = "regex")]
    Regex(regex::Regex),
}

impl RedactedValueInner {
    fn find_in(&self, buffer: &str) -> Option<std::ops::Range<usize>> {
        match self {
            Self::Str(s) => buffer.find(s).map(|offset| offset..(offset + s.len())),
            Self::String(s) => buffer.find(s).map(|offset| offset..(offset + s.len())),
            Self::Path { native, normalized } => {
                match (buffer.find(native), buffer.find(normalized)) {
                    (Some(native_offset), Some(normalized_offset)) => {
                        if native_offset <= normalized_offset {
                            Some(native_offset..(native_offset + native.len()))
                        } else {
                            Some(normalized_offset..(normalized_offset + normalized.len()))
                        }
                    }
                    (Some(offset), None) => Some(offset..(offset + native.len())),
                    (None, Some(offset)) => Some(offset..(offset + normalized.len())),
                    (None, None) => None,
                }
            }
            #[cfg(feature = "regex")]
            Self::Regex(r) => {
                let captures = r.captures(buffer)?;
                let m = captures.name("redacted").or_else(|| captures.get(0))?;
                Some(m.range())
            }
        }
    }

    fn as_cmp(&self) -> (usize, std::cmp::Reverse<usize>, &str) {
        match self {
            Self::Str(s) => (0, std::cmp::Reverse(s.len()), s),
            Self::String(s) => (0, std::cmp::Reverse(s.len()), s),
            Self::Path { normalized: s, .. } => (0, std::cmp::Reverse(s.len()), s),
            #[cfg(feature = "regex")]
            Self::Regex(r) => {
                let s = r.as_str();
                (1, std::cmp::Reverse(s.len()), s)
            }
        }
    }
}

impl From<&'static str> for RedactedValue {
    fn from(inner: &'static str) -> Self {
        if inner.is_empty() {
            Self { inner: None }
        } else {
            Self {
                inner: Some(RedactedValueInner::Str(inner)),
            }
        }
    }
}

impl From<String> for RedactedValue {
    fn from(inner: String) -> Self {
        if inner.is_empty() {
            Self { inner: None }
        } else {
            Self {
                inner: Some(RedactedValueInner::String(inner)),
            }
        }
    }
}

impl From<&'_ String> for RedactedValue {
    fn from(inner: &'_ String) -> Self {
        inner.clone().into()
    }
}

impl From<Cow<'static, str>> for RedactedValue {
    fn from(inner: Cow<'static, str>) -> Self {
        match inner {
            Cow::Borrowed(s) => s.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

impl From<&'static Path> for RedactedValue {
    fn from(inner: &'static Path) -> Self {
        inner.to_owned().into()
    }
}

impl From<PathBuf> for RedactedValue {
    fn from(inner: PathBuf) -> Self {
        if inner.as_os_str().is_empty() {
            Self { inner: None }
        } else {
            let native = match inner.into_os_string().into_string() {
                Ok(s) => s,
                Err(os) => PathBuf::from(os).display().to_string(),
            };
            let normalized = crate::filter::normalize_paths(&native);
            Self {
                inner: Some(RedactedValueInner::Path { native, normalized }),
            }
        }
    }
}

impl From<&'_ PathBuf> for RedactedValue {
    fn from(inner: &'_ PathBuf) -> Self {
        inner.clone().into()
    }
}

#[cfg(feature = "regex")]
impl From<regex::Regex> for RedactedValue {
    fn from(inner: regex::Regex) -> Self {
        Self {
            inner: Some(RedactedValueInner::Regex(inner)),
        }
    }
}

#[cfg(feature = "regex")]
impl From<&'_ regex::Regex> for RedactedValue {
    fn from(inner: &'_ regex::Regex) -> Self {
        inner.clone().into()
    }
}

impl PartialOrd for RedactedValueInner {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RedactedValueInner {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_cmp().cmp(&other.as_cmp())
    }
}

impl PartialEq for RedactedValueInner {
    fn eq(&self, other: &Self) -> bool {
        self.as_cmp().eq(&other.as_cmp())
    }
}

impl Eq for RedactedValueInner {}

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
        return Err(format!("Key `{placeholder}` is not enclosed in []").into());
    }

    if placeholder[1..(placeholder.len() - 1)]
        .find(|c: char| !c.is_ascii_uppercase() && c != '_')
        .is_some()
    {
        return Err(format!("Key `{placeholder}` can only be A-Z but ").into());
    }

    Ok(placeholder)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_placeholder() {
        let cases = [
            ("[HELLO", false),
            ("HELLO]", false),
            ("[HELLO]", true),
            ("[HELLO_WORLD]", true),
            ("[hello]", false),
            ("[HE  O]", false),
        ];
        for (placeholder, expected) in cases {
            let actual = validate_placeholder(placeholder).is_ok();
            assert_eq!(expected, actual, "placeholder={placeholder:?}");
        }
    }
}
