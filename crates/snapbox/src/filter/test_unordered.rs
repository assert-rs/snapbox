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
        .unordered()
        .normalize(input.into_data(), &pattern.into_data());
    assert_eq!(actual, expected.into_data());
}

#[test]
#[cfg(feature = "json")]
fn json_normalize_empty() {
    let input = json!([]);
    let pattern = json!([]);
    let expected = json!([]);
    let actual = NormalizeToExpected::new()
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
        .unordered()
        .normalize(Data::json(input), &Data::json(pattern));
    assert_eq!(actual, Data::json(expected));
}
