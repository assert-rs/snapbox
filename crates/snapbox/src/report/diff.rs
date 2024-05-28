use crate::report::Styled;

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
    if let (Some(expected_relevant), Some(actual_relevant)) =
        (expected.relevant(), actual.relevant())
    {
        let expected_rendered = expected.render().unwrap();
        let expected_line_offset = expected_rendered[..expected_rendered
            .find(expected_relevant)
            .unwrap_or(expected_rendered.len())]
            .lines()
            .count();
        let actual_rendered = actual.render().unwrap();
        let actual_line_offset = actual_rendered[..actual_rendered
            .find(actual_relevant)
            .unwrap_or(actual_rendered.len())]
            .lines()
            .count();
        write_diff_inner(
            writer,
            expected_relevant,
            actual_relevant,
            expected_name,
            actual_name,
            palette,
            expected_line_offset,
            actual_line_offset,
        )?;
        rendered = true;
    } else if let (Some(expected), Some(actual)) = (expected.render(), actual.render()) {
        let expected_line_offset = 0;
        let actual_line_offset = 0;
        write_diff_inner(
            writer,
            &expected,
            &actual,
            expected_name,
            actual_name,
            palette,
            expected_line_offset,
            actual_line_offset,
        )?;
        rendered = true;
    }

    if !rendered {
        if let Some(expected_name) = expected_name {
            writeln!(writer, "{} {}:", expected_name, palette.error("(expected)"))?;
        } else {
            writeln!(writer, "{}:", palette.error("Expected"))?;
        }
        writeln!(writer, "{}", palette.error(&expected))?;
        if let Some(actual_name) = actual_name {
            writeln!(writer, "{} {}:", actual_name, palette.info("(actual)"))?;
        } else {
            writeln!(writer, "{}:", palette.info("Actual"))?;
        }
        writeln!(writer, "{}", palette.info(&actual))?;
    }
    Ok(())
}

