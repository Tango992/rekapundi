use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

/// Repository to interact with other supporting tables in the database.
/// This includes tables like `category`, `tag`, and `wallet`.
pub struct UtilRepository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl UtilRepository {
    /// Creates a new `UtilRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Reusable struct for entities with an ID and name.
#[derive(Deserialize, Serialize)]
pub struct SimpleEntity {
    /// The ID of the entity.
    id: i64,
    /// The name of the entity.
    name: String,
}

/// Represents a record of `tag` table in the database.
#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Tag {
    /// The ID of the tag.
    id: i64,
    /// The name of the tag.
    name: String,
    /// Whether the tag has precedence over other tags.
    is_important: bool,
}

/// Represents a record of `parent_category` table and its children in the database.
#[derive(sqlx::FromRow, Serialize)]
pub struct ParentCategory {
    /// The ID of the parent category.
    id: i64,
    /// The name of the parent category.
    name: String,
    /// The list of child categories.
    categories: sqlx::types::Json<Vec<SimpleEntity>>,
}

#[async_trait]
pub trait UtilOperation {
    /// Finds multiple categories from the database.
    /// The result is paginated based on the provided offset and limit.
    async fn find_many_categories(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SimpleEntity>, sqlx::Error>;

    /// Finds multiple parent categories and their children from the database.
    /// The result is paginated based on the provided offset and limit.
    async fn find_many_parent_categories(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ParentCategory>, sqlx::Error>;

    /// Finds multiple tags from the database.
    /// The result is paginated based on the provided offset and limit.
    async fn find_many_tags(
        &self,
        mark_important_value: Option<bool>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Tag>, sqlx::Error>;

    /// Finds multiple wallets from the database.
    /// The result is paginated based on the provided offset and limit.
    async fn find_many_wallets(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SimpleEntity>, sqlx::Error>;
}

#[async_trait]
impl UtilOperation for UtilRepository {
    async fn find_many_categories(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SimpleEntity>, sqlx::Error> {
        let categories = sqlx::query_as!(
            SimpleEntity,
            r#"
            SELECT id, name
            FROM category
            ORDER BY name
            OFFSET $1 LIMIT $2
            "#,
            offset,
            limit,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(categories)
    }

    async fn find_many_parent_categories(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<ParentCategory>, sqlx::Error> {
        let parent_categories = sqlx::query_as!(
            ParentCategory,
            r#"
            SELECT
                pc.id,
                pc.name,
                c.categories AS "categories!: sqlx::types::Json<Vec<SimpleEntity>>"
            FROM
                parent_category pc
            LEFT JOIN LATERAL (
                SELECT COALESCE(
                    JSONB_AGG(
                        JSONB_BUILD_OBJECT('id', c.id, 'name', c.name) ORDER BY c.name
                    ) FILTER (WHERE c.id IS NOT NULL),
                    '[]'::JSONB
                ) AS categories
                FROM
                    category c
                WHERE
                    c.parent_category_id = pc.id
            ) AS c ON TRUE
            ORDER BY
                pc.name
            OFFSET $1 LIMIT $2
            "#,
            offset,
            limit,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(parent_categories)
    }

    async fn find_many_tags(
        &self,
        mark_important_value: Option<bool>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT id, name, is_important
            FROM tag
            WHERE $1::BOOLEAN IS NULL OR is_important = $1
            ORDER BY name
            OFFSET $2 LIMIT $3
            "#,
            mark_important_value,
            offset,
            limit,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(tags)
    }

    async fn find_many_wallets(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SimpleEntity>, sqlx::Error> {
        let wallets = sqlx::query_as!(
            SimpleEntity,
            r#"
            SELECT id, name
            FROM wallet
            ORDER BY name
            OFFSET $1 LIMIT $2
            "#,
            offset,
            limit,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(wallets)
    }
}
