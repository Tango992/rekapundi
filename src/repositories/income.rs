use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, query, query_as};
use std::sync::Arc;

use crate::dtos::{
    income::{IndexIncomeQuery, SaveIncome},
    query_result::{IndexIncomeElement, ShowIncome, ShowLatestIncome, SimpleEntity},
};

/// Repository to interact with the `income` table in the database.
pub struct Repository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl Repository {
    /// Creates a new `IncomeRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Trait defining operations for the `income` table.
#[async_trait]
pub trait RepositoryOperation: Send + Sync {
    /// Deletes an income from the database.
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error>;
    /// Finds all incomes from the database.
    async fn find_all(
        &self,
        query: &IndexIncomeQuery,
    ) -> Result<Vec<IndexIncomeElement>, sqlx::Error>;
    /// Finds the latest income from the database.
    async fn find_latest(&self) -> Result<ShowLatestIncome, sqlx::Error>;
    /// Finds a specific income by ID from the database.
    async fn find_one(&self, id: i32) -> Result<ShowIncome, sqlx::Error>;
    /// Inserts multiple incomes into the database.
    async fn insert_bulk(&self, incomes: Vec<SaveIncome>) -> Result<(), sqlx::Error>;
    /// Updates an existing income in the database.
    async fn update(&self, id: i32, income: &SaveIncome) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl RepositoryOperation for Repository {
    async fn find_all(
        &self,
        query: &IndexIncomeQuery,
    ) -> Result<Vec<IndexIncomeElement>, sqlx::Error> {
        let incomes = query_as!(
            IndexIncomeElement,
            r#"
            SELECT
                id,
                amount,
                TO_CHAR(date, 'YYYY-MM-DD') AS "date!",
                description
            FROM
                income
            WHERE
                ($1::DATE IS NULL OR date >= $1::DATE)
                AND ($2::DATE IS NULL OR date <= $2::DATE)
            ORDER BY id
            LIMIT $3 OFFSET $4
            "#,
            query.start_date,
            query.end_date,
            query.pagination.limit(),
            query.pagination.offset(),
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(incomes)
    }

    async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {
        let rows_affected = query!("DELETE FROM income WHERE id = $1", id)
            .execute(&*self.pool)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn find_latest(&self) -> Result<ShowLatestIncome, sqlx::Error> {
        let latest_income = query_as!(
            ShowLatestIncome,
            r#"
            SELECT
                i.id,
                i.amount,
                TO_CHAR(i.date, 'YYYY-MM-DD') AS "date!",
                i.description,
                JSONB_BUILD_OBJECT(
                    'id', w.id,
                    'name', w.name
                ) AS "wallet!: sqlx::types::Json<SimpleEntity>"
            FROM
                income i
            JOIN
                wallet w ON i.wallet_id = w.id
            ORDER BY id DESC
            LIMIT 1
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(latest_income)
    }

    async fn find_one(&self, id: i32) -> Result<ShowIncome, sqlx::Error> {
        let income = query_as!(
            ShowIncome,
            r#"
            SELECT
                i.amount,
                TO_CHAR(i.date, 'YYYY-MM-DD') AS "date!",
                i.description,
                JSONB_BUILD_OBJECT(
                    'id', w.id,
                    'name', w.name
                ) AS "wallet!: sqlx::types::Json<SimpleEntity>"
            FROM
                income i
            JOIN
                wallet w ON i.wallet_id = w.id
            WHERE i.id = $1
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(income)
    }

    async fn insert_bulk(&self, incomes: Vec<SaveIncome>) -> Result<(), sqlx::Error> {
        let mut income_query = QueryBuilder::<Postgres>::new(
            "INSERT INTO income (amount, date, description, wallet_id) ",
        );

        income_query.push_values(&incomes, |mut builder, income| {
            builder
                .push_bind(income.amount)
                .push_bind(income.date)
                .push_bind(income.description.clone())
                .push_bind(income.wallet_id);
        });

        let mut tx = self.pool.begin().await?;
        income_query.build().execute(&mut *tx).await?;
        tx.commit().await?;

        Ok(())
    }

    async fn update(&self, id: i32, income: &SaveIncome) -> Result<(), sqlx::Error> {
        let rows_affected = query!(
            r#"
            UPDATE income
            SET amount = $1,
                date = $2,
                description = $3,
                wallet_id = $4
            WHERE id = $5
            "#,
            income.amount,
            income.date,
            income.description.clone(),
            income.wallet_id,
            id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}
