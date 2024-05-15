#![allow(deprecated)]

use super::Data;

pub use crate::filter::Filter as Normalize;

#[deprecated(since = "0.5.11", note = "Replaced with `filter::FilterNewlines")]
pub struct NormalizeNewlines;
impl Normalize for NormalizeNewlines {
    fn normalize(&self, data: Data) -> Data {
        crate::filter::FilterNewlines.normalize(data)
    }
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::FilterPaths")]
pub struct NormalizePaths;
impl Normalize for NormalizePaths {
    fn normalize(&self, data: Data) -> Data {
        crate::filter::FilterPaths.normalize(data)
    }
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::FilterMatches")]
pub type NormalizeMatches<'a> = crate::filter::FilterMatches<'a>;
