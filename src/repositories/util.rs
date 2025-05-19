use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::dtos::query_result::{ParentCategory, SimpleEntity, Tag};

/// Repository to interact with other supporting tables in the database.
/// This includes tables like `category`, `tag`, and `wallet`.
pub struct Repository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl Repository {
    /// Creates a new `UtilRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait RepositoryOperation: Send + Sync {
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
}

#[async_trait]
impl RepositoryOperation for Repository {
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
            ORDER BY LOWER(name)
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
                        JSONB_BUILD_OBJECT('id', c.id, 'name', c.name) ORDER BY LOWER(c.name)
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
            ORDER BY (CASE WHEN is_important IS true THEN 0 ELSE 1 END), LOWER(name)
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
}
