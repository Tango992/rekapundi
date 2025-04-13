use crate::common::deserializer;
use serde::Deserialize;
use time::Date;

/// The request body to generate a summary.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
#[cfg_attr(test, derive(Debug))]
pub struct GenerateSummaryRequest {
    /// The start date for the summary.
    #[serde(deserialize_with = "deserializer::date")]
    pub start_date: Date,
    /// The end date for the summary.
    #[serde(deserialize_with = "deserializer::date")]
    pub end_date: Date,
    /// The list of category IDs to exclude from the summary.
    #[serde(deserialize_with = "deserializer::positive_int_vec")]
    pub exclude_category_ids: Vec<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_generate_summary_request_valid() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "excludeCategoryIds": [1, 2, 3]
        }"#;

        let request: GenerateSummaryRequest = serde_json::from_str(json_str).unwrap();

        let expected_start = Date::from_calendar_date(2025, time::Month::March, 1).unwrap();
        let expected_end = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(request.start_date, expected_start);
        assert_eq!(request.end_date, expected_end);
        assert_eq!(request.exclude_category_ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_generate_summary_request_empty_category_ids() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "excludeCategoryIds": []
        }"#;

        let request: GenerateSummaryRequest = serde_json::from_str(json_str).unwrap();

        let expected_start = Date::from_calendar_date(2025, time::Month::March, 1).unwrap();
        let expected_end = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(request.start_date, expected_start);
        assert_eq!(request.end_date, expected_end);
        assert_eq!(request.exclude_category_ids, Vec::<i32>::new());
    }

    #[test]
    fn test_generate_summary_request_zero_category_id() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "excludeCategoryIds": [1, 0, 3]
        }"#;

        let result = serde_json::from_str::<GenerateSummaryRequest>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_generate_summary_request_negative_category_id() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "excludeCategoryIds": [1, -5, 3]
        }"#;

        let result = serde_json::from_str::<GenerateSummaryRequest>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_generate_summary_request_invalid_start_date() {
        let json_str = r#"{
            "startDate": "2025-13-01",
            "endDate": "2025-04-01",
            "excludeCategoryIds": [1, 2, 3]
        }"#;

        let result = serde_json::from_str::<GenerateSummaryRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_summary_request_invalid_end_date() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-32",
            "excludeCategoryIds": [1, 2, 3]
        }"#;

        let result = serde_json::from_str::<GenerateSummaryRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_summary_request_missing_field() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "excludeCategoryIds": [1, 2, 3]
        }"#;

        let result = serde_json::from_str::<GenerateSummaryRequest>(json_str);
        assert!(result.is_err());
    }
}
