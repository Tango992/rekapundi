use async_trait::async_trait;
use serde::Serialize;
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
#[derive(Serialize)]
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

#[async_trait]
pub trait UtilOperation: Send + Sync + 'static {
    /// Finds multiple categories from the database.
    /// The result is paginated based on the provided offset and limit.
    async fn find_many_categories(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SimpleEntity>, sqlx::Error>;

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

    async fn find_many_tags(
        &self,
        mark_important_value: Option<bool>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT id, name, is_important AS is_important
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
