use serde::{Deserialize, Serialize};

/// Reusable struct for entities with an ID and name.
#[derive(Deserialize, Serialize)]
pub struct SimpleEntity {
    /// The ID of the entity.
    pub id: i64,
    /// The name of the entity.
    pub name: String,
}

/// Represents a record of `tag` table in the database.
#[derive(Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Tag {
    /// The ID of the tag.
    pub id: i64,
    /// The name of the tag.
    pub name: String,
    /// Whether the tag has precedence over other tags.
    pub is_important: bool,
}

/// Represents a record of `parent_category` table and its children in the database.
#[derive(sqlx::FromRow, Serialize)]
pub struct ParentCategory {
    /// The ID of the parent category.
    pub id: i64,
    /// The name of the parent category.
    pub name: String,
    /// The list of child categories.
    pub categories: sqlx::types::Json<Vec<SimpleEntity>>,
}

/// Data transfer object for showing the latest expense.
#[derive(Deserialize, Serialize)]
pub struct ShowLatestExpense {
    /// The identifier of the expense.
    pub id: i32,
    /// The amount of the expense.
    pub amount: i32,
    /// The date of the expense.
    pub date: String,
    /// Optional description of the expense.
    pub description: Option<String>,
    /// The priority level of the expense.
    /// 0: high, 1: medium, 2: low
    pub priority: i16,
    /// The category associated with the expense.
    pub category: sqlx::types::Json<SimpleEntity>,
    /// The tags associated with the expense.
    pub tags: sqlx::types::Json<Vec<Tag>>,
    /// The wallet associated with the expense.
    pub wallet: sqlx::types::Json<SimpleEntity>,
}

/// Data transfer object for showing the latest expense.
#[derive(Deserialize, Serialize)]
pub struct ShowExpense {
    /// The amount of the expense.
    pub amount: i32,
    /// The date of the expense.
    pub date: String,
    /// Optional description of the expense.
    pub description: Option<String>,
    /// The priority level of the expense.
    /// 0: high, 1: medium, 2: low
    pub priority: i16,
    /// The category associated with the expense.
    pub category: sqlx::types::Json<SimpleEntity>,
    /// The tags associated with the expense.
    pub tags: sqlx::types::Json<Vec<Tag>>,
    /// The wallet associated with the expense.
    pub wallet: sqlx::types::Json<SimpleEntity>,
}

/// Data transfer object to show the list of expenses.
#[derive(Deserialize, Serialize)]
pub struct IndexExpenseElement {
    /// The ID of the expense.
    pub id: i32,
    /// The amount of the expense.
    pub amount: i32,
    /// The date of the expense.
    pub date: String,
    /// Optional description of the expense.
    pub description: Option<String>,
}
