pub fn write_diff(
    writer: &mut dyn std::fmt::Write,
    expected: &crate::Data,
    actual: &crate::Data,
    expected_name: &dyn std::fmt::Display,
    actual_name: &dyn std::fmt::Display,
    palette: crate::report::Palette,
) -> Result<(), std::fmt::Error> {
    #[allow(unused_mut)]
    let mut rendered = false;
    #[cfg(feature = "diff")]
    if let (Some(expected), Some(actual)) = (expected.as_str(), actual.as_str()) {
        let diff = render_diff(expected, actual, expected_name, actual_name, palette);
        writeln!(writer, "{}", diff)?;
        rendered = true;
    }

    if !rendered {
        writeln!(writer, "{} {}:", expected_name, palette.info("(expected)"))?;
        writeln!(writer, "{}", palette.info(&expected))?;
        writeln!(writer, "{} {}:", actual_name, palette.error("(actual)"))?;
        writeln!(writer, "{}", palette.error(&actual))?;
    }
    Ok(())
}

#[cfg(feature = "diff")]
fn render_diff(
    expected: &str,
    actual: &str,
    expected_name: impl std::fmt::Display,
    actual_name: impl std::fmt::Display,
    palette: crate::report::Palette,
) -> String {
    diff_inner(
        expected,
        actual,
        &palette.info(expected_name).to_string(),
        &palette.error(actual_name).to_string(),
        palette,
    )
}

#[cfg(feature = "diff")]
fn diff_inner(
    expected: &str,
    actual: &str,
    expected_name: &str,
    actual_name: &str,
    palette: crate::report::Palette,
) -> String {
    let expected: Vec<_> = crate::utils::LinesWithTerminator::new(expected).collect();
    let actual: Vec<_> = crate::utils::LinesWithTerminator::new(actual).collect();
    let diff = difflib::unified_diff(
        &expected,
        &actual,
        expected_name,
        actual_name,
        &palette.info("expected").to_string(),
        &palette.error("actual").to_string(),
        0,
    );
    let mut diff = colorize_diff(diff, palette);
    diff.insert(0, "\n".to_owned());

    diff.join("")
}

#[cfg(feature = "color")]
fn colorize_diff(mut lines: Vec<String>, palette: crate::report::Palette) -> Vec<String> {
    for (i, line) in lines.iter_mut().enumerate() {
        match (i, line.as_bytes().get(0)) {
            (0, _) => {
                if let Some((prefix, body)) = line.split_once(' ') {
                    *line = format!("{} {}", palette.info(prefix), body);
                }
            }
            (1, _) => {
                if let Some((prefix, body)) = line.split_once(' ') {
                    *line = format!("{} {}", palette.error(prefix), body);
                }
            }
            (_, Some(b'-')) => {
                let (prefix, body) = line.split_at(1);
                *line = format!("{}{}", palette.info(prefix), body);
            }
            (_, Some(b'+')) => {
                let (prefix, body) = line.split_at(1);
                *line = format!("{}{}", palette.error(prefix), body);
            }
            (_, Some(b'@')) => {
                *line = format!("{}", palette.hint(&line));
            }
            _ => (),
        }
    }
    lines
}

#[cfg(feature = "diff")]
#[cfg(not(feature = "color"))]
fn colorize_diff(lines: Vec<String>, _palette: crate::report::Palette) -> Vec<String> {
    lines
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "diff")]
    #[test]
    fn diff_eq() {
        let expected = "Hello\nWorld\n";
        let expected_name = "A";
        let actual = "Hello\nWorld\n";
        let actual_name = "B";
        let palette = crate::report::Palette::never();

        let actual_diff = render_diff(expected, actual, expected_name, actual_name, palette);
        let expected_diff = "
";

        assert_eq!(actual_diff, expected_diff);
    }

    #[cfg(feature = "diff")]
    #[test]
    fn diff_ne_line_missing() {
        let expected = "Hello\nWorld\n";
        let expected_name = "A";
        let actual = "Hello\n";
        let actual_name = "B";
        let palette = crate::report::Palette::never();

        let actual_diff = render_diff(expected, actual, expected_name, actual_name, palette);
        let expected_diff = "
--- A\texpected
+++ B\tactual
@@ -2 +1,0 @@
-World
";

        assert_eq!(actual_diff, expected_diff);
    }

    #[cfg(feature = "diff")]
    #[test]
    fn diff_eq_trailing_newline_missing() {
        let expected = "Hello\nWorld\n";
        let expected_name = "A";
        let actual = "Hello\nWorld";
        let actual_name = "B";
        let palette = crate::report::Palette::never();

        let actual_diff = render_diff(expected, actual, expected_name, actual_name, palette);
        let expected_diff = "
--- A\texpected
+++ B\tactual
@@ -2 +2 @@
-World
+World";

        assert_eq!(actual_diff, expected_diff);
    }
}
