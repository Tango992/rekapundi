use serde::{Deserialize, Serialize};

/// Reusable struct for entities with an ID and name.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct SimpleEntity {
    /// The ID of the entity.
    pub id: i64,
    /// The name of the entity.
    pub name: String,
}

/// Represents a record of `tag` table in the database.
#[derive(Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[cfg_attr(test, serde(rename_all(deserialize = "camelCase")))]
pub struct Tag {
    /// The ID of the tag.
    pub id: i64,
    /// The name of the tag.
    pub name: String,
    /// Whether the tag has precedence over other tags.
    pub is_important: bool,
}

/// Represents a record of `parent_category` table and its children in the database.
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, Deserialize, Eq, PartialEq))]
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
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
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
    pub priority: i32,
    /// The category associated with the expense.
    pub category: sqlx::types::Json<SimpleEntity>,
    /// The tags associated with the expense.
    pub tags: sqlx::types::Json<Vec<Tag>>,
    /// The wallet associated with the expense.
    pub wallet: sqlx::types::Json<SimpleEntity>,
}

/// Data transfer object for showing the latest expense.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct ShowExpense {
    /// The amount of the expense.
    pub amount: i32,
    /// The date of the expense.
    pub date: String,
    /// Optional description of the expense.
    pub description: Option<String>,
    /// The priority level of the expense.
    /// 0: high, 1: medium, 2: low
    pub priority: i32,
    /// The category associated with the expense.
    pub category: sqlx::types::Json<SimpleEntity>,
    /// The tags associated with the expense.
    pub tags: sqlx::types::Json<Vec<Tag>>,
    /// The wallet associated with the expense.
    pub wallet: sqlx::types::Json<SimpleEntity>,
}

/// Data transfer object to show the list of expenses.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
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

/// Data transfer object for showing a single income.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct ShowIncome {
    /// The amount of the income.
    pub amount: i32,
    /// The date of the income.
    pub date: String,
    /// Optional description of the income.
    pub description: Option<String>,
    /// The wallet associated with the income.
    pub wallet: sqlx::types::Json<SimpleEntity>,
}

/// Data transfer object for showing the latest income.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct ShowLatestIncome {
    /// The identifier of the income.
    pub id: i32,
    /// The amount of the income.
    pub amount: i32,
    /// The date of the income.
    pub date: String,
    /// Optional description of the income.
    pub description: Option<String>,
    /// The wallet associated with the income.
    pub wallet: sqlx::types::Json<SimpleEntity>,
}

/// Data transfer object to show the list of incomes.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct IndexIncomeElement {
    /// The ID of the income.
    pub id: i32,
    /// The amount of the income.
    pub amount: i32,
    /// The date of the income.
    pub date: String,
    /// Optional description of the income.
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct SimpleAmountEntity {
    pub name: String,
    pub amount: i32,
}

#[derive(Serialize)]
pub struct ExpenseParentCategory {
    pub name: String,
    pub amount: i32,
    pub categories: Vec<SimpleAmountEntity>,
}

#[derive(Serialize)]
pub struct ExpensePriority {
    pub level: i16,
    pub amount: i32,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ExpenseGroupedSummary {
    pub parent_categories: Vec<ExpenseParentCategory>,
    pub priorities: Vec<ExpensePriority>,
}

#[derive(Serialize)]
pub struct IncomeGroupedSummary {
    pub wallets: Vec<SimpleAmountEntity>,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ExpenseSummary {
    pub amount: i32,
    pub grouped_summary: ExpenseGroupedSummary,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct IncomeSummary {
    pub amount: i32,
    pub grouped_summary: IncomeGroupedSummary,
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ShowSummary {
    pub expense: sqlx::types::Json<ExpenseSummary>,
    pub income: sqlx::types::Json<IncomeSummary>,
}
