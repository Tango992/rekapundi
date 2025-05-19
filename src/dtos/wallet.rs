use crate::{common::deserializer, dtos::query_result::SimpleEntity};
use serde::{self, Deserialize, Serialize};
use time::Date;

/// The request body for saving a money transfer between wallets.
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
#[cfg_attr(test, derive(Debug))]
pub struct SaveMoneyTransferRequest {
    /// The ID of the source wallet.
    #[serde(deserialize_with = "deserializer::positive_int")]
    pub source_wallet_id: i32,
    /// The ID of the target wallet.
    #[serde(deserialize_with = "deserializer::positive_int")]
    pub target_wallet_id: i32,
    /// The amount of the transfer.
    #[serde(deserialize_with = "deserializer::non_negative_int")]
    pub amount: i32,
    /// The fee associated with the transfer.
    #[serde(deserialize_with = "deserializer::non_negative_int")]
    pub fee: i32,
    /// The date of the transfer.
    #[serde(deserialize_with = "deserializer::date")]
    pub date: Date,
    /// Optional description of the transfer.
    pub description: Option<String>,
}

/// DTO used by the repository for saving a money transfer record in the database.
pub struct SaveMoneyTransfer {
    /// The ID of the source wallet.
    pub source_wallet_id: i32,
    /// The ID of the target wallet.
    pub target_wallet_id: i32,
    /// The amount of the transfer.
    pub amount: i32,
    /// The date of the transfer.
    pub date: Date,
    /// Optional description of the transfer.
    pub description: Option<String>,
}

/// DTO used by the repository for saving a money transfer fee record in the database.
/// It will be inserted into the `expense` table.
pub struct SaveMoneyTransferFee {
    /// The priority of the fee. It's defaulted to secondary priority.
    pub priority: i32,
    /// The ID of the source wallet.
    pub wallet_id: i32,
    /// The ID that indicates fee category.
    pub category_id: i32,
    /// The amount of the fee.
    pub amount: i32,
    /// The date of the transfer.
    pub date: Date,
    /// Optional description of the fee.
    pub description: Option<String>,
}

/// The response body to list all wallets.
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, Deserialize, Eq, PartialEq))]
pub struct IndexWalletsResponse {
    /// The list of wallets.
    pub wallets: Vec<SimpleEntity>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_save_transfer_valid() {
        let json_str = r#"{
            "sourceWalletId": 1,
            "targetWalletId": 2,
            "amount": 1000,
            "fee": 10,
            "date": "2025-05-06",
            "description": "Test transfer"
        }"#;

        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_ok());
        let transfer = result.unwrap();
        assert_eq!(transfer.source_wallet_id, 1);
        assert_eq!(transfer.target_wallet_id, 2);
        assert_eq!(transfer.amount, 1000);
        assert_eq!(transfer.fee, 10);
        assert_eq!(transfer.description, Some("Test transfer".to_string()));
    }

    #[test]
    fn test_save_transfer_invalid_source_wallet_id() {
        let json_str = r#"{
            "sourceWalletId": 0,
            "targetWalletId": 2,
            "amount": 1000,
            "fee": 10,
            "date": "2025-05-06",
            "description": null
        }"#;
        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_transfer_invalid_target_wallet_id() {
        let json_str = r#"{
            "sourceWalletId": 1,
            "targetWalletId": -2,
            "amount": 1000,
            "fee": 10,
            "date": "2025-05-06",
            "description": null
        }"#;
        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_transfer_invalid_amount() {
        let json_str = r#"{
            "sourceWalletId": 1,
            "targetWalletId": 2,
            "amount": -1,
            "fee": 10,
            "date": "2025-05-06",
            "description": null
        }"#;
        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_transfer_invalid_fee() {
        let json_str = r#"{
            "sourceWalletId": 1,
            "targetWalletId": 2,
            "amount": 1000,
            "fee": -1,
            "date": "2025-05-06",
            "description": null
        }"#;
        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_transfer_invalid_date() {
        let json_str = r#"{
            "sourceWalletId": 1,
            "targetWalletId": 2,
            "amount": 1000,
            "fee": 10,
            "date": "2025-13-01",
            "description": null
        }"#;
        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_transfer_optional_description() {
        let json_str = r#"{
            "sourceWalletId": 1,
            "targetWalletId": 2,
            "amount": 1000,
            "fee": 10,
            "date": "2025-05-06"
        }"#;
        let result = serde_json::from_str::<SaveMoneyTransferRequest>(json_str);
        assert!(result.is_ok());
        let transfer = result.unwrap();
        assert_eq!(transfer.description, None);
    }
}
