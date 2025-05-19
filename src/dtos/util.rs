use crate::common::deserializer;
use serde::{Deserialize, Serialize};

use crate::dtos::{
    Pagination,
    query_result::{ParentCategory, SimpleEntity, Tag},
};

/// The response body to list all categories.
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, Deserialize, Eq, PartialEq))]
pub struct IndexCategoriesResponse {
    /// The list of categories.
    pub categories: Vec<SimpleEntity>,
}

/// The response body to list all parent categories.
#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
#[cfg_attr(test, derive(Debug, Deserialize, Eq, PartialEq))]
#[cfg_attr(test, serde(rename_all(deserialize = "camelCase")))]
pub struct IndexParentCategoriesResponse {
    /// The list of parent categories.
    pub parent_categories: Vec<ParentCategory>,
}

/// The query string for filtering tags.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct IndexTagsQuery {
    /// The value to filter tags by their importance.
    #[serde(deserialize_with = "deserializer::bool_with_fallback", default)]
    pub mark_important_value: Option<bool>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

/// The response body to list all tags.
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, Deserialize, Eq, PartialEq))]
pub struct IndexTagsResponse {
    /// The list of tags.
    pub tags: Vec<Tag>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_index_tag_query_happy() {
        let json_str = r#"{
            "markImportantValue": true,
            "offset": "0",
            "limit": "10"
        }"#;

        let query: IndexTagsQuery = serde_json::from_str(json_str).unwrap();
        assert_eq!(query.mark_important_value, Some(true));
        assert_eq!(query.pagination.offset(), 0);
        assert_eq!(query.pagination.limit(), 10);
    }

    #[test]
    fn test_index_tag_query_invalid_mark_important_value() {
        let json_str = r#"{
            "markImportantValue": "invalid",
            "offset": "0",
            "limit": "10"
        }"#;

        let result: IndexTagsQuery = serde_json::from_str(json_str).unwrap();
        assert_eq!(result.mark_important_value, None);
        assert_eq!(result.pagination.offset(), 0);
        assert_eq!(result.pagination.limit(), 10);
    }

    #[test]
    fn test_index_tag_query_missing_mark_important_value() {
        let json_str = r#"{
            "offset": "0",
            "limit": "10"
        }"#;

        let query: IndexTagsQuery = serde_json::from_str(json_str).unwrap();
        assert_eq!(query.mark_important_value, None);
        assert_eq!(query.pagination.offset(), 0);
        assert_eq!(query.pagination.limit(), 10);
    }
}
