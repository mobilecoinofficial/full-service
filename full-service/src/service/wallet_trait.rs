use crate::service::decorated_types::{JsonWalletStatus, JsonCreateAccountResponse, JsonAccount, JsonTxo, JsonBalanceResponse, JsonAddress, JsonSubmitResponse, JsonTransactionLog, JsonBlock, JsonBlockContents, JsonProof};
use crate::error::WalletServiceError;
use crate::db::account::AccountID;
use mc_mobilecoind_json::data_types::{JsonTxProposal, JsonTx, JsonTxOut};

/// A MobileCoin wallet.
pub trait Wallet {

    /// An overview of this wallet.
    fn get_wallet_status(&self) -> Result<JsonWalletStatus, WalletServiceError>;

    /// Create a new account.
    ///
    /// # Arguments
    /// * `name` - The name of the account.
    /// * `first_block` - ???
    fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<JsonCreateAccountResponse, WalletServiceError>;

    /// Import an existing account into this wallet.
    ///
    /// # Arguments
    /// * `entropy` - ???
    /// * `name` - ???
    /// * `first_block` - ???
    fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<JsonAccount, WalletServiceError>;

    /// Get an account by ID.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    /// * `name` - The new account name.
    fn get_account(
        &self,
        account_id_hex: &AccountID,
    ) -> Result<JsonAccount, WalletServiceError>;

    /// Get all accounts.
    fn list_accounts(&self) -> Result<Vec<JsonAccount>, WalletServiceError>;

    /// Set the account name.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    /// * `name` - The new account name.
    fn update_account_name(
        &self,
        account_id_hex: &str,
        name: String,
    ) -> Result<JsonAccount, WalletServiceError>;

    /// Deletes an account.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletServiceError>;

    /// Get all TXOs associated with the given account.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    fn list_txos(&self, account_id_hex: &str) -> Result<Vec<JsonTxo>, WalletServiceError>;

    /// Get a TXO by ID.
    ///
    /// # Arguments
    /// * `txo_id_hex` - ???
    fn get_txo(&self, txo_id_hex: &str) -> Result<JsonTxo, WalletServiceError>;

    /// Get the current balance of the given account.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    fn get_balance(
        &self,
        account_id_hex: &str,
    ) -> Result<JsonBalanceResponse, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    /// * `comment` - ???
    fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: Option<&str>,
        // FIXME: WS-32 - add "sync from block"
    ) -> Result<JsonAddress, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    fn list_assigned_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonAddress>, WalletServiceError>;

    /// Creates a transaction (but does not submit it).
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    /// * `recipient_public_address` - ???
    /// * `value` - ???
    /// * `input_txo_ids` - ???
    /// * `fee` - ???
    /// * `tombstone_block` - ???
    /// * `max_spendable_value` - ???
    #[allow(clippy::too_many_arguments)]
    fn build_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<JsonTxProposal, WalletServiceError>;

    /// Submit a transaction
    ///
    /// # Arguments
    /// * `tx_proposal` - The transaction to submit.
    /// * `comment` - ???
    /// * `account_id_hex` - ???
    fn submit_transaction(
        &self,
        tx_proposal: JsonTxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<JsonSubmitResponse, WalletServiceError>;

    /// Convenience method that builds and submits in one go.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    /// * `recipient_public_address` - ???
    /// * `value` - ???
    /// * `input_txo_ids` - ???
    /// * `fee` - ???
    /// * `tombstone_block` - ???
    /// * `max_spendable_value` - ???
    /// * `comment` - ???
    #[allow(clippy::too_many_arguments)]
    fn send_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    ) -> Result<JsonSubmitResponse, WalletServiceError>;

    /// Get all transactions associated with the given account.
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<JsonTransactionLog>, WalletServiceError>;

    /// Get a transaction by ID.
    ///
    /// # Arguments
    /// * `transaction_id_hex` - ???
    fn get_transaction(
        &self,
        transaction_id_hex: &str,
    ) -> Result<JsonTransactionLog, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `transaction_id_hex` - ???
    fn get_transaction_object(
        &self,
        transaction_id_hex: &str,
    ) -> Result<JsonTx, WalletServiceError>;

    /// Get a TXO by ID.
    ///
    /// # Arguments
    /// * `txo_id_hex` - ???
    fn get_txo_object(&self, txo_id_hex: &str) -> Result<JsonTxOut, WalletServiceError>;

    /// Get a block by index.
    ///
    /// # Arguments
    /// * `block_index` - The block index.
    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(JsonBlock, JsonBlockContents), WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `transaction_log_id` - ???
    fn get_proofs(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<JsonProof>, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `account_id_hex` - ???
    /// * `txo_id_hex` - ???
    /// * `proof_hex` - ???
    fn verify_proof(
        &self,
        account_id_hex: &str,
        txo_id_hex: &str,
        proof_hex: &str,
    ) -> Result<bool, WalletServiceError>;
}
