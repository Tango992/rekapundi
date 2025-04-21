pub mod expense;
pub mod income;
pub mod query_result;
pub mod summary;
pub mod util;

use crate::{common::deserializer, constants::MAX_PAGINATION_LIMIT};
use serde::Deserialize;

/// This struct should only be used for pagination extracted from the query string,
/// since it implements a custom deserializer that coerce string values to `i32`.
/// While the `qs` crate automatically coerces string values to `i32`,
/// for whatever reason it doesn't work when combined with `#[serde(flatten)]`.
/// https://github.com/nox/serde_urlencoded/issues/33
/// https://github.com/serde-rs/serde/issues/1183
#[derive(Deserialize)]
pub struct Pagination {
    /// The maximum number of elements to return.
    #[serde(deserialize_with = "deserializer::from_str", default)]
    limit: Option<i32>,
    /// The offset to start returning elements from.
    #[serde(deserialize_with = "deserializer::from_str", default)]
    offset: Option<i32>,
}

impl Pagination {
    /// Returns the limit for pagination, defaulting to `MAX_PAGINATION_LIMIT` if not set or invalid.
    pub fn limit(&self) -> i64 {
        let raw_limit = self.limit.unwrap_or(MAX_PAGINATION_LIMIT);

        if raw_limit < 0 || raw_limit > MAX_PAGINATION_LIMIT {
            return MAX_PAGINATION_LIMIT.into();
        }

        raw_limit.into()
    }

    /// Returns the offset for pagination, defaulting to `0` if not set or invalid.
    pub fn offset(&self) -> i64 {
        let raw_offset = self.offset.unwrap_or(0);

        if raw_offset < 0 {
            return 0;
        }

        raw_offset.into()
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: Some(MAX_PAGINATION_LIMIT),
            offset: Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_happy_path() {
        let json_str = r#"{
            "limit": "50",
            "offset": "10"
        }"#;

        let pagination = serde_json::from_str::<Pagination>(json_str).unwrap();

        assert_eq!(pagination.limit(), 50);
        assert_eq!(pagination.offset(), 10);
    }

    #[test]
    fn test_pagination_default() {
        let json_str = r#"{
        }"#;

        let pagination = serde_json::from_str::<Pagination>(json_str).unwrap();

        assert_eq!(pagination.limit(), MAX_PAGINATION_LIMIT as i64);
        assert_eq!(pagination.offset(), 0);
    }

    #[test]
    fn test_pagination_negative_limit_and_offset() {
        let json_str = r#"{
            "limit": "-5",
            "offset": "-3"
        }"#;

        let pagination = serde_json::from_str::<Pagination>(json_str).unwrap();

        assert_eq!(pagination.limit(), MAX_PAGINATION_LIMIT as i64);
        assert_eq!(pagination.offset(), 0);
    }

    #[test]
    fn test_pagination_limit_above_max() {
        let json_str = r#"{
            "limit": "110",
            "offset": "110"
        }"#;

        let pagination = serde_json::from_str::<Pagination>(&json_str).unwrap();

        assert_eq!(pagination.limit(), MAX_PAGINATION_LIMIT as i64);
        assert_eq!(pagination.offset(), 110);
    }
}
