use std::collections::BTreeMap;

use super::Data;
use super::Inline;
use super::Position;

pub(crate) fn get() -> std::sync::MutexGuard<'static, Runtime> {
    static RT: std::sync::Mutex<Runtime> = std::sync::Mutex::new(Runtime::new());
    RT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Default)]
pub(crate) struct Runtime {
    per_file: Vec<SourceFileRuntime>,
    path_count: Vec<PathRuntime>,
}

impl Runtime {
    const fn new() -> Self {
        Self {
            per_file: Vec::new(),
            path_count: Vec::new(),
        }
    }

    pub(crate) fn count(&mut self, path_prefix: &str) -> usize {
        if let Some(entry) = self
            .path_count
            .iter_mut()
            .find(|entry| entry.is(path_prefix))
        {
            entry.next()
        } else {
            let entry = PathRuntime::new(path_prefix);
            let next = entry.count();
            self.path_count.push(entry);
            next
        }
    }

    pub(crate) fn write(&mut self, actual: &Data, inline: &Inline) -> std::io::Result<()> {
        let actual = actual.render().expect("`actual` must be UTF-8");
        if let Some(entry) = self
            .per_file
            .iter_mut()
            .find(|f| f.path == inline.position.file)
        {
            entry.update(&actual, inline)?;
        } else {
            let mut entry = SourceFileRuntime::new(inline)?;
            entry.update(&actual, inline)?;
            self.per_file.push(entry);
        }

        Ok(())
    }
}

struct SourceFileRuntime {
    path: std::path::PathBuf,
    original_text: String,
    patchwork: Patchwork,
}

impl SourceFileRuntime {
    fn new(inline: &Inline) -> std::io::Result<SourceFileRuntime> {
        let path = inline.position.file.clone();
        let original_text = std::fs::read_to_string(&path)?;
        let patchwork = Patchwork::new(original_text.clone());
        Ok(SourceFileRuntime {
            path,
            original_text,
            patchwork,
        })
    }
    fn update(&mut self, actual: &str, inline: &Inline) -> std::io::Result<()> {
        let span = Span::from_pos(&inline.position, &self.original_text);
        let patch = format_patch(actual);
        self.patchwork.patch(span.literal_range, &patch)?;
        std::fs::write(&inline.position.file, &self.patchwork.text)
    }
}

#[derive(Debug)]
struct Patchwork {
    text: String,
    indels: BTreeMap<OrdRange, (usize, String)>,
}

impl Patchwork {
    fn new(text: String) -> Patchwork {
        Patchwork {
            text,
            indels: BTreeMap::new(),
        }
    }
    fn patch(&mut self, mut range: std::ops::Range<usize>, patch: &str) -> std::io::Result<()> {
        let key: OrdRange = range.clone().into();
        match self.indels.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((patch.len(), patch.to_owned()));
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                if entry.get().1 == patch {
                    return Ok(());
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "cannot update as it was already modified",
                    ));
                }
            }
        }

        let (delete, insert) = self
            .indels
            .iter()
            .take_while(|(delete, _)| delete.start < range.start)
            .map(|(delete, (insert, _))| (delete.end - delete.start, insert))
            .fold((0usize, 0usize), |(x1, y1), (x2, y2)| (x1 + x2, y1 + y2));

        for pos in &mut [&mut range.start, &mut range.end] {
            **pos -= delete;
            **pos += insert;
        }

        self.text.replace_range(range, patch);
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct OrdRange {
    start: usize,
    end: usize,
}

impl From<std::ops::Range<usize>> for OrdRange {
    fn from(other: std::ops::Range<usize>) -> Self {
        Self {
            start: other.start,
            end: other.end,
        }
    }
}

fn lit_kind_for_patch(patch: &str) -> StrLitKind {
    let has_dquote = patch.chars().any(|c| c == '"');
    if !has_dquote {
        let has_bslash_or_newline = patch.chars().any(|c| matches!(c, '\\' | '\n'));
        return if has_bslash_or_newline {
            StrLitKind::Raw(1)
        } else {
            StrLitKind::Normal
        };
    }

    // Find the maximum number of hashes that follow a double quote in the string.
    // We need to use one more than that to delimit the string.
    let leading_hashes = |s: &str| s.chars().take_while(|&c| c == '#').count();
    let max_hashes = patch.split('"').map(leading_hashes).max().unwrap();
    StrLitKind::Raw(max_hashes + 1)
}

