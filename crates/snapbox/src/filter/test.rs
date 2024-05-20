#[cfg(feature = "json")]
use serde_json::json;

#[cfg(feature = "json")]
use super::*;

// Tests for normalization on json
#[test]
#[cfg(feature = "json")]
fn json_normalize_paths_and_lines() {
    let json = json!({"name": "John\\Doe\r\n"});
    let data = Data::json(json);
    let data = FilterPaths.filter(data);
    assert_eq!(Data::json(json!({"name": "John/Doe\r\n"})), data);
    let data = FilterNewlines.filter(data);
    assert_eq!(Data::json(json!({"name": "John/Doe\n"})), data);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_obj_paths_and_lines() {
    let json = json!({
        "person": {
            "name": "John\\Doe\r\n",
            "nickname": "Jo\\hn\r\n",
        }
    });
    let data = Data::json(json);
    let data = FilterPaths.filter(data);
    let assert = json!({
        "person": {
            "name": "John/Doe\r\n",
            "nickname": "Jo/hn\r\n",
        }
    });
    assert_eq!(Data::json(assert), data);
    let data = FilterNewlines.filter(data);
    let assert = json!({
        "person": {
            "name": "John/Doe\n",
            "nickname": "Jo/hn\n",
        }
    });
    assert_eq!(Data::json(assert), data);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_array_paths_and_lines() {
    let json = json!({"people": ["John\\Doe\r\n", "Jo\\hn\r\n"]});
    let data = Data::json(json);
    let data = FilterPaths.filter(data);
    let paths = json!({"people": ["John/Doe\r\n", "Jo/hn\r\n"]});
    assert_eq!(Data::json(paths), data);
    let data = FilterNewlines.filter(data);
    let new_lines = json!({"people": ["John/Doe\n", "Jo/hn\n"]});
    assert_eq!(Data::json(new_lines), data);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_array_obj_paths_and_lines() {
    let json = json!({
        "people": [
            {
                "name": "John\\Doe\r\n",
                "nickname": "Jo\\hn\r\n",
            }
        ]
    });
    let data = Data::json(json);
    let data = FilterPaths.filter(data);
    let paths = json!({
        "people": [
            {
                "name": "John/Doe\r\n",
                "nickname": "Jo/hn\r\n",
            }
        ]
    });
    assert_eq!(Data::json(paths), data);
    let data = FilterNewlines.filter(data);
    let new_lines = json!({
        "people": [
            {
                "name": "John/Doe\n",
                "nickname": "Jo/hn\n",
            }
        ]
    });
    assert_eq!(Data::json(new_lines), data);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_matches_string() {
    let exp = json!({"name": "{...}"});
    let expected = Data::json(exp);
    let actual = json!({"name": "JohnDoe"});
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_matches_array() {
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
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_matches_obj() {
    let exp = json!({"people": "{...}"});
    let expected = Data::json(exp);
    let actual = json!({
        "people": {
            "name": "JohnDoe",
            "nickname": "John",
        }
    });
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_matches_diff_order_array() {
    let exp = json!({
        "people": ["John", "Jane"]
    });
    let expected = Data::json(exp);
    let actual = json!({
        "people": ["Jane", "John"]
    });
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_ne!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_obj_redact_with_disparate_keys() {
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
    let actual = FilterRedactions::new(&sub, &expected).filter(actual);

    let expected_actual = json!({
        "a": "[A]",
        "c": "[C]",
    });
    let expected_actual = Data::json(expected_actual);
    assert_eq!(actual, expected_actual);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_wildcard_object_first() {
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
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_wildcard_array_first() {
    let exp = json!([
        "{...}",
        {
            "name": "three",
            "nickname": "3",
        }
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
            "name": "three",
            "nickname": "3",
        }
    ]);
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_wildcard_array_first_last() {
    let exp = json!([
        "{...}",
        {
            "name": "two",
            "nickname": "2",
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
            "name": "three",
            "nickname": "3",
        },
        {
            "name": "four",
            "nickname": "4",
        }
    ]);
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_wildcard_array_middle_last() {
    let exp = json!([
        {
            "name": "one",
            "nickname": "1",
        },
        "{...}",
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
            "name": "three",
            "nickname": "3",
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
    let actual = FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_eq!(exp, act);
    }
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_wildcard_array_middle_last_early_return() {
    let exp = json!([
        {
            "name": "one",
            "nickname": "1",
        },
        "{...}",
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
    let actual_normalized =
        FilterRedactions::new(&Default::default(), &expected).filter(Data::json(actual.clone()));
    if let DataInner::Json(act) = actual_normalized.inner {
        assert_eq!(act, actual);
    }
}
