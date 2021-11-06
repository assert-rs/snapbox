pub(crate) fn normalize(input: &str, pattern: &str) -> String {
    if input == pattern {
        return input.to_owned();
    }

    let mut normalized: Vec<&str> = Vec::new();
    let input_lines: Vec<_> = LinesWithTerminator::new(input).collect();
    let pattern_lines: Vec<_> = LinesWithTerminator::new(pattern).collect();

    let mut input_index = 0;
    let mut pattern_index = 0;
    'outer: loop {
        let pattern_line = if let Some(pattern_line) = pattern_lines.get(pattern_index) {
            *pattern_line
        } else {
            normalized.extend(&input_lines[input_index..]);
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
            normalized.push(input_line);
            continue 'outer;
        }

        if is_line_elide(pattern_line) {
            let next_pattern_line: &str =
                if let Some(pattern_line) = pattern_lines.get(next_pattern_index) {
                    pattern_line
                } else {
                    normalized.push(pattern_line);
                    break 'outer;
                };
            if let Some(future_input_index) = input_lines[input_index..]
                .iter()
                .enumerate()
                .find(|(_, l)| **l == next_pattern_line)
                .map(|(i, _)| input_index + i)
            {
                normalized.push(pattern_line);
                pattern_index = next_pattern_index;
                input_index = future_input_index;
                continue 'outer;
            } else {
                normalized.extend(&input_lines[input_index..]);
                break 'outer;
            }
        }

        for future_input_index in next_input_index..input_lines.len() {
            let future_input_line = input_lines[future_input_index];
            if let Some(future_pattern_index) = pattern_lines[next_pattern_index..]
                .iter()
                .enumerate()
                .find(|(_, l)| **l == future_input_line || is_line_elide(**l))
                .map(|(i, _)| next_pattern_index + i)
            {
                normalized.extend(&input_lines[input_index..future_input_index]);
                pattern_index = future_pattern_index;
                input_index = future_input_index;
                continue 'outer;
            }
        }

        normalized.extend(&input_lines[input_index..]);
        break 'outer;
    }

    normalized.join("")
}

fn is_line_elide(line: &str) -> bool {
    line == "...\n" || line == "..."
}

#[derive(Clone, Debug)]
struct LinesWithTerminator<'a> {
    data: &'a str,
}

impl<'a> LinesWithTerminator<'a> {
    fn new(data: &'a str) -> LinesWithTerminator<'a> {
        LinesWithTerminator { data }
    }
}

impl<'a> Iterator for LinesWithTerminator<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        match self.data.find('\n') {
            None if self.data.is_empty() => None,
            None => {
                let line = self.data;
                self.data = "";
                Some(line)
            }
            Some(end) => {
                let line = &self.data[..end + 1];
                self.data = &self.data[end + 1..];
                Some(line)
            }
        }
    }
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
}
