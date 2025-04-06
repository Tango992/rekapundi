pub mod expense;
pub mod query_result;
pub mod util;

use crate::{common::deserializer, constants::MAX_PAGINATION_LIMIT};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    /// The maximum number of expenses to return.
    #[serde(
        deserialize_with = "deserializer::pagination_value_with_fallback",
        default
    )]
    limit: Option<u32>,
    /// The offset for pagination.
    #[serde(
        deserialize_with = "deserializer::pagination_value_with_fallback",
        default
    )]
    offset: Option<u32>,
}

impl Pagination {
    /// Returns the limit for pagination, defaulting to `MAX_PAGINATION_LIMIT` if not set or invalid.
    pub fn limit(&self) -> i64 {
        let raw_limit = self.limit.unwrap_or(MAX_PAGINATION_LIMIT);

        if raw_limit > MAX_PAGINATION_LIMIT {
            return MAX_PAGINATION_LIMIT as i64;
        }

        raw_limit as i64
    }

    /// Returns the offset for pagination, defaulting to `0` if not set or invalid.
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as i64
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
            "limit": 50,
            "offset": 10
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
    fn test_pagination_invalid_limit() {
        let json_str = r#"{
            "limit": 200,
            "offset": -10
        }"#;

        let pagination = serde_json::from_str::<Pagination>(json_str).unwrap();

        assert_eq!(pagination.limit(), MAX_PAGINATION_LIMIT as i64);
        assert_eq!(pagination.offset(), 0);
    }
}
