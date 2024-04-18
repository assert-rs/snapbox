use snapbox::assert_data_eq;
use snapbox::file;
use snapbox::str;

#[test]
fn test_trivial_assert() {
    assert_data_eq!("5", str!["5"]);
}

#[test]
fn smoke_test_indent() {
    assert_data_eq!(
        "\
line1
  line2
",
        str![[r#"
line1
  line2
"#]],
    );
}

#[test]
fn test_expect_file() {
    assert_data_eq!(include_str!("../../README.md"), file!["../../README.md"]);
}
