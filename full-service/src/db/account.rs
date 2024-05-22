// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for the Account model.

use crate::{
    db::{
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, NewAccount, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        Conn, WalletDbError,
    },
    util::constants::{
        DEFAULT_FIRST_BLOCK_INDEX, DEFAULT_NEXT_SUBADDRESS_INDEX, LEGACY_CHANGE_SUBADDRESS_INDEX,
        MNEMONIC_KEY_DERIVATION_VERSION, ROOT_ENTROPY_KEY_DERIVATION_VERSION,
    },
};
use base64::engine::{general_purpose::STANDARD as BASE64_ENGINE, Engine};
use bip39::Mnemonic;
use diesel::{
    dsl::{exists, select},
    prelude::*,
};
use mc_account_keys::{
    AccountKey, PublicAddress, RootEntropy, RootIdentity, ViewAccountKey, CHANGE_SUBADDRESS_INDEX,
    DEFAULT_SUBADDRESS_INDEX,
};
use mc_core::slip10::Slip10KeyGenerator;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_transaction_core::{get_tx_out_shared_secret, TokenId};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AccountID(pub String);

impl From<&AccountKey> for AccountID {
    fn from(src: &AccountKey) -> Self {
        let main_subaddress = src.subaddress(DEFAULT_SUBADDRESS_INDEX);
        AccountID::from(&main_subaddress)
    }
}

impl From<&ViewAccountKey> for AccountID {
    fn from(src: &ViewAccountKey) -> Self {
        let main_subaddress = src.subaddress(DEFAULT_SUBADDRESS_INDEX);
        AccountID::from(&main_subaddress)
    }
}

impl From<&PublicAddress> for AccountID {
    fn from(src: &PublicAddress) -> Self {
        let temp: [u8; 32] = src.digest32::<MerlinTranscript>(b"account_data");
        Self(hex::encode(temp))
    }
}

impl From<String> for AccountID {
    fn from(src: String) -> Self {
        Self(src)
    }
}

