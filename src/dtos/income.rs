use crate::common::deserializer;
use crate::dtos::{Pagination, query_result::IndexIncomeElement};
use serde::{Deserialize, Serialize};
use time::Date;

/// Data transfer object for saving an income.
/// Numeric fields are represented with unsigned integers to automatically filter out negative values from the client.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
#[cfg_attr(test, derive(Debug))]
pub struct SaveIncome {
    /// The amount of the income.
    #[serde(deserialize_with = "deserializer::positive_int")]
    pub amount: i32,
    /// The date of the income.
    #[serde(deserialize_with = "deserializer::date")]
    pub date: Date,
    /// Optional description of the income.
    pub description: Option<String>,
    /// The wallet ID where the income is going to.
    #[serde(deserialize_with = "deserializer::positive_int")]
    pub wallet_id: i32,
}

/// Data transfer object for saving a batch of incomes.
#[derive(Deserialize)]
pub struct SaveBatchIncome {
    /// The list of incomes to be saved.
    pub incomes: Vec<SaveIncome>,
}

/// Data transfer object for the response of the index income endpoint.
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, Deserialize, PartialEq, Eq))]
pub struct IndexIncomeResponse {
    /// The list of incomes.
    pub incomes: Vec<IndexIncomeElement>,
}

/// The query string for filtering incomes.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct IndexIncomeQuery {
    /// The lower bound date (inclusive) for filtering incomes.
    #[serde(
        deserialize_with = "deserializer::optional_date",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub start_date: Option<Date>,
    /// The upper bound date (inclusive) for filtering incomes.
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
    fn test_save_income_valid() {
        let json_str = r#"{
            "amount": 1500000,
            "date": "2025-04-01",
            "description": "Salary",
            "walletId": 1
        }"#;

        let save_income: SaveIncome = serde_json::from_str(json_str).unwrap();

        let expected_date = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(save_income.amount, 1500000);
        assert_eq!(save_income.date, expected_date);
        assert_eq!(save_income.description, Some("Salary".to_string()));
        assert_eq!(save_income.wallet_id, 1);
    }

    #[test]
    fn test_save_income_zero_amount() {
        let json_str = r#"{
            "amount": 0,
            "date": "2025-04-01",
            "description": "Salary",
            "walletId": 1
        }"#;

        let result = serde_json::from_str::<SaveIncome>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_income_negative_amount() {
        let json_str = r#"{
            "amount": -500,
            "date": "2025-04-01",
            "description": "Salary",
            "walletId": 1
        }"#;

        let result = serde_json::from_str::<SaveIncome>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_income_zero_wallet_id() {
        let json_str = r#"{
            "amount": 1500000,
            "date": "2025-04-01",
            "description": "Salary",
            "walletId": 0
        }"#;

        let result = serde_json::from_str::<SaveIncome>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_income_negative_wallet_id() {
        let json_str = r#"{
            "amount": 1500000,
            "date": "2025-04-01",
            "description": "Salary",
            "walletId": -2
        }"#;

        let result = serde_json::from_str::<SaveIncome>(json_str);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Value must be positive")
        );
    }

    #[test]
    fn test_save_batch_income_valid() {
        let json_str = r#"{
            "incomes": [
                {
                    "amount": 1500000,
                    "date": "2025-04-01",
                    "description": "Salary",
                    "walletId": 1
                },
                {
                    "amount": 500000,
                    "date": "2025-04-02",
                    "description": "Bonus",
                    "walletId": 2
                }
            ]
        }"#;

        let save_batch_income: SaveBatchIncome = serde_json::from_str(json_str).unwrap();

        assert_eq!(save_batch_income.incomes.len(), 2);
        assert_eq!(save_batch_income.incomes[0].amount, 1500000);
        assert_eq!(save_batch_income.incomes[1].amount, 500000);
    }

    #[test]
    fn test_index_income_query_with_all_fields() {
        let json_str = r#"{
            "startDate": "2025-03-01",
            "endDate": "2025-04-01",
            "limit": "25",
            "offset": "0"
        }"#;

        let query: IndexIncomeQuery = serde_json::from_str(json_str).unwrap();

        let expected_start = Date::from_calendar_date(2025, time::Month::March, 1).unwrap();
        let expected_end = Date::from_calendar_date(2025, time::Month::April, 1).unwrap();

        assert_eq!(query.start_date, Some(expected_start));
        assert_eq!(query.end_date, Some(expected_end));
        assert_eq!(query.pagination.limit(), 25);
        assert_eq!(query.pagination.offset(), 0);
    }

    #[test]
    fn test_index_income_query_with_missing_fields() {
        let json_str = r#"{
        }"#;

        let query: IndexIncomeQuery = serde_json::from_str(json_str).unwrap();

        assert_eq!(query.start_date, None);
        assert_eq!(query.end_date, None);
        assert_eq!(query.pagination.limit(), 100);
        assert_eq!(query.pagination.offset(), 0);
    }
}
