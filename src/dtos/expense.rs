use crate::common::deserializer;
use serde::{Deserialize, Serialize};
use time::Date;
use validator::Validate;

/// Data transfer object for saving an expense.
/// Numeric fields are represented with unsigned integers to automatically filter out negative values from the client.
#[derive(Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SaveExpense {
    /// The amount of the expense.
    pub amount: u32,
    /// The date of the expense.
    #[serde(deserialize_with = "deserializer::raw_to_date")]
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
#[derive(Clone, Deserialize, Serialize, Validate)]
pub struct SaveBatchExpense {
    /// The list of expenses to be saved.
    #[validate(nested)]
    pub expenses: Vec<SaveExpense>,
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
}
