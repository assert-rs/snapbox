pub(crate) fn diff(
    expected: &str,
    actual: &str,
    expected_name: impl std::fmt::Display,
    actual_name: impl std::fmt::Display,
    palette: crate::Palette,
) -> String {
    diff_inner(
        expected,
        actual,
        &palette.info.paint(expected_name).to_string(),
        &palette.error.paint(actual_name).to_string(),
        palette,
    )
}

pub(crate) fn diff_inner(
    expected: &str,
    actual: &str,
    expected_name: &str,
    actual_name: &str,
    palette: crate::Palette,
) -> String {
    let expected: Vec<_> = crate::lines::LinesWithTerminator::new(expected).collect();
    let actual: Vec<_> = crate::lines::LinesWithTerminator::new(actual).collect();
    let diff = difflib::unified_diff(
        &expected,
        &actual,
        expected_name,
        actual_name,
        &palette.info.paint("expected").to_string(),
        &palette.error.paint("actual").to_string(),
        0,
    );
    let mut diff = colorize_diff(diff, palette);
    diff.insert(0, "\n".to_owned());

    diff.join("")
}

#[cfg(feature = "color")]
fn colorize_diff(mut lines: Vec<String>, palette: crate::Palette) -> Vec<String> {
    for (i, line) in lines.iter_mut().enumerate() {
        match (i, line.as_bytes().get(0)) {
            (0, _) => {
                if let Some((prefix, body)) = line.split_once(' ') {
                    *line = format!("{} {}", palette.info.paint(prefix), body);
                }
            }
            (1, _) => {
                if let Some((prefix, body)) = line.split_once(' ') {
                    *line = format!("{} {}", palette.error.paint(prefix), body);
                }
            }
            (_, Some(b'-')) => {
                let (prefix, body) = line.split_at(1);
                *line = format!("{}{}", palette.info.paint(prefix), body);
            }
            (_, Some(b'+')) => {
                let (prefix, body) = line.split_at(1);
                *line = format!("{}{}", palette.error.paint(prefix), body);
            }
            (_, Some(b'@')) => {
                *line = format!("{}", palette.hint.paint(&line));
            }
            _ => (),
        }
    }
    lines
}

#[cfg(not(feature = "color"))]
fn colorize_diff(lines: Vec<String>, _palette: crate::Palette) -> Vec<String> {
    lines
}
