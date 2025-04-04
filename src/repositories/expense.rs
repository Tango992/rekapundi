use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use std::sync::Arc;

use crate::dtos::expense::SaveExpense;

/// Repository to interact with the `expense` table in the database.
pub struct ExpenseRepository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl ExpenseRepository {
    /// Creates a new `ExpenseRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait ExpenseOperation {
    /// Inserts multiple expenses into the database.
    async fn insert_bulk(&self, expenses: Vec<SaveExpense>) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl ExpenseOperation for ExpenseRepository {
    async fn insert_bulk(&self, expenses: Vec<SaveExpense>) -> Result<(), sqlx::Error> {
        let mut expense_query: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO expense (amount, date, description, category_id, wallet_id) ",
        );

        expense_query.push_values(&expenses, |mut builder, expense| {
            builder
                .push_bind(expense.amount as i32)
                .push_bind(expense.date)
                .push_bind(expense.description.clone())
                .push_bind(expense.category_id as i32)
                .push_bind(expense.wallet_id as i32);
        });
        expense_query.push("RETURNING id");

        let mut tx = self.pool.begin().await?;

        let expense_inserted_ids = expense_query
            .build()
            .fetch_all(&mut *tx)
            .await?
            .iter()
            .filter_map(|row| {
                let id: i32 = row.try_get(0).unwrap();
                Some(id)
            })
            .collect::<Vec<i32>>();

        assert_eq!(expense_inserted_ids.len(), expenses.len());

        let mut expense_tag_query: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO expense_tag (expense_id, tag_id) ");

        let mut is_expense_tag_query_empty = true;

        for i in 0..expenses.len() {
            let expense_tag_ids = &expenses[i].tag_ids;
            let expense_id = expense_inserted_ids[i];

            if !expense_tag_ids.is_empty() {
                is_expense_tag_query_empty = false;
                expense_tag_query.push_values(expense_tag_ids, |mut builder, tag_id| {
                    builder.push_bind(expense_id).push_bind(*tag_id as i32);
                });
            }
        }

        if !is_expense_tag_query_empty {
            expense_tag_query.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
