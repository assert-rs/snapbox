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
    let data = NormalizePaths.normalize(data);
    assert_eq!(Data::json(json!({"name": "John/Doe\r\n"})), data);
    let data = NormalizeNewlines.normalize(data);
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
    let data = NormalizePaths.normalize(data);
    let assert = json!({
        "person": {
            "name": "John/Doe\r\n",
            "nickname": "Jo/hn\r\n",
        }
    });
    assert_eq!(Data::json(assert), data);
    let data = NormalizeNewlines.normalize(data);
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
    let data = NormalizePaths.normalize(data);
    let paths = json!({"people": ["John/Doe\r\n", "Jo/hn\r\n"]});
    assert_eq!(Data::json(paths), data);
    let data = NormalizeNewlines.normalize(data);
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
    let data = NormalizePaths.normalize(data);
    let paths = json!({
        "people": [
            {
                "name": "John/Doe\r\n",
                "nickname": "Jo/hn\r\n",
            }
        ]
    });
    assert_eq!(Data::json(paths), data);
    let data = NormalizeNewlines.normalize(data);
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
    if let (DataInner::Json(exp), DataInner::Json(act)) = (expected.inner, actual.inner) {
        assert_ne!(exp, act);
    }
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
    let actual =
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual));
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
        NormalizeMatches::new(&Default::default(), &expected).normalize(Data::json(actual.clone()));
    if let DataInner::Json(act) = actual_normalized.inner {
        assert_eq!(act, actual);
    }
}
