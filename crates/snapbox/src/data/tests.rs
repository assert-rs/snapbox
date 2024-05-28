#[cfg(feature = "json")]
use serde_json::json;

use super::*;

#[test]
#[cfg(feature = "term-svg")]
fn term_svg_eq() {
    let left = Data::with_inner(DataInner::TermSvg(
        "
irrelevant
  <text>relevant

</text>
irrelevant"
            .to_owned(),
    ));
    let right = Data::with_inner(DataInner::TermSvg(
        "
irrelevant
  <text>relevant

</text>
irrelevant"
            .to_owned(),
    ));
    assert_eq!(left, right);

    let left = Data::with_inner(DataInner::TermSvg(
        "
irrelevant 1
  <text>relevant

</text>
irrelevant 1"
            .to_owned(),
    ));
    let right = Data::with_inner(DataInner::TermSvg(
        "
irrelevant 2
  <text>relevant

</text>
irrelevant 2"
            .to_owned(),
    ));
    assert_eq!(left, right);
}

#[test]
#[cfg(feature = "term-svg")]
fn term_svg_ne() {
    let left = Data::with_inner(DataInner::TermSvg(
        "
irrelevant 1
  <text>relevant 1

</text>
irrelevant 1"
            .to_owned(),
    ));
    let right = Data::with_inner(DataInner::TermSvg(
        "
irrelevant 2
  <text>relevant 2

</text>
irrelevant 2"
            .to_owned(),
    ));
    assert_ne!(left, right);
}

// Tests for checking to_bytes and render produce the same results
#[test]
fn text_to_bytes_render() {
    let d = Data::text(String::from("test"));
    let bytes = d.to_bytes().unwrap();
    let bytes = String::from_utf8(bytes).unwrap();
    let rendered = d.render().unwrap();
    assert_eq!(bytes, rendered);
}

#[test]
#[cfg(feature = "json")]
fn json_to_bytes_render() {
    let d = Data::json(json!({"name": "John\\Doe\r\n"}));
    let bytes = d.to_bytes().unwrap();
    let bytes = String::from_utf8(bytes).unwrap();
    let rendered = d.render().unwrap();
    assert_eq!(bytes, rendered);
}

// Tests for checking all types are coercible to each other and
// for when the coercion should fail
#[test]
fn binary_to_text() {
    let binary = String::from("test").into_bytes();
    let d = Data::binary(binary);
    let text = d.coerce_to(DataFormat::Text);
    assert_eq!(DataFormat::Text, text.format());
}

#[test]
fn binary_to_text_not_utf8() {
    let binary = b"\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00".to_vec();
    let d = Data::binary(binary);
    let d = d.coerce_to(DataFormat::Text);
    assert_ne!(DataFormat::Text, d.format());
    assert_eq!(DataFormat::Binary, d.format());
}

#[test]
#[cfg(feature = "json")]
fn binary_to_json() {
    let value = json!({"name": "John\\Doe\r\n"});
    let binary = serde_json::to_vec_pretty(&value).unwrap();
    let d = Data::binary(binary);
    let json = d.coerce_to(DataFormat::Json);
    assert_eq!(DataFormat::Json, json.format());
}

#[test]
#[cfg(feature = "json")]
fn binary_to_json_not_utf8() {
    let binary = b"\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00".to_vec();
    let d = Data::binary(binary);
    let d = d.coerce_to(DataFormat::Json);
    assert_ne!(DataFormat::Json, d.format());
    assert_eq!(DataFormat::Binary, d.format());
}

#[test]
#[cfg(feature = "json")]
fn binary_to_json_not_json() {
    let binary = String::from("test").into_bytes();
    let d = Data::binary(binary);
    let d = d.coerce_to(DataFormat::Json);
    assert_ne!(DataFormat::Json, d.format());
    assert_eq!(DataFormat::Binary, d.format());
}

#[test]
fn text_to_binary() {
    let text = String::from("test");
    let d = Data::text(text);
    let binary = d.coerce_to(DataFormat::Binary);
    assert_eq!(DataFormat::Binary, binary.format());
}

#[test]
#[cfg(feature = "json")]
fn text_to_json() {
    let value = json!({"name": "John\\Doe\r\n"});
    let text = serde_json::to_string_pretty(&value).unwrap();
    let d = Data::text(text);
    let json = d.coerce_to(DataFormat::Json);
    assert_eq!(DataFormat::Json, json.format());
}

#[test]
#[cfg(feature = "json")]
fn text_to_json_not_json() {
    let text = String::from("test");
    let d = Data::text(text);
    let json = d.coerce_to(DataFormat::Json);
    assert_eq!(DataFormat::Text, json.format());
}

#[test]
#[cfg(feature = "json")]
fn json_to_binary() {
    let value = json!({"name": "John\\Doe\r\n"});
    let d = Data::json(value);
    let binary = d.coerce_to(DataFormat::Binary);
    assert_eq!(DataFormat::Binary, binary.format());
}

#[test]
#[cfg(feature = "json")]
fn json_to_text() {
    let value = json!({"name": "John\\Doe\r\n"});
    let d = Data::json(value);
    let text = d.coerce_to(DataFormat::Text);
    assert_eq!(DataFormat::Text, text.format());
}

// Tests for coercible conversions create the same output as to_bytes/render
//
// render does not need to be checked against bin -> text since render
// outputs None for binary
#[test]
fn text_to_bin_coerce_equals_to_bytes() {
    let text = String::from("test");
    let d = Data::text(text);
    let binary = d.clone().coerce_to(DataFormat::Binary);
    assert_eq!(Data::binary(d.to_bytes().unwrap()), binary);
}

#[test]
#[cfg(feature = "json")]
fn json_to_bin_coerce_equals_to_bytes() {
    let json = json!({"name": "John\\Doe\r\n"});
    let d = Data::json(json);
    let binary = d.clone().coerce_to(DataFormat::Binary);
    assert_eq!(Data::binary(d.to_bytes().unwrap()), binary);
}

#[test]
#[cfg(feature = "json")]
fn json_to_text_coerce_equals_render() {
    let json = json!({"name": "John\\Doe\r\n"});
    let d = Data::json(json);
    let text = d.clone().coerce_to(DataFormat::Text);
    assert_eq!(Data::text(d.render().unwrap()), text);
}

#[cfg(feature = "term-svg")]
mod term_svg_body {
    use super::super::*;

    #[test]
    fn empty() {
        let input = "";
        let expected = None;
        let actual = term_svg_body(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn no_open_tag() {
        let input = "hello
</text>
world!";
        let expected = None;
        let actual = term_svg_body(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn unclosed_open_text() {
        let input = "
Hello
<text
world!";
        let expected = None;
        let actual = term_svg_body(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn capture_one() {
        let input = "
Hello
<text>
world
</text>
Oh";
        let expected = Some(
            "<text>
world
</text>
",
        );
        let actual = term_svg_body(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn no_end_tag() {
        let input = "
Hello
<text>
world";
        let expected = Some(
            "<text>
world",
        );
        let actual = term_svg_body(input);
        assert_eq!(expected, actual);
    }
}
