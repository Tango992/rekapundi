use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    dtos::query_result::SimpleEntity,
    entities::wallet::{SaveWalletTransfer, SaveWalletTransferFee},
};

/// Repository to interact with the `wallet` table in the database.
/// Additionally, it may interact with the `wallet_transfer` and `expense` tables.
pub struct Repository {
    /// The PostgreSQL connection pool.
    pool: Arc<PgPool>,
}

impl Repository {
    /// Creates a new `WalletRepository` instance.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait RepositoryOperation: Send + Sync {
    /// Finds multiple wallets from the database.
    /// The result is paginated based on the provided offset and limit.
    async fn find_many(&self, offset: i64, limit: i64) -> Result<Vec<SimpleEntity>, sqlx::Error>;

    /// Saves a record of money transfer between wallets.
    /// If a fee record is provided, the fee will be saved in the `expense` table.
    async fn insert_wallet_transfer_with_fee(
        &self,
        wallet_transfer_record: &SaveWalletTransfer,
        fee_record: Option<&SaveWalletTransferFee>,
    ) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl RepositoryOperation for Repository {
    async fn find_many(&self, offset: i64, limit: i64) -> Result<Vec<SimpleEntity>, sqlx::Error> {
        let wallets = sqlx::query_as!(
            SimpleEntity,
            r#"
            SELECT id, name
            FROM wallet
            ORDER BY LOWER(name)
            OFFSET $1 LIMIT $2
            "#,
            offset,
            limit,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(wallets)
    }

    async fn insert_wallet_transfer_with_fee(
        &self,
        wallet_transfer_record: &SaveWalletTransfer,
        fee_record: Option<&SaveWalletTransferFee>,
    ) -> Result<(), sqlx::Error> {
        let insert_wallet_transfer_with_fee_query = sqlx::query!(
            r#"
            INSERT INTO wallet_transfer (source_wallet_id, target_wallet_id, amount, date, description)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            wallet_transfer_record.source_wallet_id,
            wallet_transfer_record.target_wallet_id,
            wallet_transfer_record.amount,
            wallet_transfer_record.date,
            wallet_transfer_record.description,
        );

        match fee_record {
            None => {
                insert_wallet_transfer_with_fee_query
                    .execute(&*self.pool)
                    .await?;
            }

            Some(fee_record) => {
                let mut tx = self.pool.begin().await?;

                insert_wallet_transfer_with_fee_query
                    .execute(&mut *tx)
                    .await?;

                sqlx::query!(
                    r#"
                    INSERT INTO expense (category_id, priority, wallet_id, amount, date, description)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                    fee_record.category_id,
                    fee_record.priority,
                    fee_record.wallet_id,
                    fee_record.amount,
                    fee_record.date,
                    fee_record.description,
                )
                .execute(&mut *tx).await?;

                tx.commit().await?;
            }
        }

        Ok(())
    }
}
