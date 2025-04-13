use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::dtos::{
    query_result::{ExpenseSummary, IncomeSummary, ShowSummary},
    summary::GenerateSummaryRequest,
};

/// Repository responsible for handling the summary of income and expenses.
/// It interacts with the `expense` and `income` tables in the database.
pub struct SummaryRepository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl SummaryRepository {
    /// Creates a new `SummaryRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait RepositoryOperation: Send + Sync {
    /// Generates a summary of income and expenses based on the provided request.
    async fn generate_raw(
        &self,
        request: &GenerateSummaryRequest,
    ) -> Result<ShowSummary, sqlx::Error>;
}

#[async_trait]
impl RepositoryOperation for SummaryRepository {
    async fn generate_raw(
        &self,
        request: &GenerateSummaryRequest,
    ) -> Result<ShowSummary, sqlx::Error> {
        let query = sqlx::query_as!(
            ShowSummary,
            r#"
            WITH filtered_expense AS (
                SELECT e.amount, e.date,  e.category_id, e.priority
                FROM expense e
                JOIN category c ON e.category_id = c.id
                WHERE
                    e.date BETWEEN $1::DATE AND $2::DATE
                    AND e.category_id != ALL($3::INT[])
            ),
            filtered_income AS (
                SELECT amount, date, wallet_id
                FROM income
                WHERE date BETWEEN $1 AND $2
            ),
            total_expense AS (
                SELECT COALESCE(SUM(fe.amount), 0) AS amount
                FROM filtered_expense fe
            ),
            total_income AS (
                SELECT COALESCE(SUM(amount), 0) AS amount
                FROM filtered_income
            ),
            category_summary AS (
                SELECT 
                    pc.id AS parent_id,
                    c.name,
                    COALESCE(SUM(fe.amount), 0) AS amount
                FROM filtered_expense fe
                JOIN category c ON fe.category_id = c.id
                JOIN parent_category pc ON c.parent_category_id = pc.id
                GROUP BY pc.id, c.name
            ),
            parent_category_summary AS (
                SELECT 
                    pc.id,
                    pc.name,
                    COALESCE(SUM(cs.amount), 0) AS amount,
                    COALESCE(
                        JSONB_AGG(
                            JSONB_BUILD_OBJECT(
                                'name', cs.name,
                                'amount', cs.amount
                            ) ORDER BY cs.amount DESC
                        ),
                        '[]'
                    ) AS categories
                FROM category_summary cs
                JOIN parent_category pc ON cs.parent_id = pc.id
                WHERE cs.amount > 0
                GROUP BY pc.id, pc.name
            ),
            priority_summary AS (
                SELECT 
                    priority AS level,
                    COALESCE(SUM(amount), 0) AS amount
                FROM filtered_expense
                GROUP BY priority
                ORDER BY amount DESC
            ),
            wallet_summary AS (
                SELECT 
                    w.name,
                    COALESCE(SUM(fi.amount), 0) AS amount
                FROM filtered_income fi
                JOIN wallet w ON fi.wallet_id = w.id
                GROUP BY w.name, fi.amount
                ORDER BY fi.amount DESC
            )
            SELECT 
                JSONB_BUILD_OBJECT(
                    'amount', te.amount,
                    'group_summary', JSONB_BUILD_OBJECT(
                        'parent_categories', (
                            SELECT COALESCE(
                                JSONB_AGG(
                                    JSONB_BUILD_OBJECT(
                                        'name', name,
                                        'amount', amount,
                                        'categories', categories
                                    ) ORDER BY amount DESC
                                ),
                                '[]'
                            )
                            FROM parent_category_summary
                        ),
                        'priorities', (
                            SELECT COALESCE(
                                JSONB_AGG(
                                    JSONB_BUILD_OBJECT(
                                        'level', level,
                                        'amount', amount
                                    ) ORDER BY amount DESC
                                ),
                                '[]'
                            )
                            FROM priority_summary
                        )
                    )
                ) AS "expense!: sqlx::types::Json<ExpenseSummary>",
                JSONB_BUILD_OBJECT(
                    'amount', ti.amount,
                    'group_summary', JSONB_BUILD_OBJECT(
                        'wallets', (
                            SELECT COALESCE(
                                JSONB_AGG(
                                    JSONB_BUILD_OBJECT(
                                        'name', name,
                                        'amount', amount
                                    ) ORDER BY amount DESC
                                ),
                                '[]'
                            )
                            FROM wallet_summary
                        )
                    )
                ) AS "income!: sqlx::types::Json<IncomeSummary>"
            FROM
                total_income ti,
                total_expense te
            "#,
            request.start_date,
            request.end_date,
            &request.exclude_category_ids,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(query)
    }
}
