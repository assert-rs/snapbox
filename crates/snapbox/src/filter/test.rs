#[cfg(feature = "json")]
use serde_json::json;

#[cfg(feature = "json")]
use super::*;

// Tests for normalization on json
#[test]
#[cfg(feature = "json")]
fn json_normalize_paths_and_lines_string() {
    let json = json!({"name": "John\\Doe\r\n"});
    let data = Data::json(json);
    let data = FilterPaths.filter(data);
    assert_eq!(Data::json(json!({"name": "John/Doe\r\n"})), data);
    let data = FilterNewlines.filter(data);
    assert_eq!(Data::json(json!({"name": "John/Doe\n"})), data);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_paths_and_lines_nested_string() {
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
fn json_normalize_paths_and_lines_obj_key() {
    let json = json!({
        "person": {
            "John\\Doe\r\n": "name",
            "Jo\\hn\r\n": "nickname",
        }
    });
    let data = Data::json(json);
    let data = FilterPaths.filter(data);
    let assert = json!({
        "person": {
            "John/Doe\r\n": "name",
            "Jo/hn\r\n": "nickname",
        }
    });
    assert_eq!(Data::json(assert), data);
    let data = FilterNewlines.filter(data);
    let assert = json!({
        "person": {
            "John/Doe\n": "name",
            "Jo/hn\n": "nickname",
        }
    });
    assert_eq!(Data::json(assert), data);
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_paths_and_lines_array() {
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
fn json_normalize_paths_and_lines_array_obj() {
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
