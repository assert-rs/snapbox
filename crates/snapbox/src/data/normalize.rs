#![allow(deprecated)]

use super::Data;

pub use crate::filter::Normalize;

#[deprecated(since = "0.5.11", note = "Replaced with `filter::NormalizeNewlines")]
pub struct NormalizeNewlines;
impl Normalize for NormalizeNewlines {
    fn normalize(&self, data: Data) -> Data {
        crate::filter::NormalizeNewlines.normalize(data)
    }
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::NormalizePaths")]
pub struct NormalizePaths;
impl Normalize for NormalizePaths {
    fn normalize(&self, data: Data) -> Data {
        crate::filter::NormalizePaths.normalize(data)
    }
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::NormalizeMatches")]
pub type NormalizeMatches<'a> = crate::filter::NormalizeMatches<'a>;
