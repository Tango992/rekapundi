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
#[derive(Serialize)]
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