#[cfg(feature = "diff")]
#[allow(clippy::too_many_arguments)]
fn write_diff_inner(
    writer: &mut dyn std::fmt::Write,
    expected: &str,
    actual: &str,
    expected_name: Option<&dyn std::fmt::Display>,
    actual_name: Option<&dyn std::fmt::Display>,
    palette: crate::report::Palette,
    expected_line_offset: usize,
    actual_line_offset: usize,
) -> Result<(), std::fmt::Error> {
    let timeout = std::time::Duration::from_millis(500);
    let min_elide = 20;
    let context = 5;

    let changes = similar::TextDiff::configure()
        .algorithm(similar::Algorithm::Patience)
        .timeout(timeout)
        .newline_terminated(false)
        .diff_lines(expected, actual);

    writeln!(writer)?;
    if let Some(expected_name) = expected_name {
        writeln!(
            writer,
            "{}",
            palette.error(format_args!("{:->4} expected: {}", "", expected_name))
        )?;
    } else {
        writeln!(writer, "{}", palette.error(format_args!("--- Expected")))?;
    }
    if let Some(actual_name) = actual_name {
        writeln!(
            writer,
            "{}",
            palette.info(format_args!("{:+>4} actual:   {}", "", actual_name))
        )?;
    } else {
        writeln!(writer, "{}", palette.info(format_args!("+++ Actual")))?;
    }
    let changes = changes
        .ops()
        .iter()
        .flat_map(|op| changes.iter_inline_changes(op))
        .collect::<Vec<_>>();
    let tombstones = if min_elide < changes.len() {
        let mut tombstones = vec![true; changes.len()];

        let mut counter = context;
        for (i, change) in changes.iter().enumerate() {
            match change.tag() {
                similar::ChangeTag::Insert | similar::ChangeTag::Delete => {
                    counter = context;
                    tombstones[i] = false;
                }
                similar::ChangeTag::Equal => {
                    if counter != 0 {
                        tombstones[i] = false;
                        counter -= 1;
                    }
                }
            }
        }

        let mut counter = context;
        for (i, change) in changes.iter().enumerate().rev() {
            match change.tag() {
                similar::ChangeTag::Insert | similar::ChangeTag::Delete => {
                    counter = context;
                    tombstones[i] = false;
                }
                similar::ChangeTag::Equal => {
                    if counter != 0 {
                        tombstones[i] = false;
                        counter -= 1;
                    }
                }
            }
        }
        tombstones
    } else {
        Vec::new()
    };

    let mut elided = false;
    for (i, change) in changes.into_iter().enumerate() {
        if tombstones.get(i).copied().unwrap_or(false) {
            if !elided {
                let sign = "⋮";

                write!(writer, "{:>4} ", " ",)?;
                write!(writer, "{:>4} ", " ",)?;
                writeln!(writer, "{}", palette.hint(sign))?;
            }
            elided = true;
        } else {
            elided = false;
            match change.tag() {
                similar::ChangeTag::Insert => {
                    write_change(
                        writer,
                        change,
                        "+",
                        palette.actual,
                        palette.info,
                        palette,
                        expected_line_offset,
                        actual_line_offset,
                    )?;
                }
                similar::ChangeTag::Delete => {
                    write_change(
                        writer,
                        change,
                        "-",
                        palette.expected,
                        palette.error,
                        palette,
                        expected_line_offset,
                        actual_line_offset,
                    )?;
                }
                similar::ChangeTag::Equal => {
                    write_change(
                        writer,
                        change,
                        "|",
                        palette.hint,
                        palette.hint,
                        palette,
                        expected_line_offset,
                        actual_line_offset,
                    )?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "diff")]
#[allow(clippy::too_many_arguments)]
fn write_change(
    writer: &mut dyn std::fmt::Write,
    change: similar::InlineChange<'_, str>,
    sign: &str,
    em_style: crate::report::Style,
    style: crate::report::Style,
    palette: crate::report::Palette,
    expected_line_offset: usize,
    actual_line_offset: usize,
) -> Result<(), std::fmt::Error> {
    if let Some(index) = change.old_index() {
        write!(
            writer,
            "{:>4} ",
            palette.hint(index + 1 + expected_line_offset),
        )?;
    } else {
        write!(writer, "{:>4} ", " ",)?;
    }
    if let Some(index) = change.new_index() {
        write!(
            writer,
            "{:>4} ",
            palette.hint(index + 1 + actual_line_offset),
        )?;
    } else {
        write!(writer, "{:>4} ", " ",)?;
    }
    write!(writer, "{} ", Styled::new(sign, style))?;
    for &(emphasized, change) in change.values() {
        let cur_style = if emphasized { em_style } else { style };
        write!(writer, "{}", Styled::new(change, cur_style))?;
    }
    if change.missing_newline() {
        writeln!(writer, "{}", Styled::new("∅", em_style))?;
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
        let palette = crate::report::Palette::plain();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
            0,
            0,
        )
        .unwrap();
        let expected_diff = "
---- expected: A
++++ actual:   B
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
        let palette = crate::report::Palette::plain();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
            0,
            0,
        )
        .unwrap();
        let expected_diff = "
---- expected: A
++++ actual:   B
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
        let palette = crate::report::Palette::plain();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
            0,
            0,
        )
        .unwrap();
        let expected_diff = "
---- expected: A
++++ actual:   B
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
        let palette = crate::report::Palette::plain();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            expected,
            actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
            0,
            0,
        )
        .unwrap();
        let expected_diff = "
---- expected: A
++++ actual:   B
   1    1 | Hello
   2      - World
        2 + World∅
";

        assert_eq!(expected_diff, actual_diff);
    }

    #[cfg(feature = "diff")]
    #[test]
    fn diff_eq_elided() {
        let mut expected = String::new();
        expected.push_str("Hello\n");
        for i in 0..20 {
            expected.push_str(&i.to_string());
            expected.push('\n');
        }
        expected.push_str("World\n");
        for i in 0..20 {
            expected.push_str(&i.to_string());
            expected.push('\n');
        }
        expected.push_str("!\n");
        let expected_name = "A";

        let mut actual = String::new();
        actual.push_str("Goodbye\n");
        for i in 0..20 {
            actual.push_str(&i.to_string());
            actual.push('\n');
        }
        actual.push_str("Moon\n");
        for i in 0..20 {
            actual.push_str(&i.to_string());
            actual.push('\n');
        }
        actual.push_str("?\n");
        let actual_name = "B";

        let palette = crate::report::Palette::plain();

        let mut actual_diff = String::new();
        write_diff_inner(
            &mut actual_diff,
            &expected,
            &actual,
            Some(&expected_name),
            Some(&actual_name),
            palette,
            0,
            0,
        )
        .unwrap();
        let expected_diff = "
---- expected: A
++++ actual:   B
   1      - Hello
        1 + Goodbye
   2    2 | 0
   3    3 | 1
   4    4 | 2
   5    5 | 3
   6    6 | 4
          ⋮
  17   17 | 15
  18   18 | 16
  19   19 | 17
  20   20 | 18
  21   21 | 19
  22      - World
       22 + Moon
  23   23 | 0
  24   24 | 1
  25   25 | 2
  26   26 | 3
  27   27 | 4
          ⋮
  38   38 | 15
  39   39 | 16
  40   40 | 17
  41   41 | 18
  42   42 | 19
  43      - !
       43 + ?
";

        assert_eq!(expected_diff, actual_diff);
    }

    #[cfg(feature = "diff")]
    #[cfg(feature = "term-svg")]
    #[test]
    fn diff_ne_ignore_irrelevant_details() {
        let expected = "<svg width='100px' height='200px'>
<text>
Hello Moon
</text>
</svg>";
        let expected_name = "A";
        let actual = "<svg width='200px' height='400px'>
<text>
Hello World
</text>
</svg>";
        let actual_name = "B";
        let palette = crate::report::Palette::plain();

        let mut actual_diff = String::new();
        write_diff(
            &mut actual_diff,
            &crate::Data::with_inner(crate::data::DataInner::TermSvg(expected.to_owned())),
            &crate::Data::with_inner(crate::data::DataInner::TermSvg(actual.to_owned())),
            Some(&expected_name),
            Some(&actual_name),
            palette,
        )
        .unwrap();
        let expected_diff = "
---- expected: A
++++ actual:   B
   2    2 | <text>
   3      - Hello Moon
        3 + Hello World
   4    4 | </text>
";

        assert_eq!(expected_diff, actual_diff);
    }
}
