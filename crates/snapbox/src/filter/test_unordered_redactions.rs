use std::path::PathBuf;

#[cfg(feature = "json")]
use serde_json::json;

use super::*;
use crate::prelude::*;

#[test]
fn str_normalize_empty() {
    let input = "";
    let pattern = "";
    let expected = "";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_same_order() {
    let input = "1
2
3
";
    let pattern = "1
2
3
";
    let expected = "1
2
3
";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_reverse_order() {
    let input = "1
2
3
";
    let pattern = "3
2
1
";
    let expected = "3
2
1
";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_actual_missing() {
    let input = "1
3
";
    let pattern = "1
2
3
";
    let expected = "1
3
";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_expected_missing() {
    let input = "1
2
3
";
    let pattern = "1
3
";
    let expected = "1
3
2
";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_actual_duplicated() {
    let input = "1
2
2
3
";
    let pattern = "1
2
3
";
    let expected = "1
2
3
2
";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_expected_duplicated() {
    let input = "1
2
3
";
    let pattern = "1
2
2
3
";
    let expected = "1
2
3
";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_elide_delimited_with_sub() {
    let input = "Hello World\nHow are you?\nGoodbye World";
    let pattern = "Hello [..]\n...\nGoodbye [..]";
    let expected = "Hello [..]\n...\nGoodbye [..]";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_leading_elide() {
    let input = "Hello\nWorld\nGoodbye";
    let pattern = "...\nGoodbye";
    let expected = "...\nGoodbye";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_trailing_elide() {
    let input = "Hello\nWorld\nGoodbye";
    let pattern = "Hello\n...";
    let expected = "Hello\n...";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_middle_elide() {
    let input = "Hello\nWorld\nGoodbye";
    let pattern = "Hello\n...\nGoodbye";
    let expected = "Hello\n...\nGoodbye";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_post_elide_diverge() {
    let input = "Hello\nSun\nAnd\nWorld";
    let pattern = "Hello\n...\nMoon";
    let expected = "Hello\n...\n";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_post_diverge_elide() {
    let input = "Hello\nWorld\nGoodbye\nSir";
    let pattern = "Hello\nMoon\nGoodbye\n...";
    let expected = "Hello\nGoodbye\n...";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_inline_elide() {
    let input = "Hello\nWorld\nGoodbye\nSir";
    let pattern = "Hello\nW[..]d\nGoodbye\nSir";
    let expected = "Hello\nW[..]d\nGoodbye\nSir";
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, expected.into_data());
}

#[test]
fn str_normalize_user_literal() {
    let input = "Hello world!";
    let pattern = "Hello [OBJECT]!";
    let mut sub = Redactions::new();
    sub.insert("[OBJECT]", "world").unwrap();
    let actual = NormalizeToExpected::new()
        .redact_with(&sub)
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, pattern.into_data());
}

#[test]
fn str_normalize_user_path() {
    let input = "input: /home/epage";
    let pattern = "input: [HOME]";
    let mut sub = Redactions::new();
    let sep = std::path::MAIN_SEPARATOR.to_string();
    let redacted = PathBuf::from(sep).join("home").join("epage");
    sub.insert("[HOME]", redacted).unwrap();
    let actual = NormalizeToExpected::new()
        .redact_with(&sub)
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, pattern.into_data());
}

#[test]
fn str_normalize_user_overlapping_path() {
    let input = "\
a: /home/epage
b: /home/epage/snapbox";
    let pattern = "\
a: [A]
b: [B]";
    let mut sub = Redactions::new();
    let sep = std::path::MAIN_SEPARATOR.to_string();
    let redacted = PathBuf::from(&sep).join("home").join("epage");
    sub.insert("[A]", redacted).unwrap();
    let redacted = PathBuf::from(sep)
        .join("home")
        .join("epage")
        .join("snapbox");
    sub.insert("[B]", redacted).unwrap();
    let actual = NormalizeToExpected::new()
        .redact_with(&sub)
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, pattern.into_data());
}

#[test]
fn str_normalize_user_disabled() {
    let input = "cargo";
    let pattern = "cargo[EXE]";
    let mut sub = Redactions::new();
    sub.insert("[EXE]", "").unwrap();
    let actual = NormalizeToExpected::new()
        .redact_with(&sub)
        .unordered()
        .normalize(input.into(), &pattern.into());
    assert_eq!(actual, pattern.into_data());
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_empty() {
    let input = json!([]);
    let pattern = json!([]);
    let expected = json!([]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_same_order() {
    let input = json!([1, 2, 3]);
    let pattern = json!([1, 2, 3]);
    let expected = json!([1, 2, 3]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_reverse_order() {
    let input = json!([1, 2, 3]);
    let pattern = json!([3, 2, 1]);
    let expected = json!([3, 2, 1]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_actual_missing() {
    let input = json!([1, 3]);
    let pattern = json!([1, 2, 3]);
    let expected = json!([1, 3]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_expected_missing() {
    let input = json!([1, 2, 3]);
    let pattern = json!([1, 3]);
    let expected = json!([1, 3, 2]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_actual_duplicated() {
    let input = json!([1, 2, 2, 3]);
    let pattern = json!([1, 2, 3]);
    let expected = json!([1, 2, 3, 2]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_expected_duplicated() {
    let input = json!([1, 2, 3]);
    let pattern = json!([1, 2, 2, 3]);
    let expected = json!([1, 2, 3]);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_glob_for_string() {
    let exp = json!({"name": "{...}"});
    let expected = Data::json(exp);
    let actual = json!({"name": "JohnDoe"});
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(actual), &expected);
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_glob_for_array() {
    let exp = json!({"people": "{...}"});
    let expected = Data::json(exp);
    let actual = json!({
        "people": [
            {
                "name": "JohnDoe",
                "nickname": "John",
            }
        ]
    });
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(actual), &expected);
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_glob_for_obj() {
    let exp = json!({"people": "{...}"});
    let expected = Data::json(exp);
    let actual = json!({
        "people": {
            "name": "JohnDoe",
            "nickname": "John",
        }
    });
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(actual), &expected);
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_glob_array_start() {
    let exp = json!({
        "people": [
            "{...}",
            {
                "name": "three",
                "nickname": "3",
            }
        ]
    });
    let expected = Data::json(exp);
    let actual = json!({
        "people": [
            {
                "name": "one",
                "nickname": "1",
            },
            {
                "name": "two",
                "nickname": "2",
            },
            {
                "name": "three",
                "nickname": "3",
            }
        ]
    });
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(actual), &expected);
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_glob_for_array_mismatch() {
    let exp = json!([
        {
            "name": "one",
            "nickname": "1",
        },
        {
            "name": "three",
            "nickname": "3",
        },
        "{...}"
    ]);
    let expected = Data::json(exp);
    let actual = json!([
        {
            "name": "one",
            "nickname": "1",
        },
        {
            "name": "two",
            "nickname": "2",
        },
        {
            "name": "four",
            "nickname": "4",
        },
        {
            "name": "five",
            "nickname": "5",
        }
    ]);
    let expected_actual = json!([
        {
            "name": "one",
            "nickname": "1",
        },
        "{...}"
    ]);
    let actual_normalized = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(Data::json(actual.clone()), &expected);
    if let DataInner::Json(act) = actual_normalized.inner {
        assert_eq!(act, expected_actual);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_obj_key() {
    let expected = json!({
        "[A]": "value-a",
        "[B]": "value-b",
        "[C]": "value-c",
    });
    let expected = Data::json(expected);
    let actual = json!({
        "key-a": "value-a",
        "key-b": "value-b",
        "key-c": "value-c",
    });
    let actual = Data::json(actual);
    let mut sub = Redactions::new();
    sub.insert("[A]", "key-a").unwrap();
    sub.insert("[B]", "key-b").unwrap();
    sub.insert("[C]", "key-c").unwrap();
    let actual = NormalizeToExpected::new()
        .redact_with(&sub)
        .unordered()
        .normalize(actual, &expected);

    let expected_actual = json!({
        "[A]": "value-a",
        "[B]": "value-b",
        "[C]": "value-c",
    });
    let expected_actual = Data::json(expected_actual);
    assert_eq!(actual, expected_actual);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_with_missing_obj_key() {
    let expected = json!({
        "a": "[A]",
        "b": "[B]",
        "c": "[C]",
    });
    let expected = Data::json(expected);
    let actual = json!({
        "a": "value-a",
        "c": "value-c",
    });
    let actual = Data::json(actual);
    let mut sub = Redactions::new();
    sub.insert("[A]", "value-a").unwrap();
    sub.insert("[B]", "value-b").unwrap();
    sub.insert("[C]", "value-c").unwrap();
    let actual = NormalizeToExpected::new()
        .redact_with(&sub)
        .unordered()
        .normalize(actual, &expected);

    let expected_actual = json!({
        "a": "[A]",
        "c": "[C]",
    });
    let expected_actual = Data::json(expected_actual);
    assert_eq!(actual, expected_actual);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_glob_obj_key() {
    let expected = json!({
        "a": "value-a",
        "c": "value-c",
        "...": "{...}",
    });
    let expected = Data::json(expected);
    let actual = json!({
        "a": "value-a",
        "b": "value-b",
        "c": "value-c",
    });
    let actual = Data::json(actual);
    let actual = NormalizeToExpected::new()
        .redact()
        .unordered()
        .normalize(actual, &expected);

    let expected_actual = json!({
        "a": "value-a",
        "c": "value-c",
        "...": "{...}",
    });
    let expected_actual = Data::json(expected_actual);
    assert_eq!(actual, expected_actual);
}