fn format_patch(patch: &str) -> String {
    let lit_kind = lit_kind_for_patch(patch);
    let is_multiline = patch.contains('\n');

    let mut buf = String::new();
    if matches!(lit_kind, StrLitKind::Raw(_)) {
        buf.push('[');
    }
    lit_kind.write_start(&mut buf).unwrap();
    if is_multiline {
        buf.push('\n');
    }
    buf.push_str(patch);
    if is_multiline {
        buf.push('\n');
    }
    lit_kind.write_end(&mut buf).unwrap();
    if matches!(lit_kind, StrLitKind::Raw(_)) {
        buf.push(']');
    }
    buf
}

#[derive(Clone, Debug)]
struct Span {
    /// The byte range of the argument to `expect!`, including the inner `[]` if it exists.
    literal_range: std::ops::Range<usize>,
}

impl Span {
    fn from_pos(pos: &Position, file: &str) -> Span {
        let mut target_line = None;
        let mut line_start = 0;
        for (i, line) in crate::utils::LinesWithTerminator::new(file).enumerate() {
            if i == pos.line as usize - 1 {
                // `column` points to the first character of the macro invocation:
                //
                //    expect![[r#""#]]        expect![""]
                //    ^       ^               ^       ^
                //  column   offset                 offset
                //
                // Seek past the exclam, then skip any whitespace and
                // the macro delimiter to get to our argument.
                #[allow(clippy::skip_while_next)]
                let byte_offset = line
                    .char_indices()
                    .skip((pos.column - 1).try_into().unwrap())
                    .skip_while(|&(_, c)| c != '!')
                    .skip(1) // !
                    .skip_while(|&(_, c)| c.is_whitespace())
                    .skip(1) // [({
                    .skip_while(|&(_, c)| c.is_whitespace())
                    .next()
                    .expect("Failed to parse macro invocation")
                    .0;

                let literal_start = line_start + byte_offset;
                target_line = Some(literal_start);
                break;
            }
            line_start += line.len();
        }
        let literal_start = target_line.unwrap();

        let lit_to_eof = &file[literal_start..];
        let lit_to_eof_trimmed = lit_to_eof.trim_start();

        let literal_start = literal_start + (lit_to_eof.len() - lit_to_eof_trimmed.len());

        let literal_len =
            locate_end(lit_to_eof_trimmed).expect("Couldn't find closing delimiter for `expect!`.");
        let literal_range = literal_start..literal_start + literal_len;
        Span { literal_range }
    }
}

fn locate_end(arg_start_to_eof: &str) -> Option<usize> {
    match arg_start_to_eof.chars().next()? {
        c if c.is_whitespace() => panic!("skip whitespace before calling `locate_end`"),

        // expect![[]]
        '[' => {
            let str_start_to_eof = arg_start_to_eof[1..].trim_start();
            let str_len = find_str_lit_len(str_start_to_eof)?;
            let str_end_to_eof = &str_start_to_eof[str_len..];
            let closing_brace_offset = str_end_to_eof.find(']')?;
            Some((arg_start_to_eof.len() - str_end_to_eof.len()) + closing_brace_offset + 1)
        }

        // expect![] | expect!{} | expect!()
        ']' | '}' | ')' => Some(0),

        // expect!["..."] | expect![r#"..."#]
        _ => find_str_lit_len(arg_start_to_eof),
    }
}

/// Parses a string literal, returning the byte index of its last character
/// (either a quote or a hash).
fn find_str_lit_len(str_lit_to_eof: &str) -> Option<usize> {
    fn try_find_n_hashes(
        s: &mut impl Iterator<Item = char>,
        desired_hashes: usize,
    ) -> Option<(usize, Option<char>)> {
        let mut n = 0;
        loop {
            match s.next()? {
                '#' => n += 1,
                c => return Some((n, Some(c))),
            }

            if n == desired_hashes {
                return Some((n, None));
            }
        }
    }

    let mut s = str_lit_to_eof.chars();
    let kind = match s.next()? {
        '"' => StrLitKind::Normal,
        'r' => {
            let (n, c) = try_find_n_hashes(&mut s, usize::MAX)?;
            if c != Some('"') {
                return None;
            }
            StrLitKind::Raw(n)
        }
        _ => return None,
    };

    let mut oldc = None;
    loop {
        let c = oldc.take().or_else(|| s.next())?;
        match (c, kind) {
            ('\\', StrLitKind::Normal) => {
                let _escaped = s.next()?;
            }
            ('"', StrLitKind::Normal) => break,
            ('"', StrLitKind::Raw(0)) => break,
            ('"', StrLitKind::Raw(n)) => {
                let (seen, c) = try_find_n_hashes(&mut s, n)?;
                if seen == n {
                    break;
                }
                oldc = c;
            }
            _ => {}
        }
    }

    Some(str_lit_to_eof.len() - s.as_str().len())
}

