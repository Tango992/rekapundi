use crate::dtos::query_result::IndexExpenseElement;
use crate::{common::deserializer, constants::MAX_PAGINATION_LIMIT};
use serde::{Deserialize, Serialize};
use time::Date;
use validator::Validate;

/// Data transfer object for saving an expense.
/// Numeric fields are represented with unsigned integers to automatically filter out negative values from the client.
#[derive(Deserialize, Validate)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SaveExpense {
    /// The amount of the expense.
    pub amount: u32,
    /// The date of the expense.
    #[serde(deserialize_with = "deserializer::date")]
    pub date: Date,
    /// Optional description of the expense.
    pub description: Option<String>,
    /// The priority level of the expense.
    /// 0: high, 1: medium, 2: low
    #[validate(range(min = 0, max = 2, message = "Priority must be between 0 and 2"))]
    pub priority: u8,
    /// The ID of the category associated with the expense.
    pub category_id: u32,
    /// The ID of the wallet associated with the expense.
    pub wallet_id: u32,
    /// The IDs of the tags associated with the expense.
    pub tag_ids: Vec<u32>,
}

/// Data transfer object for saving a batch of expenses.
#[derive(Deserialize, Validate)]
pub struct SaveBatchExpense {
    /// The list of expenses to be saved.
    #[validate(nested)]
    pub expenses: Vec<SaveExpense>,
}

/// Data transfer object for the response of the index expense endpoint.
#[derive(Serialize)]
pub struct IndexExpenseResponse {
    /// The list of expenses.
    pub expenses: Vec<IndexExpenseElement>,
}

/// The query string for filtering expenses.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct IndexExpenseQuery {
    /// The lower bound date (inclusive) for filtering expenses.
    #[serde(
        deserialize_with = "deserializer::optional_date",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub start_date: Option<Date>,
    /// The upper bound date (inclusive) for filtering expenses.
    #[serde(
        deserialize_with = "deserializer::optional_date",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub end_date: Option<Date>,
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

impl IndexExpenseQuery {
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

impl Default for IndexExpenseQuery {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            limit: Some(MAX_PAGINATION_LIMIT),
            offset: Some(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_save_expense_valid() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": []
        }"#;

        let expense: SaveExpense = serde_json::from_str(json_str).unwrap();

        let validation_result = expense.validate();
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_save_expense_invalid_priority() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 3,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": [1, 2, 3]
        }"#;

        let expense: SaveExpense = serde_json::from_str(json_str).unwrap();

        let validation_result = expense.validate();
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_save_expense_invalid_date() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-12-32",
            "description": "Test expense",
            "priority": 2,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": [1, 2, 3]
        }"#;

        let expense = serde_json::from_str::<SaveExpense>(json_str);
        assert!(expense.is_err());
    }

    #[test]
    fn test_save_batch_expense_valid() {
        let json_str = r#"{
            "expenses": [
                {
                    "amount": 1000,
                    "date": "2025-04-01",
                    "description": "Test expense 1",
                    "priority": 0,
                    "categoryId": 1,
                    "walletId": 1,
                    "tagIds": [1, 2]
                },
                {
                    "amount": 2000,
                    "date": "2025-04-02",
                    "description": null,
                    "priority": 2,
                    "categoryId": 2,
                    "walletId": 1,
                    "tagIds": []
                }
            ]
        }"#;

        let batch: SaveBatchExpense = serde_json::from_str(json_str).unwrap();

        let validation_result = batch.validate();
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_save_batch_expense_with_invalid_expense() {
        let json_str = r#"{
            "expenses": [
                {
                    "amount": 1000,
                    "date": "2025-04-01",
                    "description": "Test expense 1",
                    "priority": 0,
                    "categoryId": 1,
                    "walletId": 1,
                    "tagIds": [1, 2]
                },
                {
                    "amount": 2000,
                    "date": "2025-04-02",
                    "description": null,
                    "priority": 5,
                    "categoryId": 2,
                    "walletId": 1,
                    "tagIds": []
                }
            ]
        }"#;

        let batch: SaveBatchExpense = serde_json::from_str(json_str).unwrap();

        let validation_result = batch.validate();
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_index_expense_query_with_all_fields() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "limit": 10,
            "offset": 0
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        let expected_start = Date::from_calendar_date(2025, time::Month::March, 1).unwrap();
        let expected_end = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(query.start_date, Some(expected_start));
        assert_eq!(query.end_date, Some(expected_end));
        assert_eq!(query.limit(), 10);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn test_index_expense_query_with_missing_pagination_fields() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01"
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        let expected_start = Date::from_calendar_date(2025, time::Month::March, 1).unwrap();
        let expected_end = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(query.start_date, Some(expected_start));
        assert_eq!(query.end_date, Some(expected_end));

        assert_eq!(query.limit(), 100);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn test_index_expense_query_with_null_pagination_fields() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "limit": null,
            "offset": null
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        let expected_start = Date::from_calendar_date(2025, time::Month::March, 1).unwrap();
        let expected_end = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(query.start_date, Some(expected_start));
        assert_eq!(query.end_date, Some(expected_end));

        assert_eq!(query.limit(), 100);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn test_index_expense_query_with_invalid_pagination() {
        let json_str = r#"{
            "limit": 101,
            "offset": -1
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        assert_eq!(query.start_date, None);
        assert_eq!(query.end_date, None);
        assert_eq!(query.limit(), 100);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn test_index_expense_query_with_missing_optional_fields() {
        let json_str = r#"{
            "limit": 20,
            "offset": 5
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        assert_eq!(query.start_date, None);
        assert_eq!(query.end_date, None);
        assert_eq!(query.limit(), 20);
        assert_eq!(query.offset(), 5);
    }

    #[test]
    fn test_index_expense_query_with_null_dates() {
        let json_str = r#"{
            "startDate": null,
            "endDate": null,
            "limit": 15,
            "offset": 10
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        assert_eq!(query.start_date, None);
        assert_eq!(query.end_date, None);
        assert_eq!(query.limit(), 15);
        assert_eq!(query.offset(), 10);
    }

    #[test]
    fn test_index_expense_query_with_invalid_date_format() {
        let json_str = r#"{
            "startDate": "2025/03/01",
            "endDate": "2025-04-32",
            "limit": 25,
            "offset": 0
        }"#;

        let query: IndexExpenseQuery = serde_json::from_str(json_str).unwrap();

        assert_eq!(query.start_date, None);
        assert_eq!(query.end_date, None);
        assert_eq!(query.limit(), 25);
        assert_eq!(query.offset(), 0);
    }
}
