//! A MobileCoin wallet.

use crate::db::AccountID;
use crate::json_rpc;
use crate::json_rpc::SubmitResponse;
use crate::service::WalletServiceError;
use mockall::*;

/// A MobileCoin wallet.
#[automock]
pub trait Wallet {
    /// An overview of this wallet.
    fn get_wallet_status(&self) -> Result<json_rpc::WalletStatus, WalletServiceError>;

    /// Create a new account.
    ///
    /// # Arguments
    /// * `name` - The name of the account.
    /// * `first_block` - Previous blocks will be ignored when updating the balance for this account.
    fn create_account(
        &self,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<json_rpc::CreateAccountResponse, WalletServiceError>;

    /// Import an existing account into this wallet.
    ///
    /// # Arguments
    /// * `entropy` - ???
    /// * `name` - The name for this account.
    /// * `first_block` - Previous blocks will be ignored when updating the balance for this account.
    fn import_account(
        &self,
        entropy: String,
        name: Option<String>,
        first_block: Option<u64>,
    ) -> Result<json_rpc::Account, WalletServiceError>;

    /// Get an account by ID.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    fn get_account(
        &self,
        account_id_hex: &AccountID,
    ) -> Result<json_rpc::Account, WalletServiceError>;

    /// Get all accounts.
    fn list_accounts(&self) -> Result<Vec<json_rpc::Account>, WalletServiceError>;

    /// Set the account name.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    /// * `name` - The new account name.
    fn update_account_name(
        &self,
        account_id_hex: &str,
        name: String,
    ) -> Result<json_rpc::Account, WalletServiceError>;

    /// Deletes an account.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    fn delete_account(&self, account_id_hex: &str) -> Result<(), WalletServiceError>;

    /// Get all TXOs associated with the given account.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    fn list_txos(&self, account_id_hex: &str) -> Result<Vec<json_rpc::Txo>, WalletServiceError>;

    /// Get a TXO by ID.
    ///
    /// # Arguments
    /// * `txo_id_hex` - ???
    fn get_txo(&self, txo_id_hex: &str) -> Result<json_rpc::Txo, WalletServiceError>;

    /// Get the current balance of the given account.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    fn get_balance(
        &self,
        account_id_hex: &str,
    ) -> Result<json_rpc::BalanceResponse, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    /// * `comment` - ???
    fn create_assigned_subaddress(
        &self,
        account_id_hex: &str,
        comment: Option<String>,
        // FIXME: WS-32 - add "sync from block"
    ) -> Result<json_rpc::Address, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    fn list_assigned_subaddresses(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<json_rpc::Address>, WalletServiceError>;

    /// Creates a transaction (but does not submit it).
    ///
    /// The transaction sends `value` to a single recipient.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    /// * `recipient_public_address` - The recipient of the transaction.
    /// * `value` - The value (excluding fee) sent by this transaction, denominated in picoMOB.
    /// * `input_txo_ids` - ???
    /// * `fee` - Transaction fee, denominated in picoMOB.
    /// * `tombstone_block` - ???
    /// * `max_spendable_value` - ???
    #[allow(clippy::too_many_arguments)]
    fn build_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTxProposal, WalletServiceError>;

    /// Submit a transaction
    ///
    /// # Arguments
    /// * `tx_proposal` - The transaction to submit.
    /// * `comment` - ???
    /// * `account_id_hex` - Unique account identifier.
    fn submit_transaction(
        &self,
        tx_proposal: mc_mobilecoind_json::data_types::JsonTxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<SubmitResponse, WalletServiceError>;

    /// Convenience method that builds and submits in one go.
    ///
    /// The transaction sends `value` to a single recipient.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    /// * `recipient_public_address` - The recipient of the transaction.
    /// * `value` - The value (excluding fee) sent by this transaction, denominated in picoMOB.
    /// * `input_txo_ids` - ???
    /// * `fee` - Transaction fee, denominated in picoMOB.
    /// * `tombstone_block` - ???
    /// * `max_spendable_value` - ???
    /// * `comment` - ???
    #[allow(clippy::too_many_arguments)]
    fn send_transaction(
        &self,
        account_id_hex: &str,
        recipient_public_address: &str,
        value: String,
        input_txo_ids: Option<Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    ) -> Result<SubmitResponse, WalletServiceError>;

    /// Get all transactions associated with the given account.
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    fn list_transactions(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<json_rpc::TransactionLog>, WalletServiceError>;

    /// Get a transaction by ID.
    ///
    /// # Arguments
    /// * `transaction_id_hex` - ???
    fn get_transaction(
        &self,
        transaction_id_hex: &str,
    ) -> Result<json_rpc::TransactionLog, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `transaction_id_hex` - ???
    fn get_transaction_object(
        &self,
        transaction_id_hex: &str,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTx, WalletServiceError>;

    /// Get a TXO by ID.
    ///
    /// # Arguments
    /// * `txo_id_hex` - ???
    fn get_txo_object(
        &self,
        txo_id_hex: &str,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTxOut, WalletServiceError>;

    /// Get a block by index.
    ///
    /// # Arguments
    /// * `block_index` - The block index.
    fn get_block_object(
        &self,
        block_index: u64,
    ) -> Result<(json_rpc::Block, json_rpc::BlockContents), WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `transaction_log_id` - ???
    fn get_proofs(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<json_rpc::MembershipProof>, WalletServiceError>;

    /// ???
    ///
    /// # Arguments
    /// * `account_id_hex` - Unique account identifier.
    /// * `txo_id_hex` - ???
    /// * `proof_hex` - ???
    fn verify_proof(
        &self,
        account_id_hex: &str,
        txo_id_hex: &str,
        proof_hex: &str,
    ) -> Result<bool, WalletServiceError>;
}

#[cfg(test)]
mod tests {
    use crate::json_rpc;
    use crate::service::wallet_trait::{MockWallet, Wallet};

    /// Example of creating a mock Wallet
    #[test]
    fn mock_wallet_example() {
        let mut mock_wallet = MockWallet::new();

        // WalletStatus, now with more Pinnipeds!
        let expected_status = json_rpc::WalletStatus {
            object: "LeopardSeal".to_string(),
            network_height: "GreySeal".to_string(),
            local_height: "HarbourSeal".to_string(),
            is_synced_all: false,
            total_available_pmob: "RibbonSeal".to_string(),
            total_pending_pmob: "BeardedSeal".to_string(),
            account_ids: vec![],
            account_map: Default::default(),
        };

        // Configure the mock wallet.
        // get_wallet_status should be called once and should return `expected_status`.
        // return_once is used instead of return_const because WalletServiceError is note Clone.
        {
            let expected_status = expected_status.clone();
            mock_wallet
                .expect_get_wallet_status()
                .return_once(move || Ok(expected_status));
        }

        // Query the mock wallet.
        match mock_wallet.get_wallet_status() {
            Ok(status) => assert_eq!(status, expected_status),
            Err(e) => panic!(format!("Unexpected error {}", e)),
        }
    }
}
