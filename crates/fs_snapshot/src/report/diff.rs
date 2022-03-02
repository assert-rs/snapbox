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
        writeln!(writer, "{} {}:", expected_name, palette.info("(expected)"))?;
        writeln!(writer, "{}", palette.info(&expected))?;
        writeln!(writer, "{} {}:", actual_name, palette.error("(actual)"))?;
        writeln!(writer, "{}", palette.error(&actual))?;
    }
    Ok(())
}

#[cfg(feature = "diff")]
fn write_diff_inner(
    writer: &mut dyn std::fmt::Write,
    expected: &str,
    actual: &str,
    expected_name: &dyn std::fmt::Display,
    actual_name: &dyn std::fmt::Display,
    palette: crate::report::Palette,
) -> Result<(), std::fmt::Error> {
    let changes = dissimilar::diff(expected, actual);
    writeln!(writer)?;
    writeln!(
        writer,
        "{}",
        palette.info(format_args!("--- {} (expected)", expected_name))
    )?;
    writeln!(
        writer,
        "{}",
        palette.error(format_args!("+++ {} (actual)", actual_name))
    )?;
    let mut expected_line_num = 1;
    let mut actual_line_num = 1;
    for change in changes {
        match change {
            dissimilar::Chunk::Equal(body) => {
                for line in body.lines() {
                    writeln!(
                        writer,
                        "{:>4} {:>4} {} {}",
                        palette.hint(expected_line_num),
                        palette.hint(actual_line_num),
                        palette.hint(' '),
                        palette.hint(line),
                    )?;
                    expected_line_num += 1;
                    actual_line_num += 1;
                }
            }
            dissimilar::Chunk::Delete(body) => {
                for line in body.lines() {
                    writeln!(
                        writer,
                        "{:>4} {:>4} {} {}",
                        palette.hint(expected_line_num),
                        "",
                        palette.info('-'),
                        palette.info(line),
                    )?;
                    expected_line_num += 1;
                }
            }
            dissimilar::Chunk::Insert(body) => {
                for line in body.lines() {
                    writeln!(
                        writer,
                        "{:>4} {:>4} {} {}",
                        "",
                        palette.hint(actual_line_num),
                        palette.error('+'),
                        palette.error(line),
                    )?;
                    actual_line_num += 1;
                }
            }
        }
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
            &expected_name,
            &actual_name,
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1   Hello
   2    2   World
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

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            &expected_name,
            &actual_name,
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1   Hello
   2      - World
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

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            &expected_name,
            &actual_name,
            palette,
        )
        .unwrap();
        let expected_diff = "
--- A (expected)
+++ B (actual)
   1    1   Hello
   2    2   World
   3      - 
";

        assert_eq!(actual_diff, expected_diff);
    }
}
