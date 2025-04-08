use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row, query, query_as};
use std::sync::Arc;

use crate::dtos::{
    expense::{IndexExpenseQuery, SaveExpense},
    query_result::{IndexExpenseElement, ShowExpense, ShowLatestExpense, SimpleEntity, Tag},
};

/// Repository to interact with the `expense` table in the database.
pub struct Repository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl Repository {
    /// Creates a new `ExpenseRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Trait defining operations for the `expense` table.
#[async_trait]
pub trait RepositoryOperation: Send + Sync {
    /// Deletes an expense from the database.
    async fn delete(&self, id: i32) -> Result<(), sqlx::Error>;
    /// Finds all expenses from the database.
    async fn find_all(
        &self,
        query: &IndexExpenseQuery,
    ) -> Result<Vec<IndexExpenseElement>, sqlx::Error>;
    /// Finds the latest expense from the database.
    async fn find_latest(&self) -> Result<ShowLatestExpense, sqlx::Error>;
    /// Finds a specific expense by ID from the database.
    async fn find_one(&self, id: i32) -> Result<ShowExpense, sqlx::Error>;
    /// Inserts multiple expenses into the database.
    async fn insert_bulk(&self, expenses: &[SaveExpense]) -> Result<(), sqlx::Error>;
    /// Updates an existing expense in the database.
    async fn update(&self, id: i32, expense: &SaveExpense) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl RepositoryOperation for Repository {
    async fn find_all(
        &self,
        query: &IndexExpenseQuery,
    ) -> Result<Vec<IndexExpenseElement>, sqlx::Error> {
        let expenses = query_as!(
            IndexExpenseElement,
            r#"
            SELECT
                id,
                amount,
                TO_CHAR(date, 'YYYY-MM-DD') AS "date!",
                description
            FROM
                expense
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

        Ok(expenses)
    }

    async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {
        let rows_affected = query!("DELETE FROM expense WHERE id = $1", id)
            .execute(&*self.pool)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn find_latest(&self) -> Result<ShowLatestExpense, sqlx::Error> {
        let latest_expense = query_as!(
            ShowLatestExpense,
            r#"
            SELECT
                e.id,
                e.amount,
                TO_CHAR(e.date, 'YYYY-MM-DD') AS "date!",
                e.description,
                e.priority,
                JSONB_BUILD_OBJECT(
                    'id', c.id,
                    'name', c.name
                ) AS "category!: sqlx::types::Json<SimpleEntity>",
                JSONB_BUILD_OBJECT(
                    'id', w.id,
                    'name', w.name
                ) AS "wallet!: sqlx::types::Json<SimpleEntity>",
                COALESCE(
                    JSONB_AGG(
                        JSONB_BUILD_OBJECT(
                            'id', t.id,
                            'name', t.name,
                            'is_important', t.is_important
                        ) ORDER BY (CASE WHEN t.is_important IS true THEN 0 ELSE 1 END), t.name
                    ) FILTER (WHERE t.id IS NOT NULL), 
                    '[]'
                ) AS "tags!: sqlx::types::Json<Vec<Tag>>"
            FROM
                expense e
            JOIN
                category c ON e.category_id = c.id
            JOIN
                wallet w ON e.wallet_id = w.id
            LEFT JOIN
                expense_tag et ON e.id = et.expense_id
            LEFT JOIN 
                tag t ON et.tag_id = t.id
            GROUP BY
                e.id, c.id, w.id
            ORDER BY id DESC
            LIMIT 1
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(latest_expense)
    }

    async fn find_one(&self, id: i32) -> Result<ShowExpense, sqlx::Error> {
        let latest_expense = query_as!(
            ShowExpense,
            r#"
            SELECT
                e.amount,
                TO_CHAR(e.date, 'YYYY-MM-DD') AS "date!",
                e.description,
                e.priority,
                JSONB_BUILD_OBJECT(
                    'id', c.id,
                    'name', c.name
                ) AS "category!: sqlx::types::Json<SimpleEntity>",
                JSONB_BUILD_OBJECT(
                    'id', w.id,
                    'name', w.name
                ) AS "wallet!: sqlx::types::Json<SimpleEntity>",
                COALESCE(
                    JSONB_AGG(
                        JSONB_BUILD_OBJECT(
                            'id', t.id,
                            'name', t.name,
                            'is_important', t.is_important
                        ) ORDER BY (CASE WHEN t.is_important IS true THEN 0 ELSE 1 END), t.name
                    ) FILTER (WHERE t.id IS NOT NULL), 
                    '[]'
                ) AS "tags!: sqlx::types::Json<Vec<Tag>>"
            FROM
                expense e
            JOIN
                category c ON e.category_id = c.id
            JOIN
                wallet w ON e.wallet_id = w.id
            LEFT JOIN
                expense_tag et ON e.id = et.expense_id
            LEFT JOIN 
                tag t ON et.tag_id = t.id
            WHERE e.id = $1
            GROUP BY
                e.id, c.id, w.id
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(latest_expense)
    }

    async fn insert_bulk(&self, expenses: &[SaveExpense]) -> Result<(), sqlx::Error> {
        let mut expense_query = QueryBuilder::<Postgres>::new(
            "INSERT INTO expense (amount, date, description, category_id, wallet_id, priority) ",
        );

        expense_query.push_values(expenses, |mut builder, expense| {
            builder
                .push_bind(expense.amount as i32)
                .push_bind(expense.date)
                .push_bind(expense.description.clone())
                .push_bind(expense.category_id as i32)
                .push_bind(expense.wallet_id as i32)
                .push_bind(expense.priority as i16);
        });
        expense_query.push(" RETURNING id");

        let mut tx = self.pool.begin().await?;

        let expense_inserted_ids = expense_query
            .build()
            .fetch_all(&mut *tx)
            .await?
            .iter()
            .map(|row| row.try_get(0).unwrap())
            .collect::<Vec<i32>>();

        drop(expense_query);

        // A flag to check if the expense_tag_query is empty to avoid executing an empty query.
        let mut is_expense_tag_query_empty = true;

        // Array of tuples to hold the values for the expense_tag table.
        // The order of the tuple is (expense_id, tag_id).
        let mut expense_tag_values = Vec::<(i32, i32)>::new();

        for i in 0..expenses.len() {
            let expense_tag_ids = &expenses[i].tag_ids;
            let expense_id = expense_inserted_ids[i];

            for tag_id in expense_tag_ids {
                is_expense_tag_query_empty = false;
                expense_tag_values.push((expense_id, *tag_id as i32));
            }
        }

        if is_expense_tag_query_empty {
            tx.commit().await?;
            return Ok(());
        }

        let mut expense_tag_query =
            QueryBuilder::<Postgres>::new("INSERT INTO expense_tag (expense_id, tag_id) ");

        expense_tag_query.push_values(expense_tag_values, |mut builder, (expense_id, tag_id)| {
            builder.push_bind(expense_id).push_bind(tag_id);
        });

        expense_tag_query.build().execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update(&self, id: i32, expense: &SaveExpense) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let rows_affected = query!(
            r#"
            UPDATE expense
            SET amount = $1,
                date = $2,
                description = $3,
                category_id = $4,
                wallet_id = $5,
                priority = $6
            WHERE id = $7
            "#,
            expense.amount as i32,
            expense.date,
            expense.description.clone(),
            expense.category_id as i32,
            expense.wallet_id as i32,
            expense.priority as i16,
            id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            tx.rollback().await?;
            return Err(sqlx::Error::RowNotFound);
        }

        query!(
            r#"
            DELETE FROM expense_tag
            WHERE expense_id = $1
            "#,
            id
        )
        .execute(&mut *tx)
        .await?;

        let mut expense_tag_query =
            QueryBuilder::<Postgres>::new("INSERT INTO expense_tag (expense_id, tag_id) ");
        expense_tag_query.push_values(&expense.tag_ids, |mut builder, tag_id| {
            builder.push_bind(id).push_bind(*tag_id as i32);
        });

        expense_tag_query.build().execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }
}
