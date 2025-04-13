use crate::common::deserializer;
use crate::dtos::{Pagination, query_result::IndexExpenseElement};
use serde::{Deserialize, Serialize};
use time::Date;

/// Data transfer object for saving an expense.
/// Numeric fields are represented with unsigned integers to automatically filter out negative values from the client.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
#[cfg_attr(test, derive(Debug))]
pub struct SaveExpense {
    /// The amount of the expense.
    #[serde(deserialize_with = "deserializer::non_negative_int")]
    pub amount: i32,
    /// The date of the expense.
    #[serde(deserialize_with = "deserializer::date")]
    pub date: Date,
    /// Optional description of the expense.
    pub description: Option<String>,
    /// The priority level of the expense.
    /// 0: high, 1: medium, 2: low
    #[serde(deserialize_with = "deserializer::priority_value")]
    pub priority: i32,
    /// The ID of the category associated with the expense.
    #[serde(deserialize_with = "deserializer::positive_int")]
    pub category_id: i32,
    /// The ID of the wallet associated with the expense.
    #[serde(deserialize_with = "deserializer::positive_int")]
    pub wallet_id: i32,
    /// The IDs of the tags associated with the expense.
    #[serde(deserialize_with = "deserializer::positive_int_vec")]
    pub tag_ids: Vec<i32>,
}

/// Data transfer object for saving a batch of expenses.
#[derive(Deserialize)]
pub struct SaveBatchExpense {
    /// The list of expenses to be saved.
    pub expenses: Vec<SaveExpense>,
}

/// Data transfer object for the response of the index expense endpoint.
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, Deserialize, PartialEq, Eq))]
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
    /// The pagination information for the query.
    #[serde(flatten)]
    pub pagination: Pagination,
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

        let result = serde_json::from_str::<SaveExpense>(json_str);

        assert!(result.is_ok());
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

        let result = serde_json::from_str::<SaveExpense>(json_str);

        assert!(result.is_err());
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

        let result = serde_json::from_str::<SaveExpense>(json_str);

        assert!(result.is_err());
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

        let res = serde_json::from_str::<SaveBatchExpense>(json_str);

        assert!(res.is_ok());
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

        let result = serde_json::from_str::<SaveBatchExpense>(json_str);

        assert!(result.is_err());
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
        assert_eq!(query.pagination.limit(), 10);
        assert_eq!(query.pagination.offset(), 0);
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

        assert_eq!(query.pagination.limit(), 100);
        assert_eq!(query.pagination.offset(), 0);
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

        assert_eq!(query.pagination.limit(), 100);
        assert_eq!(query.pagination.offset(), 0);
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
        assert_eq!(query.pagination.limit(), 100);
        assert_eq!(query.pagination.offset(), 0);
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
        assert_eq!(query.pagination.limit(), 20);
        assert_eq!(query.pagination.offset(), 5);
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
        assert_eq!(query.pagination.limit(), 15);
        assert_eq!(query.pagination.offset(), 10);
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
        assert_eq!(query.pagination.limit(), 25);
        assert_eq!(query.pagination.offset(), 0);
    }

    #[test]
    fn test_save_expense_invalid_amount() {
        let json_str = r#"{
            "amount": -100,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": []
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be non-negative")
        );
    }

    #[test]
    fn test_save_expense_zero_amount() {
        let json_str = r#"{
            "amount": 0,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": []
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_expense_invalid_category_id() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 0,
            "walletId": 1,
            "tagIds": []
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_expense_invalid_wallet_id() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": -5,
            "tagIds": []
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_expense_invalid_tag_ids() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": [1, 2, 0, 4]
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_expense_negative_tag_ids() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": [1, -3, 4]
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_expense_empty_tag_ids() {
        let json_str = r#"{
            "amount": 1000,
            "date": "2025-04-01",
            "description": "Test expense",
            "priority": 1,
            "categoryId": 1,
            "walletId": 1,
            "tagIds": []
        }"#;

        let result = serde_json::from_str::<SaveExpense>(json_str).unwrap();

        assert_eq!(result.tag_ids, Vec::<i32>::new());
    }
}
