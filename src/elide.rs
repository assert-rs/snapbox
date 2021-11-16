use std::borrow::Cow;

pub(crate) fn normalize(input: &str, pattern: &str) -> String {
    if input == pattern {
        return input.to_owned();
    }

    let mut normalized: Vec<Cow<str>> = Vec::new();
    let input_lines: Vec<_> = crate::lines::LinesWithTerminator::new(input).collect();
    let pattern_lines: Vec<_> = crate::lines::LinesWithTerminator::new(pattern).collect();

    let mut input_index = 0;
    let mut pattern_index = 0;
    'outer: loop {
        let pattern_line = if let Some(pattern_line) = pattern_lines.get(pattern_index) {
            *pattern_line
        } else {
            normalized.extend(
                input_lines[input_index..]
                    .iter()
                    .copied()
                    .map(Cow::Borrowed),
            );
            break 'outer;
        };
        let next_pattern_index = pattern_index + 1;

        let input_line = if let Some(input_line) = input_lines.get(input_index) {
            *input_line
        } else {
            break 'outer;
        };
        let next_input_index = input_index + 1;

        if input_line == pattern_line {
            pattern_index = next_pattern_index;
            input_index = next_input_index;
            normalized.push(Cow::Borrowed(pattern_line));
            continue 'outer;
        } else if is_line_elide(pattern_line) {
            let next_pattern_line: &str =
                if let Some(pattern_line) = pattern_lines.get(next_pattern_index) {
                    pattern_line
                } else {
                    normalized.push(Cow::Borrowed(pattern_line));
                    break 'outer;
                };
            if let Some(future_input_index) = input_lines[input_index..]
                .iter()
                .enumerate()
                .find(|(_, l)| **l == next_pattern_line)
                .map(|(i, _)| input_index + i)
            {
                normalized.push(Cow::Borrowed(pattern_line));
                pattern_index = next_pattern_index;
                input_index = future_input_index;
                continue 'outer;
            } else {
                normalized.extend(
                    input_lines[input_index..]
                        .iter()
                        .copied()
                        .map(Cow::Borrowed),
                );
                break 'outer;
            }
        } else if line_matches(input_line, pattern_line) {
            pattern_index = next_pattern_index;
            input_index = next_input_index;
            normalized.push(Cow::Borrowed(pattern_line));
            continue 'outer;
        } else {
            // Find where we can pick back up for normalizing
            for future_input_index in next_input_index..input_lines.len() {
                let future_input_line = input_lines[future_input_index];
                if let Some(future_pattern_index) = pattern_lines[next_pattern_index..]
                    .iter()
                    .enumerate()
                    .find(|(_, l)| **l == future_input_line || is_line_elide(**l))
                    .map(|(i, _)| next_pattern_index + i)
                {
                    normalized.extend(
                        input_lines[input_index..future_input_index]
                            .iter()
                            .copied()
                            .map(Cow::Borrowed),
                    );
                    pattern_index = future_pattern_index;
                    input_index = future_input_index;
                    continue 'outer;
                }
            }

            normalized.extend(
                input_lines[input_index..]
                    .iter()
                    .copied()
                    .map(Cow::Borrowed),
            );
            break 'outer;
        }
    }

    normalized.join("")
}

fn is_line_elide(line: &str) -> bool {
    line == "...\n" || line == "..."
}

fn line_matches(mut line: &str, pattern: &str) -> bool {
    let mut sections = pattern.split("[..]").peekable();
    while let Some(section) = sections.next() {
        if let Some(remainder) = line.strip_prefix(section) {
            if let Some(next_section) = sections.peek() {
                if next_section.is_empty() {
                    line = "";
                } else if let Some(restart_index) = remainder.find(next_section) {
                    line = &remainder[restart_index..];
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
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn literals_match() {
        let input = "Hello\nWorld";
        let pattern = "Hello\nWorld";
        let expected = "Hello\nWorld";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn pattern_shorter() {
        let input = "Hello\nWorld";
        let pattern = "Hello\n";
        let expected = "Hello\nWorld";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn input_shorter() {
        let input = "Hello\n";
        let pattern = "Hello\nWorld";
        let expected = "Hello\n";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn all_different() {
        let input = "Hello\nWorld";
        let pattern = "Goodbye\nMoon";
        let expected = "Hello\nWorld";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn middles_diverge() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\nMoon\nGoodbye";
        let expected = "Hello\nWorld\nGoodbye";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn leading_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "...\nGoodbye";
        let expected = "...\nGoodbye";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn trailing_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...";
        let expected = "Hello\n...";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn middle_elide() {
        let input = "Hello\nWorld\nGoodbye";
        let pattern = "Hello\n...\nGoodbye";
        let expected = "Hello\n...\nGoodbye";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_elide_diverge() {
        let input = "Hello\nSun\nAnd\nWorld";
        let pattern = "Hello\n...\nMoon";
        let expected = "Hello\nSun\nAnd\nWorld";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn post_diverge_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nMoon\nGoodbye\n...";
        let expected = "Hello\nWorld\nGoodbye\n...";
        let actual = normalize(input, pattern);
        assert_eq!(expected, actual);
    }

    #[test]
    fn inline_elide() {
        let input = "Hello\nWorld\nGoodbye\nSir";
        let pattern = "Hello\nW[..]d\nGoodbye\nSir";
        let expected = "Hello\nW[..]d\nGoodbye\nSir";
        let actual = normalize(input, pattern);
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
            let actual = line_matches(line, pattern);
            assert_eq!(actual, expected, "line={:?}  pattern={:?}", line, pattern);
        }
    }
}
