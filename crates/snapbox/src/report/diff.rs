pub fn write_diff(
    writer: &mut dyn std::fmt::Write,
    expected: &crate::Data,
    actual: &crate::Data,
    expected_name: Option<&dyn std::fmt::Display>,
    actual_name: Option<&dyn std::fmt::Display>,
    palette: crate::report::Palette,
) -> Result<(), std::fmt::Error> {
    #[allow(unused_mut)]
    let mut rendered = false;
    #[cfg(feature = "diff")]
    if let (Some(expected), Some(actual)) = (expected.as_str(), actual.as_str()) {
        write_diff_inner(
            writer,
            expected,
            actual,
            expected_name,
            actual_name,
            palette,
        )?;
        rendered = true;
    }

    if !rendered {
        if let Some(expected_name) = expected_name {
            writeln!(writer, "{} {}:", expected_name, palette.info("(expected)"))?;
        } else {
            writeln!(writer, "{}:", palette.info("Expected"))?;
        }
        writeln!(writer, "{}", palette.info(&expected))?;
        if let Some(actual_name) = actual_name {
            writeln!(writer, "{} {}:", actual_name, palette.error("(actual)"))?;
        } else {
            writeln!(writer, "{}:", palette.error("Actual"))?;
        }
        writeln!(writer, "{}", palette.error(&actual))?;
    }
    Ok(())
}

#[cfg(feature = "diff")]
fn write_diff_inner(
    writer: &mut dyn std::fmt::Write,
    expected: &str,
    actual: &str,
    expected_name: Option<&dyn std::fmt::Display>,
    actual_name: Option<&dyn std::fmt::Display>,
    palette: crate::report::Palette,
) -> Result<(), std::fmt::Error> {
    let changes = similar::TextDiff::configure()
        .algorithm(similar::Algorithm::Patience)
        .timeout(std::time::Duration::from_millis(500))
        .newline_terminated(false)
        .diff_lines(expected, actual);

    writeln!(writer)?;
    if let Some(expected_name) = expected_name {
        writeln!(
            writer,
            "{}",
            palette.info(format_args!("--- {} (expected)", expected_name))
        )?;
    } else {
        writeln!(writer, "{}", palette.info(format_args!("--- Expected")))?;
    }
    if let Some(actual_name) = actual_name {
        writeln!(
            writer,
            "{}",
            palette.error(format_args!("+++ {} (actual)", actual_name))
        )?;
    } else {
        writeln!(writer, "{}", palette.error(format_args!("+++ Actual")))?;
    }
    for op in changes.ops() {
        for change in changes.iter_inline_changes(op) {
            match change.tag() {
                similar::ChangeTag::Insert => {
                    write_change(writer, change, "+", palette.actual, palette.error, palette)?;
                }
                similar::ChangeTag::Delete => {
                    write_change(writer, change, "-", palette.expected, palette.info, palette)?;
                }
                similar::ChangeTag::Equal => {
                    write_change(writer, change, "|", palette.hint, palette.hint, palette)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "diff")]
fn write_change(
    writer: &mut dyn std::fmt::Write,
    change: similar::InlineChange<str>,
    sign: &str,
    em_style: crate::report::Style,
    style: crate::report::Style,
    palette: crate::report::Palette,
) -> Result<(), std::fmt::Error> {
    if let Some(index) = change.old_index() {
        write!(writer, "{:>4} ", palette.hint(index + 1),)?;
    } else {
        write!(writer, "{:>4} ", " ",)?;
    }
    if let Some(index) = change.new_index() {
        write!(writer, "{:>4} ", palette.hint(index + 1),)?;
    } else {
        write!(writer, "{:>4} ", " ",)?;
    }
    write!(writer, "{} ", style.paint(sign))?;
    for &(emphasized, change) in change.values() {
        let cur_style = if emphasized { em_style } else { style };
        write!(writer, "{}", cur_style.paint(change))?;
    }
    if change.missing_newline() {
        writeln!(writer, "{}", em_style.paint("∅"))?;
    }

    Ok(())
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

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1 | Hello
   2    2 | World
";

        assert_eq!(expected_diff, actual_diff);
    }

    #[cfg(feature = "diff")]
    #[test]
    fn diff_ne_line_missing() {
        let expected = "Hello\nWorld\n";
        let expected_name = "A";
        let actual = "Hello\n";
        let actual_name = "B";
        let palette = crate::report::Palette::never();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1 | Hello
   2      - World
";

        assert_eq!(expected_diff, actual_diff);
    }

    #[cfg(feature = "diff")]
    #[test]
    fn diff_eq_trailing_extra_newline() {
        let expected = "Hello\nWorld";
        let expected_name = "A";
        let actual = "Hello\nWorld\n";
        let actual_name = "B";
        let palette = crate::report::Palette::never();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1 | Hello
   2      - World∅
        2 + World
";

        assert_eq!(expected_diff, actual_diff);
    }

    #[cfg(feature = "diff")]
    #[test]
    fn diff_eq_trailing_newline_missing() {
        let expected = "Hello\nWorld\n";
        let expected_name = "A";
        let actual = "Hello\nWorld";
        let actual_name = "B";
        let palette = crate::report::Palette::never();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1 | Hello
   2      - World
        2 + World∅
";

        assert_eq!(expected_diff, actual_diff);
    }
}
