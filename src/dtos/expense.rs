use crate::common::deserializer;
use serde::{Deserialize, Serialize};
use time::Date;
use validator::Validate;

/// Data transfer object for saving an expense.
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
    #[validate]
    pub expenses: Vec<SaveExpense>,
}
