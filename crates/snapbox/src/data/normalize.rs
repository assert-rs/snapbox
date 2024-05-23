#![allow(deprecated)]

use super::Data;

pub use crate::filter::Filter as Normalize;

#[deprecated(since = "0.5.11", note = "Replaced with `filter::FilterNewlines")]
pub struct NormalizeNewlines;
impl Normalize for NormalizeNewlines {
    fn normalize(&self, data: Data) -> Data {
        crate::filter::NormalizeNewlines.normalize(data)
    }
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::FilterPaths")]
pub struct NormalizePaths;
impl Normalize for NormalizePaths {
    fn normalize(&self, data: Data) -> Data {
        crate::filter::NormalizePaths.normalize(data)
    }
}

#[deprecated(since = "0.5.11", note = "Replaced with `filter::FilterRedactions")]
pub type NormalizeMatches<'a> = crate::filter::FilterRedactions<'a>;
