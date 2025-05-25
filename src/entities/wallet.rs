use time::Date;

/// Entity for saving a wallet transfer record in the database.
pub struct SaveWalletTransfer {
    /// The ID of the source wallet.
    pub source_wallet_id: i32,
    /// The ID of the target wallet.
    pub target_wallet_id: i32,
    /// The amount of the transfer.
    pub amount: i32,
    /// The date of the transfer.
    pub date: Date,
    /// Optional description of the transfer.
    pub description: Option<String>,
}

/// Entity for saving a wallet transfer fee record in the database.
/// It will be inserted into the `expense` table.
pub struct SaveWalletTransferFee {
    /// The priority of the fee. It's defaulted to secondary priority.
    pub priority: i32,
    /// The ID of the source wallet.
    pub wallet_id: i32,
    /// The ID that indicates fee category.
    pub category_id: i32,
    /// The amount of the fee.
    pub amount: i32,
    /// The date of the transfer.
    pub date: Date,
    /// Optional description of the fee.
    pub description: Option<String>,
}