impl fmt::Display for AccountID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[rustfmt::skip]
pub trait AccountModel {
    /// Create an account from mnemonic.
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                                 | Notes                                                                 |
    ///|-------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `mnemonic`              | A BIP39-encoded mnemonic phrase used to generate the account key.       |                                                                       |
    ///| `first_block_index`     | Index of the first block when this account may have received funds.     | Defaults to 0 if not provided                                         |
    ///| `import_block_index`    | Index of the last block in local ledger database.                       |                                                                       |
    ///| `next_subaddress_index` | This index represents the next subaddress to be assigned as an address. | This is useful information in case the account is imported elsewhere. |
    ///| `name`                  | The display name for the account.                                       | A label can have duplicates, but it is not recommended.               |
    ///| `fog_report_url`        | Fog Report server url.                                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `require_spend_subaddress` | If enabled, this mode requires all transactions to spend from a provided subaddress |                                                      |
    ///
    /// # Returns:
    /// * (account_id, main_subaddress_b58)
    #[allow(clippy::too_many_arguments)]
    fn create_from_mnemonic(
        mnemonic: &Mnemonic,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Create an account from root entropy.
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                                 | Notes                                                                 |
    ///|-------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `entropy`               | The secret root entropy used to generate the account key.               | 32 bytes of randomness, hex-encoded.                                  |
    ///| `first_block_index`     | Index of the first block when this account may have received funds.     | Defaults to 0 if not provided                                         |
    ///| `import_block_index`    | Index of the last block in local ledger database.                       |                                                                       |
    ///| `next_subaddress_index` | This index represents the next subaddress to be assigned as an address. | This is useful information in case the account is imported elsewhere. |
    ///| `name`                  | The display name for the account.                                       | A label can have duplicates, but it is not recommended.               |
    ///| `fog_report_url`        | Fog Report server url.                                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `require_spend_subaddress` | If enabled, this mode requires all transactions to spend from a provided subaddress |                                                      |
    ///| `conn`                  | An reference to the pool connection of wallet database                  |                                                                       |
    ///
    /// # Returns:
    /// * (account_id, main_subaddress_b58)
    #[allow(clippy::too_many_arguments)]
    fn create_from_root_entropy(
        entropy: &RootEntropy,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Create an account from either mnemonic phrase (v2) or root entropy (v1).
    ///
    /// # Arguments
    ///
    ///| Name                     | Purpose                                                                                           | Notes                                                                 |
    ///|--------------------------|---------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `entropy`                | Either a BIP39-encoded mnemonic phrase or a secret root entropy used to generate the account key. | Depends on the `key_derivation_version` parameter                     |
    ///| `key_derivation_version` | The version number of the key derivation path used to create a account key.                       | "2" for mnemonic phrase and "1" for root entropy                      |
    ///| `account_key`            | Contains a View keypair and a Spend keypair, used to construct and receive transactions.          | Also may contain keys to connect to the Fog ledger scanning service.  |
    ///| `first_block_index`      | Index of the first block when this account may have received funds.                               | Defaults to 0 if not provided                                         |
    ///| `import_block_index`     | Index of the last block in local ledger database.                                                 |                                                                       |
    ///| `next_subaddress_index`  | This index represents the next subaddress to be assigned as an address.                           | This is useful information in case the account is imported elsewhere. |
    ///| `name`                   | The display name for the account.                                                                 | A label can have duplicates, but it is not recommended.               |
    ///| `fog_enabled`            | Indicate if fog server is enabled or disabled                                                     |                                                                       |
    ///| `require_spend_subaddress` | If enabled, this mode requires all transactions to spend from a provided subaddress |                                                      |
    ///| `conn`                   | An reference to the pool connection of wallet database                                            |                                                                       |
    ///
    /// # Returns:
    /// * (account_id, main_subaddress_b58)
    #[allow(clippy::too_many_arguments)]
    fn create(
        entropy: &[u8],
        key_derivation_version: u8,
        account_key: &AccountKey,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_enabled: bool,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Import account from a mnemonic phrase (v2).
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                                 | Notes                                                                 |
    ///|-------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `mnemonic`              | A BIP39-encoded mnemonic phrase used to generate the account key.       |                                                                       |
    ///| `name`                  | The display name for the account.                                       | A label can have duplicates, but it is not recommended.               |
    ///| `import_block_index`    | Index of the last block in local ledger database.                       |                                                                       |
    ///| `first_block_index`     | Index of the first block when this account may have received funds.     | Defaults to 0 if not provided                                         |
    ///| `next_subaddress_index` | This index represents the next subaddress to be assigned as an address. | This is useful information in case the account is imported elsewhere. |
    ///| `fog_report_url`        | Fog Report server url.                                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `require_spend_subaddress` | If enabled, this mode requires all transactions to spend from a provided subaddress |                                                      |
    ///| `conn`                  | An reference to the pool connection of wallet database                  |                                                                       |
    ///
    /// # Returns:
    /// * Account
    #[allow(clippy::too_many_arguments)]
    fn import(
        mnemonic: &Mnemonic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError>;

    /// Import account from a root entropy (v1).
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                                 | Notes                                                                 |
    ///|-------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `entropy`               | The secret root entropy used to generate the account key.               | 32 bytes of randomness, hex-encoded.                                  |
    ///| `name`                  | The display name for the account.                                       | A label can have duplicates, but it is not recommended.               |
    ///| `import_block_index`    | Index of the last block in local ledger database.                       |                                                                       |
    ///| `first_block_index`     | Index of the first block when this account may have received funds.     | Defaults to 0 if not provided                                         |
    ///| `next_subaddress_index` | This index represents the next subaddress to be assigned as an address. | This is useful information in case the account is imported elsewhere. |
    ///| `fog_report_url`        | Fog Report server url.                                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `require_spend_subaddress` | If enabled, this mode requires all transactions to spend from a provided subaddress |                                                      |
    ///| `conn`                  | An reference to the pool connection of wallet database                  |                                                                       |
    ///
    /// # Returns:
    /// * Account
    #[allow(clippy::too_many_arguments)]
    fn import_legacy(
        entropy: &RootEntropy,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError>;

    /// Import a view only account.
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                                 | Notes                                                                 |
    ///|-------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `view_private_key`      | The view private key of this import candidate.                          | Grant view only permission                                            |
    ///| `spend_public_key`      | The spend public key of this import candidate.                          | Used to generate new subaddresses                                     |
    ///| `name`                  | The display name for the account.                                       | A label can have duplicates, but it is not recommended.               |
    ///| `import_block_index`    | Index of the last block in local ledger database.                       |                                                                       |
    ///| `first_block_index`     | Index of the first block when this account may have received funds.     | Defaults to 0 if not provided                                         |
    ///| `next_subaddress_index` | This index represents the next subaddress to be assigned as an address. | This is useful information in case the account is imported elsewhere. |
    ///| `managed_by_hardware_wallet` | Whether the account is managed by a hardware wallet.                 |                                                                       |
    ///| `require_spend_subaddress` | If enabled, this mode requires all transactions to spend from a provided subaddress |                                                        |
    ///| `conn`                  | An reference to the pool connection of wallet database                  |                                                                       |
    ///
    /// # Returns:
    /// * Account
    #[allow(clippy::too_many_arguments)]
    fn import_view_only(
        view_account_key: &ViewAccountKey,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        managed_by_hardware_wallet: bool,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError>;

    fn import_view_only_from_hardware_wallet_with_fog(
        view_account_key: &ViewAccountKey,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        default_public_address: &PublicAddress,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError>;

    /// List all accounts from wallet DB.
    ///
    /// # Arguments
    ///
    ///| Name     | Purpose                                                   | Notes                    |
    ///|----------|-----------------------------------------------------------|--------------------------|
    ///| `conn`   | An reference to the pool connection of wallet database    |                          |
    ///| `offset` | The pagination offset. Results start at the offset index. | Optional, defaults to 0. |
    ///| `limit`  | Limit for the number of results.                          | Optional                 |
    ///
    /// # Returns:
    /// * Vector of all Accounts in the DB
    fn list_all(
        conn: Conn,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Account>, WalletDbError>;

    /// Get a specific account.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                                | Notes                             |
    ///|--------------|--------------------------------------------------------|-----------------------------------|
    ///| `account_id` | The account ID used to perform this GET action.        | Account must exist in the wallet. |
    ///| `conn`       | An reference to the pool connection of wallet database |                                   |
    ///
    /// # Returns:
    /// * Account
    fn get(
        account_id: &AccountID,
        conn: Conn
    ) -> Result<Account, WalletDbError>;

    /// Get the accounts associated with the given Txo.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                                              | Notes |
    ///|--------------|----------------------------------------------------------------------|-------|
    ///| `txo_id_hex` | The txo ID for which to get all accounts associate with this txo ID. |       |
    ///| `conn`       | An reference to the pool connection of wallet database               |       |
    ///
    /// # Returns:
    /// *  Vector of all Accounts associated with the given Txo
    fn get_by_txo_id(
        txo_id_hex: &str,
        conn: Conn
    ) -> Result<Vec<Account>, WalletDbError>;

    /// Update the account name for current account.
    /// * The only updatable field is the name. Any other desired update requires adding a new account, and deleting the existing if desired.
    ///
    /// # Arguments
    ///| Name       | Purpose                                                  | Notes |
    ///|------------|----------------------------------------------------------|-------|
    ///| `new_name` | The new account name used to perform this update action. |       |
    ///| `conn`     | An reference to the pool connection of wallet database   |       |
    ///
    /// # Returns:
    /// * unit
    fn update_name(
        &self,
        new_name: String,
        conn: Conn
    ) -> Result<(), WalletDbError>;

    /// Update the next block index in current account that needs to sync.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `next_block_index` | The next block index in current account that needs to sync. |       |
    ///| `conn`             | An reference to the pool connection of wallet database      |       |
    ///
    /// # Returns:
    /// * unit
    fn update_next_block_index(
        &self,
        next_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError>;

    /// Delete the current account.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `conn`             | An reference to the pool connection of wallet database      |       |
    ///
    /// # Returns:
    /// * unit
    fn delete(self, conn: Conn) -> Result<(), WalletDbError>;

    /// Get subaddress for the current account where funds are returned when the input txos exceed the amount spent.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `conn`             | An reference to the pool connection of wallet database      |       |
    ///
    /// # Returns:
    /// * AssignedSubaddress
    fn change_subaddress(self, conn: Conn) -> Result<AssignedSubaddress, WalletDbError>;

    /// Get main public address for the current account
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `conn`             | An reference to the pool connection of wallet database      |       |
    ///
    /// # Returns:
    /// * AssignedSubaddress
    fn main_subaddress(self, conn: Conn) -> Result<AssignedSubaddress, WalletDbError>;

    /// Get all of the token ids present for the current account.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `conn`             | An reference to the pool connection of wallet database      |       |
    ///
    /// # Returns:
    /// * Vector of all TokenIds
    fn get_token_ids(self, conn: Conn) -> Result<Vec<TokenId>, WalletDbError>;

    /// Get the next sequentially unassigned subaddress index for the account
    /// * reserved addresses are not included
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `conn`             | An reference to the pool connection of wallet database      |       |
    ///
    /// # Returns:
    /// * Vector of all TokenIds
    fn next_subaddress_index(self, conn: Conn) -> Result<u64, WalletDbError>;

    /// Get the account key for the current account.
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns:
    /// * AccountKey
    fn account_key(&self) -> Result<AccountKey, WalletDbError>;

    /// Get the view only account key for the current account.
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns:
    /// * ViewAccountKey
    fn view_account_key(&self) -> Result<ViewAccountKey, WalletDbError>;

    /// Get the view private account key for the current account.
    /// * A wrapper function to get view_private_key from a ViewAccountKey instance for current
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns:
    /// * RistrettoPrivate
    fn view_private_key(&self) -> Result<RistrettoPrivate, WalletDbError>;


    /// Get the shared secret for the account and the tx_public_key
    ///
    /// # Arguments
    ///| Name               | Purpose                                                     | Notes |
    ///|--------------------|-------------------------------------------------------------|-------|
    ///| `tx_public_key`    | A cryptographic key that is part of a tx_out on a block     |       |
    ///
    /// # Returns:
    /// * RistrettoPublic
    fn get_shared_secret(&self, tx_public_key: &RistrettoPublic) -> Result<RistrettoPublic, WalletDbError>;

    fn public_address(&self, index: u64) -> Result<PublicAddress, WalletDbError>;

    fn update_resyncing(&self, resyncing: bool, conn: Conn) -> Result<(), WalletDbError>;

    fn resync_in_progress(conn: Conn) -> Result<bool, WalletDbError>;
}

impl AccountModel for Account {
    fn create_from_mnemonic(
        mnemonic: &Mnemonic,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<(AccountID, String), WalletDbError> {
        let fog_enabled = !fog_report_url.is_empty();

        let slip_10_key = mnemonic.clone().derive_slip10_key(0);
        let account_key: AccountKey = slip_10_key.into();
        let account_key_with_fog = account_key.with_fog(
            &fog_report_url,
            "".to_string(),
            BASE64_ENGINE.decode(fog_authority_spki)?,
        );

        Account::create(
            mnemonic.entropy(),
            MNEMONIC_KEY_DERIVATION_VERSION,
            &account_key_with_fog,
            first_block_index,
            import_block_index,
            next_subaddress_index,
            name,
            fog_enabled,
            require_spend_subaddress,
            conn,
        )
    }

    fn create_from_root_entropy(
        entropy: &RootEntropy,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<(AccountID, String), WalletDbError> {
        let fog_enabled = !fog_report_url.is_empty();

        let root_id = RootIdentity {
            root_entropy: entropy.clone(),
            fog_report_url,
            fog_report_id: "".to_string(),
            fog_authority_spki: BASE64_ENGINE.decode(fog_authority_spki)?,
        };
        let account_key = AccountKey::from(&root_id);

        Account::create(
            &entropy.bytes,
            ROOT_ENTROPY_KEY_DERIVATION_VERSION,
            &account_key,
            first_block_index,
            import_block_index,
            next_subaddress_index,
            name,
            fog_enabled,
            require_spend_subaddress,
            conn,
        )
    }

    fn create(
        entropy: &[u8],
        key_derivation_version: u8,
        account_key: &AccountKey,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_enabled: bool,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<(AccountID, String), WalletDbError> {
        use crate::db::schema::accounts;

        let account_id = AccountID::from(account_key);

        if Account::get(&account_id, conn).is_ok() {
            return Err(WalletDbError::AccountAlreadyExists(account_id.to_string()));
        }

        let first_block_index = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX);
        let next_block_index = first_block_index;

        let next_subaddress_index =
            next_subaddress_index.unwrap_or(DEFAULT_NEXT_SUBADDRESS_INDEX) as i64;

        let new_account = NewAccount {
            id: &account_id.to_string(),
            account_key: &mc_util_serial::encode(account_key),
            entropy: Some(entropy),
            key_derivation_version: key_derivation_version as i32,
            first_block_index: first_block_index as i64,
            next_block_index: next_block_index as i64,
            import_block_index: import_block_index.map(|i| i as i64),
            name,
            fog_enabled,
            view_only: false,
            managed_by_hardware_wallet: false,
            require_spend_subaddress: require_spend_subaddress,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .execute(conn)?;

        let main_subaddress_b58 =
            AssignedSubaddress::create(account_key, DEFAULT_SUBADDRESS_INDEX, "Main", conn)?;

        AssignedSubaddress::create(account_key, CHANGE_SUBADDRESS_INDEX, "Change", conn)?;

        if !fog_enabled {
            AssignedSubaddress::create(
                account_key,
                LEGACY_CHANGE_SUBADDRESS_INDEX,
                "Legacy Change",
                conn,
            )?;

            for subaddress_index in DEFAULT_NEXT_SUBADDRESS_INDEX..next_subaddress_index as u64 {
                AssignedSubaddress::create(account_key, subaddress_index, "", conn)?;
            }
        }

        Ok((account_id, main_subaddress_b58))
    }

    fn import(
        mnemonic: &Mnemonic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError> {
        let (account_id, _public_address_b58) = Account::create_from_mnemonic(
            mnemonic,
            first_block_index,
            Some(import_block_index),
            next_subaddress_index,
            &name.unwrap_or_default(),
            fog_report_url,
            fog_authority_spki,
            require_spend_subaddress,
            conn,
        )?;
        Account::get(&account_id, conn)
    }

    fn import_legacy(
        root_entropy: &RootEntropy,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_authority_spki: String,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError> {
        let (account_id, _public_address_b58) = Account::create_from_root_entropy(
            root_entropy,
            first_block_index,
            Some(import_block_index),
            next_subaddress_index,
            &name.unwrap_or_default(),
            fog_report_url,
            fog_authority_spki,
            require_spend_subaddress,
            conn,
        )?;
        Account::get(&account_id, conn)
    }

    fn import_view_only(
        view_account_key: &ViewAccountKey,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        managed_by_hardware_wallet: bool,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts;

        let account_id = AccountID::from(view_account_key);

        if Account::get(&account_id, conn).is_ok() {
            return Err(WalletDbError::AccountAlreadyExists(account_id.to_string()));
        }

        let first_block_index = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX) as i64;
        let next_block_index = first_block_index;

        let next_subaddress_index =
            next_subaddress_index.unwrap_or(DEFAULT_NEXT_SUBADDRESS_INDEX) as i64;

        let new_account = NewAccount {
            id: &account_id.to_string(),
            account_key: &mc_util_serial::encode(view_account_key),
            entropy: None,
            key_derivation_version: MNEMONIC_KEY_DERIVATION_VERSION as i32,
            first_block_index,
            next_block_index,
            import_block_index: Some(import_block_index as i64),
            name: &name.unwrap_or_default(),
            fog_enabled: false,
            view_only: true,
            managed_by_hardware_wallet,
            require_spend_subaddress,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .execute(conn)?;

        AssignedSubaddress::create_for_view_only_account(
            view_account_key,
            DEFAULT_SUBADDRESS_INDEX,
            "Main",
            conn,
        )?;

        AssignedSubaddress::create_for_view_only_account(
            view_account_key,
            LEGACY_CHANGE_SUBADDRESS_INDEX,
            "Legacy Change",
            conn,
        )?;

        AssignedSubaddress::create_for_view_only_account(
            view_account_key,
            CHANGE_SUBADDRESS_INDEX,
            "Change",
            conn,
        )?;

        for subaddress_index in DEFAULT_NEXT_SUBADDRESS_INDEX..next_subaddress_index as u64 {
            AssignedSubaddress::create_for_view_only_account(
                view_account_key,
                subaddress_index,
                "",
                conn,
            )?;
        }

        Account::get(&account_id, conn)
    }

    fn import_view_only_from_hardware_wallet_with_fog(
        view_account_key: &ViewAccountKey,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        default_public_address: &PublicAddress,
        require_spend_subaddress: bool,
        conn: Conn,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts;

        let account_id = AccountID::from(view_account_key);

        if Account::get(&account_id, conn).is_ok() {
            return Err(WalletDbError::AccountAlreadyExists(account_id.to_string()));
        }

        let first_block_index = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX) as i64;
        let next_block_index = first_block_index;

        let new_account = NewAccount {
            id: &account_id.to_string(),
            account_key: &mc_util_serial::encode(view_account_key),
            entropy: None,
            key_derivation_version: MNEMONIC_KEY_DERIVATION_VERSION as i32,
            first_block_index,
            next_block_index,
            import_block_index: Some(import_block_index as i64),
            name: &name.unwrap_or_default(),
            fog_enabled: true,
            view_only: true,
            managed_by_hardware_wallet: true,
            require_spend_subaddress,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .execute(conn)?;

        AssignedSubaddress::create_for_view_only_fog_account(
            view_account_key,
            DEFAULT_SUBADDRESS_INDEX,
            default_public_address,
            "Main",
            conn,
        )?;

        AssignedSubaddress::create_for_view_only_account(
            view_account_key,
            CHANGE_SUBADDRESS_INDEX,
            "Change",
            conn,
        )?;

        Account::get(&account_id, conn)
    }

    fn list_all(
        conn: Conn,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Account>, WalletDbError> {
        use crate::db::schema::accounts;

        let mut query = accounts::table.into_boxed();

        if let (Some(offset), Some(limit)) = (offset, limit) {
            query = query.limit(limit as i64).offset(offset as i64);
        }

        Ok(query.load(conn)?)
    }

    fn get(account_id: &AccountID, conn: Conn) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts;

        match accounts::table
            .filter(accounts::id.eq(account_id.to_string()))
            .get_result::<Account>(conn)
        {
            Ok(a) => Ok(a),
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                Err(WalletDbError::AccountNotFound(account_id.to_string()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_by_txo_id(txo_id_hex: &str, conn: Conn) -> Result<Vec<Account>, WalletDbError> {
        let txo = Txo::get(txo_id_hex, conn)?;

        let mut accounts: Vec<Account> = Vec::<Account>::new();

        if let Some(account_id) = txo.account_id {
            let account = Account::get(&AccountID(account_id), conn)?;
            accounts.push(account);
        }

        Ok(accounts)
    }

    fn update_name(&self, new_name: String, conn: Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts;

        diesel::update(accounts::table.filter(accounts::id.eq(&self.id)))
            .set(accounts::name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }

    fn update_next_block_index(
        &self,
        next_block_index: u64,
        conn: Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts;
        diesel::update(accounts::table.filter(accounts::id.eq(&self.id)))
            .set(accounts::next_block_index.eq(next_block_index as i64))
            .execute(conn)?;
        Ok(())
    }

    fn delete(self, conn: Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts;

        // Delete transaction logs associated with this account
        TransactionLog::delete_all_for_account(&self.id, conn)?;

        // Delete associated assigned subaddresses
        AssignedSubaddress::delete_all(&self.id, conn)?;

        // Delete references to the account in the Txos table.
        Txo::scrub_account(&self.id, conn)?;

        diesel::delete(accounts::table.filter(accounts::id.eq(&self.id))).execute(conn)?;

        // Delete Txos with no references.
        Txo::delete_unreferenced(conn)?;

        Ok(())
    }

    fn change_subaddress(self, conn: Conn) -> Result<AssignedSubaddress, WalletDbError> {
        AssignedSubaddress::get_for_account_by_index(&self.id, CHANGE_SUBADDRESS_INDEX as i64, conn)
    }

    fn main_subaddress(self, conn: Conn) -> Result<AssignedSubaddress, WalletDbError> {
        AssignedSubaddress::get_for_account_by_index(
            &self.id,
            DEFAULT_SUBADDRESS_INDEX as i64,
            conn,
        )
    }

    fn get_token_ids(self, conn: Conn) -> Result<Vec<TokenId>, WalletDbError> {
        use crate::db::schema::txos;

        let distinct_token_ids = txos::table
            .filter(txos::account_id.eq(&self.id))
            .select(txos::token_id)
            .distinct()
            .load::<i64>(conn)?
            .into_iter()
            .map(|i| TokenId::from(i as u64))
            .collect();

        Ok(distinct_token_ids)
    }

    fn next_subaddress_index(self, conn: Conn) -> Result<u64, WalletDbError> {
        use crate::db::schema::assigned_subaddresses;

        let highest_subaddress_index: i64 = assigned_subaddresses::table
            .filter(assigned_subaddresses::account_id.eq(&self.id))
            .order_by(assigned_subaddresses::subaddress_index.desc())
            .select(diesel::dsl::max(assigned_subaddresses::subaddress_index))
            .select(assigned_subaddresses::subaddress_index)
            .first(conn)?;

        Ok(highest_subaddress_index as u64 + 1)
    }

    fn account_key(&self) -> Result<AccountKey, WalletDbError> {
        if self.view_only {
            return Err(WalletDbError::AccountKeyNotAvailableForViewOnlyAccount);
        }

        let account_key: AccountKey = mc_util_serial::decode(&self.account_key)?;
        Ok(account_key)
    }

    fn view_account_key(&self) -> Result<ViewAccountKey, WalletDbError> {
        if self.view_only {
            return Ok(mc_util_serial::decode(&self.account_key)?);
        }

        let account_key: AccountKey = mc_util_serial::decode(&self.account_key)?;
        let view_account_key = ViewAccountKey::from(&account_key);
        Ok(view_account_key)
    }

    fn view_private_key(&self) -> Result<RistrettoPrivate, WalletDbError> {
        Ok(*self.view_account_key()?.view_private_key())
    }

    fn get_shared_secret(
        &self,
        tx_public_key: &RistrettoPublic,
    ) -> Result<RistrettoPublic, WalletDbError> {
        Ok(get_tx_out_shared_secret(
            &self.view_private_key()?,
            tx_public_key,
        ))
    }

    fn public_address(&self, index: u64) -> Result<PublicAddress, WalletDbError> {
        let account_key = self.account_key()?;
        Ok(account_key.subaddress(index))
    }

    fn update_resyncing(&self, resyncing: bool, conn: Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts;

        diesel::update(accounts::table.filter(accounts::id.eq(&self.id)))
            .set(accounts::resyncing.eq(resyncing))
            .execute(conn)?;
        Ok(())
    }

    fn resync_in_progress(conn: Conn) -> Result<bool, WalletDbError> {
        use crate::db::schema::accounts;

        Ok(
            select(exists(accounts::table.filter(accounts::resyncing.eq(true))))
                .get_result(conn)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_utils::WalletDbTestContext, util::b58::b58_encode_public_address};
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPublic;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{collections::HashSet, convert::TryFrom, iter::FromIterator, ops::DerefMut};

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let account_id_hex = {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's Main Account",
                "".to_string(),
                "".to_string(),
                false,
                conn,
            )
            .unwrap();
            account_id_hex
        };

        {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let res = Account::list_all(conn, None, None).unwrap();
            assert_eq!(res.len(), 1);
        }

        let acc = Account::get(
            &account_id_hex,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        let expected_account = Account {
            id: account_id_hex.to_string(),
            account_key: mc_util_serial::encode(&account_key),
            entropy: Some(root_id.root_entropy.bytes.to_vec()),
            key_derivation_version: 1,
            first_block_index: 0,
            next_block_index: 0,
            import_block_index: None,
            name: "Alice's Main Account".to_string(),
            fog_enabled: false,
            view_only: false,
            managed_by_hardware_wallet: false,
            resyncing: false,
            require_spend_subaddress: false,
        };
        assert_eq!(expected_account, acc);

        // Verify that the subaddress table entries were updated for main and change
        let subaddresses = AssignedSubaddress::list_all(
            Some(account_id_hex.to_string()),
            None,
            None,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        assert_eq!(subaddresses.len(), 3);
        let subaddress_indices: HashSet<i64> =
            HashSet::from_iter(subaddresses.iter().map(|s| s.subaddress_index));
        assert!(subaddress_indices.get(&0).is_some());
        assert!(subaddress_indices
            .get(&(CHANGE_SUBADDRESS_INDEX as i64))
            .is_some());

        // Verify that we can get the correct subaddress index from the spend public key
        let main_subaddress = account_key.subaddress(0);
        let (retrieved_index, retrieved_account_id_hex) =
            AssignedSubaddress::find_by_subaddress_spend_public_key(
                main_subaddress.spend_public_key(),
                wallet_db.get_pooled_conn().unwrap().deref_mut(),
            )
            .unwrap();
        assert_eq!(retrieved_index, 0);
        assert_eq!(retrieved_account_id_hex, account_id_hex.to_string());

        // Add another account with no name, scanning from later
        let root_id_secondary = RootIdentity::from_random(&mut rng);
        let account_key_secondary = AccountKey::from(&root_id_secondary);
        let (account_id_hex_secondary, _public_address_b58_secondary) =
            Account::create_from_root_entropy(
                &root_id_secondary.root_entropy,
                Some(50),
                Some(50),
                None,
                "",
                "".to_string(),
                "".to_string(),
                false,
                wallet_db.get_pooled_conn().unwrap().deref_mut(),
            )
            .unwrap();
        let res = Account::list_all(wallet_db.get_pooled_conn().unwrap().deref_mut(), None, None)
            .unwrap();
        assert_eq!(res.len(), 2);

        let acc_secondary = Account::get(
            &account_id_hex_secondary,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        let mut expected_account_secondary = Account {
            id: account_id_hex_secondary.to_string(),
            account_key: mc_util_serial::encode(&account_key_secondary),
            entropy: Some(root_id_secondary.root_entropy.bytes.to_vec()),
            key_derivation_version: 1,
            first_block_index: 50,
            next_block_index: 50,
            import_block_index: Some(50),
            name: "".to_string(),
            fog_enabled: false,
            view_only: false,
            managed_by_hardware_wallet: false,
            resyncing: false,
            require_spend_subaddress: false,
        };
        assert_eq!(expected_account_secondary, acc_secondary);

        // Update the name for the secondary account
        acc_secondary
            .update_name(
                "Alice's Secondary Account".to_string(),
                wallet_db.get_pooled_conn().unwrap().deref_mut(),
            )
            .unwrap();
        let acc_secondary2 = Account::get(
            &account_id_hex_secondary,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        expected_account_secondary.name = "Alice's Secondary Account".to_string();
        assert_eq!(expected_account_secondary, acc_secondary2);

        // Delete the secondary account
        acc_secondary
            .delete(wallet_db.get_pooled_conn().unwrap().deref_mut())
            .unwrap();

        let res = Account::list_all(wallet_db.get_pooled_conn().unwrap().deref_mut(), None, None)
            .unwrap();
        assert_eq!(res.len(), 1);

        // Attempt to get the deleted account
        let res = Account::get(
            &account_id_hex_secondary,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        );
        match res {
            Ok(_) => panic!("Should have deleted account"),
            Err(WalletDbError::AccountNotFound(s)) => {
                assert_eq!(s, account_id_hex_secondary.to_string())
            }
            Err(_) => panic!("Should error with NotFound but got {:?}", res),
        }
    }

    // Providing entropy should succeed and derive account key.
    #[test_with_logger]
    fn test_create_account_from_entropy(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        // Test providing entropy.
        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let account_id = {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's Main Account",
                "".to_string(),
                "".to_string(),
                false,
                conn,
            )
            .unwrap();
            account_id_hex
        };
        let account = Account::get(
            &account_id,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        let decoded_entropy = RootEntropy::try_from(account.entropy.unwrap().as_slice()).unwrap();
        assert_eq!(decoded_entropy, root_id.root_entropy);
        let decoded_account_key: AccountKey = mc_util_serial::decode(&account.account_key).unwrap();
        assert_eq!(decoded_account_key, account_key);
    }

    #[test_with_logger]
    fn test_create_fog_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_id_hex = {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's FOG Account",
                "fog//some.fog.url".to_string(),
                "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ==".to_string(),
                false,
                conn,
            )
                .unwrap();
            account_id_hex
        };

        {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let res = Account::list_all(conn, None, None).unwrap();
            assert_eq!(res.len(), 1);
        }

        let acc = Account::get(
            &account_id_hex,
            wallet_db.get_pooled_conn().unwrap().deref_mut(),
        )
        .unwrap();
        let expected_account = Account {
            id: account_id_hex.to_string(),
            account_key: [
                10, 34, 10, 32, 129, 223, 141, 215, 200, 104, 120, 117, 123, 154, 151, 210, 253,
                23, 148, 151, 2, 18, 182, 100, 83, 138, 144, 99, 225, 74, 214, 14, 175, 68, 167, 4,
                18, 34, 10, 32, 24, 98, 18, 92, 9, 50, 142, 184, 114, 99, 34, 125, 211, 54, 146,
                33, 98, 71, 179, 56, 136, 67, 98, 97, 230, 228, 31, 194, 119, 169, 189, 8, 26, 17,
                102, 111, 103, 47, 47, 115, 111, 109, 101, 46, 102, 111, 103, 46, 117, 114, 108,
                42, 166, 4, 48, 130, 2, 34, 48, 13, 6, 9, 42, 134, 72, 134, 247, 13, 1, 1, 1, 5, 0,
                3, 130, 2, 15, 0, 48, 130, 2, 10, 2, 130, 2, 1, 0, 190, 112, 125, 193, 54, 211, 57,
                62, 110, 162, 44, 209, 97, 166, 27, 195, 181, 200, 18, 66, 39, 151, 193, 59, 48,
                99, 128, 66, 63, 177, 156, 47, 133, 214, 178, 23, 136, 41, 220, 254, 221, 126, 228,
                130, 22, 141, 180, 70, 133, 108, 232, 237, 16, 48, 35, 155, 17, 106, 57, 246, 177,
                62, 43, 70, 127, 77, 93, 179, 179, 100, 203, 200, 168, 78, 33, 154, 25, 132, 207,
                45, 88, 64, 208, 231, 124, 2, 192, 88, 212, 62, 88, 22, 216, 25, 111, 149, 170,
                109, 200, 149, 11, 192, 20, 88, 213, 55, 86, 24, 33, 214, 33, 108, 184, 192, 61,
                217, 32, 121, 91, 22, 125, 194, 195, 126, 138, 199, 159, 170, 209, 221, 0, 89, 97,
                146, 37, 72, 20, 18, 41, 19, 24, 254, 220, 176, 79, 23, 147, 10, 242, 182, 205,
                103, 18, 210, 70, 219, 20, 251, 65, 113, 91, 97, 237, 81, 69, 84, 240, 25, 94, 19,
                226, 247, 44, 181, 2, 249, 229, 116, 238, 167, 236, 162, 226, 186, 149, 102, 162,
                57, 68, 50, 219, 248, 21, 9, 36, 235, 78, 39, 106, 95, 163, 253, 211, 169, 149, 48,
                19, 87, 198, 189, 0, 144, 91, 61, 16, 89, 255, 36, 32, 231, 177, 6, 217, 141, 140,
                70, 33, 206, 66, 39, 201, 199, 68, 185, 85, 179, 114, 50, 111, 91, 145, 47, 19, 15,
                177, 168, 139, 159, 90, 9, 110, 142, 79, 88, 50, 122, 139, 86, 192, 184, 58, 208,
                148, 89, 77, 252, 137, 238, 86, 22, 5, 35, 179, 135, 67, 167, 104, 44, 173, 70,
                148, 150, 180, 214, 122, 109, 170, 207, 81, 227, 167, 126, 214, 231, 38, 209, 60,
                172, 88, 69, 236, 206, 5, 235, 167, 173, 208, 178, 63, 114, 235, 215, 97, 231, 48,
                87, 12, 164, 162, 241, 21, 4, 164, 69, 224, 115, 138, 244, 104, 18, 155, 94, 230,
                64, 203, 115, 172, 62, 220, 231, 159, 19, 109, 171, 156, 57, 102, 155, 97, 29, 56,
                186, 23, 64, 115, 123, 53, 208, 201, 46, 56, 213, 134, 211, 2, 16, 134, 2, 2, 133,
                133, 64, 26, 113, 75, 189, 42, 105, 219, 246, 144, 18, 173, 241, 173, 20, 132, 79,
                27, 34, 197, 66, 35, 177, 156, 150, 89, 19, 164, 184, 13, 8, 22, 119, 60, 119, 176,
                38, 114, 117, 1, 189, 21, 187, 21, 63, 249, 182, 21, 178, 147, 150, 160, 117, 133,
                6, 103, 238, 194, 214, 142, 189, 138, 20, 117, 99, 25, 168, 26, 30, 189, 124, 179,
                15, 32, 67, 161, 98, 30, 123, 219, 237, 220, 146, 9, 118, 73, 165, 120, 186, 143,
                70, 190, 76, 252, 48, 193, 145, 5, 195, 8, 50, 84, 98, 173, 44, 33, 9, 250, 51, 87,
                100, 114, 79, 149, 167, 212, 216, 15, 167, 39, 60, 204, 146, 251, 65, 211, 161, 47,
                180, 49, 111, 238, 86, 142, 112, 149, 173, 6, 7, 35, 103, 2, 3, 1, 0, 1,
            ]
            .to_vec(),
            entropy: Some(root_id.root_entropy.bytes.to_vec()),
            key_derivation_version: 1,

            first_block_index: 0,
            next_block_index: 0,
            import_block_index: None,
            name: "Alice's FOG Account".to_string(),
            fog_enabled: true,
            view_only: false,
            managed_by_hardware_wallet: false,
            resyncing: false,
            require_spend_subaddress: false,
        };
        assert_eq!(expected_account, acc);
    }

    #[test_with_logger]
    fn test_import_view_only_account(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let view_private_key = RistrettoPrivate::from_random(&mut rng);
        let spend_public_key = RistrettoPublic::from_random(&mut rng);

        let view_account_key = ViewAccountKey::new(view_private_key, spend_public_key);

        let account = {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();

            Account::import_view_only(
                &view_account_key,
                Some("View Only Account".to_string()),
                12,
                None,
                None,
                false,
                false,
                conn,
            )
            .unwrap()
        };

        {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let res = Account::list_all(conn, None, None).unwrap();
            assert_eq!(res.len(), 1);
        }

        let expected_account = Account {
            id: account.id.to_string(),
            account_key: [
                10, 34, 10, 32, 66, 186, 14, 57, 108, 119, 153, 172, 224, 25, 53, 237, 22, 219,
                222, 137, 26, 227, 37, 43, 122, 52, 71, 153, 60, 246, 90, 102, 123, 176, 139, 11,
                18, 34, 10, 32, 28, 19, 114, 110, 204, 131, 192, 90, 192, 83, 149, 201, 140, 112,
                168, 124, 195, 19, 252, 208, 160, 39, 44, 28, 108, 143, 40, 149, 53, 137, 20, 47,
            ]
            .to_vec(),
            entropy: None,
            key_derivation_version: 2,
            first_block_index: 0,
            next_block_index: 0,
            import_block_index: Some(12),
            name: "View Only Account".to_string(),
            fog_enabled: false,
            view_only: true,
            managed_by_hardware_wallet: false,
            resyncing: false,
            require_spend_subaddress: false,
        };
        assert_eq!(expected_account, account);
    }

    #[test_with_logger]
    fn test_import_view_only_from_hardware_wallet_with_fog(logger: Logger) {
        // Test Setup
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let account_key = AccountKey::random(&mut rng);
        let account_key = account_key.with_fog(
            "fog//some.fog.url".to_string(),
            "".to_string(),
            "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAvnB9wTbTOT5uoizRYaYbw7XIEkInl8E7MGOAQj+xnC+F1rIXiCnc/t1+5IIWjbRGhWzo7RAwI5sRajn2sT4rRn9NXbOzZMvIqE4hmhmEzy1YQNDnfALAWNQ+WBbYGW+Vqm3IlQvAFFjVN1YYIdYhbLjAPdkgeVsWfcLDforHn6rR3QBZYZIlSBQSKRMY/tywTxeTCvK2zWcS0kbbFPtBcVth7VFFVPAZXhPi9yy1AvnldO6n7KLiupVmojlEMtv4FQkk604nal+j/dOplTATV8a9AJBbPRBZ/yQg57EG2Y2MRiHOQifJx0S5VbNyMm9bkS8TD7Goi59aCW6OT1gyeotWwLg60JRZTfyJ7lYWBSOzh0OnaCytRpSWtNZ6barPUeOnftbnJtE8rFhF7M4F66et0LI/cuvXYecwVwykovEVBKRF4HOK9GgSm17mQMtzrD7c558TbaucOWabYR04uhdAc3s10MkuONWG0wIQhgIChYVAGnFLvSpp2/aQEq3xrRSETxsixUIjsZyWWROkuA0IFnc8d7AmcnUBvRW7FT/5thWyk5agdYUGZ+7C1o69ihR1YxmoGh69fLMPIEOhYh572+3ckgl2SaV4uo9Gvkz8MMGRBcMIMlRirSwhCfozV2RyT5Wn1NgPpyc8zJL7QdOhL7Qxb+5WjnCVrQYHI2cCAwEAAQ=="
        );
        let view_account_key: ViewAccountKey = (&account_key).into();
        let default_public_address = account_key.default_subaddress();

        // Import the account into the database
        let account = {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();

            Account::import_view_only_from_hardware_wallet_with_fog(
                &view_account_key,
                Some("View Only Account".to_string()),
                12,
                None,
                &default_public_address,
                false,
                conn,
            )
            .unwrap()
        };

        // Check to make sure the number of accounts in the database is correct
        {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            let res = Account::list_all(conn, None, None).unwrap();
            assert_eq!(res.len(), 1);
        }

        // Construct the expected account
        let account_key_proto_bytes = mc_util_serial::encode(&view_account_key);
        let expected_account = Account {
            id: account.id.to_string(),
            account_key: account_key_proto_bytes,
            entropy: None,
            key_derivation_version: 2,
            first_block_index: 0,
            next_block_index: 0,
            import_block_index: Some(12),
            name: "View Only Account".to_string(),
            fog_enabled: true,
            view_only: true,
            managed_by_hardware_wallet: true,
            resyncing: false,
            require_spend_subaddress: false,
        };

        // Check to make sure the account in the database is correct
        assert_eq!(account, expected_account);

        // Check to make sure the correct number of subaddresses were created
        let assigned_subaddresses = {
            let mut pooled_conn = wallet_db.get_pooled_conn().unwrap();
            let conn = pooled_conn.deref_mut();
            AssignedSubaddress::list_all(Some(account.id.clone()), None, None, conn).unwrap()
        };

        assert_eq!(assigned_subaddresses.len(), 2);

        // Check to make sure the default subaddress was created correctly
        let default_public_address_b58 =
            b58_encode_public_address(&default_public_address).unwrap();
        let default_subaddress_spend_public_key_bytes = default_public_address
            .spend_public_key()
            .to_bytes()
            .to_vec();

        let default_subaddress = assigned_subaddresses
            .into_iter()
            .find(|s| s.subaddress_index == 0)
            .unwrap();
        let expected_default_subaddress = AssignedSubaddress {
            public_address_b58: default_public_address_b58,
            account_id: account.id,
            subaddress_index: 0,
            comment: "Main".to_string(),
            spend_public_key: default_subaddress_spend_public_key_bytes,
        };

        assert_eq!(default_subaddress, expected_default_subaddress);
    }
}
