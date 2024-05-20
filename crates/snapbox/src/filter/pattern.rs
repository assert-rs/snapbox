use super::Redactions;

pub(crate) fn normalize_to_pattern(input: &str, pattern: &str, redactions: &Redactions) -> String {
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

    #[test]
    fn empty() {
        let input = "";
        let pattern = "";
        let expected = "";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn literals_match() {
        let input = "Hello\nWorld";
        let pattern = "Hello\nWorld";
        let expected = "Hello\nWorld";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn pattern_shorter() {
        let input = "Hello\nWorld";
        let pattern = "Hello\n";
        let expected = "Hello\nWorld";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn input_shorter() {
        let input = "Hello\n";
        let pattern = "Hello\nWorld";
        let expected = "Hello\n";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn all_different() {
        let input = "Hello\nWorld";
        let pattern = "Goodbye\nMoon";
        let expected = "Hello\nWorld";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn middles_diverge() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\nMoon\nGoodbye";
        let expected = "Hello\nWorld\nGoodbye";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn elide_delimited_with_sub() {
        let input = "Hello World\nHow are you?\nGoodbye World";
        let pattern = "Hello [..]\n...\nGoodbye [..]";
        let expected = "Hello [..]\n...\nGoodbye [..]";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn leading_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "...\nGoodbye";
        let expected = "...\nGoodbye";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn trailing_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...";
        let expected = "Hello\n...";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn middle_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...\nGoodbye";
        let expected = "Hello\n...\nGoodbye";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_elide_diverge() {
        let input = "Hello\nSun\nAnd\nWorld";
        let pattern = "Hello\n...\nMoon";
        let expected = "Hello\nSun\nAnd\nWorld";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_diverge_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nMoon\nGoodbye\n...";
        let expected = "Hello\nWorld\nGoodbye\nSir";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
        assert_eq!(expected, actual);
    }

    #[test]
    fn inline_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nW[..]d\nGoodbye\nSir";
        let expected = "Hello\nW[..]d\nGoodbye\nSir";
        let actual = normalize_to_pattern(input, pattern, &Redactions::new());
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
    fn substitute_literal() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert("[OBJECT]", "world").unwrap();
        let actual = normalize_to_pattern(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }

    #[test]
    fn substitute_path() {
        let input = "input: /home/epage";
        let pattern = "input: [HOME]";
        let mut sub = Redactions::new();
        let sep = std::path::MAIN_SEPARATOR.to_string();
        let redacted = PathBuf::from(sep).join("home").join("epage");
        sub.insert("[HOME]", redacted).unwrap();
        let actual = normalize_to_pattern(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }

    #[test]
    fn substitute_overlapping_path() {
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
        let actual = normalize_to_pattern(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }

    #[test]
    fn substitute_disabled() {
        let input = "cargo";
        let pattern = "cargo[EXE]";
        let mut sub = Redactions::new();
        sub.insert("[EXE]", "").unwrap();
        let actual = normalize_to_pattern(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }

    #[test]
    #[cfg(feature = "regex")]
    fn substitute_regex_unnamed() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert("[OBJECT]", regex::Regex::new("world").unwrap())
            .unwrap();
        let actual = normalize_to_pattern(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }

    #[test]
    #[cfg(feature = "regex")]
    fn substitute_regex_named() {
        let input = "Hello world!";
        let pattern = "Hello [OBJECT]!";
        let mut sub = Redactions::new();
        sub.insert(
            "[OBJECT]",
            regex::Regex::new("(?<redacted>world)!").unwrap(),
        )
        .unwrap();
        let actual = normalize_to_pattern(input, pattern, &sub);
        assert_eq!(actual, pattern);
    }
}
