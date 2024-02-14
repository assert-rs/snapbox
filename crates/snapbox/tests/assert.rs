use snapbox::assert_eq;
use snapbox::file;
use snapbox::str;

#[test]
fn test_trivial_assert() {
    assert_eq(str!["5"], "5");
}

#[test]
fn smoke_test_indent() {
    assert_eq(
        str![[r#"
            line1
              line2
        "#]]
        .indent(true),
        "\
line1
  line2
",
    );

    assert_eq(
        str![[r#"
line1
  line2
"#]]
        .indent(false),
        "\
line1
  line2
",
    );
}

#[test]
fn test_expect_file() {
    assert_eq(file!["../README.md"], include_str!("../README.md"))
}