#[derive(Copy, Clone)]
enum StrLitKind {
    Normal,
    Raw(usize),
}

impl StrLitKind {
    fn write_start(self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Normal => write!(w, "\""),
            Self::Raw(n) => {
                write!(w, "r")?;
                for _ in 0..n {
                    write!(w, "#")?;
                }
                write!(w, "\"")
            }
        }
    }

    fn write_end(self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Normal => write!(w, "\""),
            Self::Raw(n) => {
                write!(w, "\"")?;
                for _ in 0..n {
                    write!(w, "#")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone)]
struct PathRuntime {
    path_prefix: String,
    count: usize,
}

impl PathRuntime {
    fn new(path_prefix: &str) -> Self {
        Self {
            path_prefix: path_prefix.to_owned(),
            count: 0,
        }
    }

    fn is(&self, path_prefix: &str) -> bool {
        self.path_prefix == path_prefix
    }

    fn next(&mut self) -> usize {
        self.count += 1;
        self.count
    }

    fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_data_eq;
    use crate::prelude::*;
    use crate::str;

    #[test]
    fn test_format_patch() {
        let patch = format_patch("hello\nworld\n");

        assert_data_eq!(
            patch,
            str![[r##"
[r#"
hello
world

"#]
"##]],
        );

        let patch = format_patch(r"hello\tworld");
        assert_data_eq!(patch, str![[r##"[r#"hello\tworld"#]"##]].raw());

        let patch = format_patch("{\"foo\": 42}");
        assert_data_eq!(patch, str![[r##"[r#"{"foo": 42}"#]"##]]);
    }

    #[test]
    fn test_patchwork() {
        let mut patchwork = Patchwork::new("one two three".to_owned());
        patchwork.patch(4..7, "zwei").unwrap();
        patchwork.patch(0..3, "один").unwrap();
        patchwork.patch(8..13, "3").unwrap();
        assert_data_eq!(
            patchwork.to_debug(),
            str![[r#"
Patchwork {
    text: "один zwei 3",
    indels: {
        OrdRange {
            start: 0,
            end: 3,
        }: (
            8,
            "один",
        ),
        OrdRange {
            start: 4,
            end: 7,
        }: (
            4,
            "zwei",
        ),
        OrdRange {
            start: 8,
            end: 13,
        }: (
            1,
            "3",
        ),
    },
}

"#]],
        );
    }

    #[test]
    fn test_patchwork_overlap_diverge() {
        let mut patchwork = Patchwork::new("one two three".to_owned());
        patchwork.patch(4..7, "zwei").unwrap();
        patchwork.patch(4..7, "abcd").unwrap_err();
        assert_data_eq!(
            patchwork.to_debug(),
            str![[r#"
Patchwork {
    text: "one zwei three",
    indels: {
        OrdRange {
            start: 4,
            end: 7,
        }: (
            4,
            "zwei",
        ),
    },
}

"#]],
        );
    }

    #[test]
    fn test_patchwork_overlap_converge() {
        let mut patchwork = Patchwork::new("one two three".to_owned());
        patchwork.patch(4..7, "zwei").unwrap();
        patchwork.patch(4..7, "zwei").unwrap();
        assert_data_eq!(
            patchwork.to_debug(),
            str![[r#"
Patchwork {
    text: "one zwei three",
    indels: {
        OrdRange {
            start: 4,
            end: 7,
        }: (
            4,
            "zwei",
        ),
    },
}

"#]],
        );
    }

    #[test]
    fn test_locate() {
        macro_rules! check_locate {
            ($( [[$s:literal]] ),* $(,)?) => {$({
                let lit = stringify!($s);
                let with_trailer = format!("{} \t]]\n", lit);
                assert_eq!(locate_end(&with_trailer), Some(lit.len()));
            })*};
        }

        // Check that we handle string literals containing "]]" correctly.
        check_locate!(
            [[r#"{ arr: [[1, 2], [3, 4]], other: "foo" } "#]],
            [["]]"]],
            [["\"]]"]],
            [[r#""]]"#]],
        );

        // Check `str![[  ]]` as well.
        assert_eq!(locate_end("]]"), Some(0));
    }

    #[test]
    fn test_find_str_lit_len() {
        macro_rules! check_str_lit_len {
            ($( $s:literal ),* $(,)?) => {$({
                let lit = stringify!($s);
                assert_eq!(find_str_lit_len(lit), Some(lit.len()));
            })*}
        }

        check_str_lit_len![
            r##"foa\""#"##,
            r##"

                asdf][]]""""#
            "##,
            "",
            "\"",
            "\"\"",
            "#\"#\"#",
        ];
    }
}
