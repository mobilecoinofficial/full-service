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
use bip39::Mnemonic;
use diesel::prelude::*;
use mc_account_keys::{
    AccountKey, PublicAddress, RootEntropy, RootIdentity, ViewAccountKey, CHANGE_SUBADDRESS_INDEX,
    DEFAULT_SUBADDRESS_INDEX,
};
use mc_core::slip10::Slip10KeyGenerator;
use mc_crypto_digestible::{Digestible, MerlinTranscript};
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_transaction_core::TokenId;
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
    ///| `fog_report_id`         | Fog Report Key.                                                         | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                  | Applicable only if user has Fog service, empty string otherwise.      |
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
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Create an account from root entropy.
    ///
    /// # Arguments
    ///
    ///| Name                    | Purpose                                                                 | Notes                                                                 |
    ///|-------------------------|-------------------------------------------------------------------------|-----------------------------------------------------------------------|
    ///| `entropy`               | The secret root entropy.                                                | 32 bytes of randomness, hex-encoded.                                  |
    ///| `first_block_index`     | Index of the first block when this account may have received funds.     | Defaults to 0 if not provided                                         |
    ///| `import_block_index`    | Index of the last block in local ledger database.                       |                                                                       |
    ///| `next_subaddress_index` | This index represents the next subaddress to be assigned as an address. | This is useful information in case the account is imported elsewhere. |
    ///| `name`                  | The display name for the account.                                       | A label can have duplicates, but it is not recommended.               |
    ///| `fog_report_url`        | Fog Report server url.                                                  | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_report_id`         | Fog Report Key.                                                         | Applicable only if user has Fog service, empty string otherwise.      |
    ///| `fog_authority_spki`    | Fog Authority Subject Public Key Info.                                  | Applicable only if user has Fog service, empty string otherwise.      |
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
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Create an account.
    ///
    /// Returns:
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
        conn: &Conn,
    ) -> Result<(AccountID, String), WalletDbError>;

    /// Import account.
    #[allow(clippy::too_many_arguments)]
    fn import(
        mnemonic: &Mnemonic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<Account, WalletDbError>;

    /// Import account.
    #[allow(clippy::too_many_arguments)]
    fn import_legacy(
        entropy: &RootEntropy,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<Account, WalletDbError>;

    /// Import a view only account.
    fn import_view_only(
        view_private_key: &RistrettoPrivate,
        spend_public_key: &RistrettoPublic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        conn: &Conn,
    ) -> Result<Account, WalletDbError>;

    /// List all accounts.
    ///
    /// Returns:
    /// * Vector of all Accounts in the DB
    fn list_all(
        conn: &Conn,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Account>, WalletDbError>;

    /// Get a specific account.
    ///
    /// Returns:
    /// * Account
    fn get(account_id: &AccountID, conn: &Conn) -> Result<Account, WalletDbError>;

    /// Get the accounts associated with the given Txo.
    fn get_by_txo_id(txo_id_hex: &str, conn: &Conn) -> Result<Vec<Account>, WalletDbError>;

    /// Update an account.
    /// The only updatable field is the name. Any other desired update requires
    /// adding a new account, and deleting the existing if desired.
    fn update_name(&self, new_name: String, conn: &Conn) -> Result<(), WalletDbError>;

    /// Update the next block index this account will need to sync.
    fn update_next_block_index(
        &self,
        next_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError>;

    /// Delete an account.
    fn delete(self, conn: &Conn) -> Result<(), WalletDbError>;

    /// Get change public address
    fn change_subaddress(self, conn: &Conn) -> Result<AssignedSubaddress, WalletDbError>;

    /// Get main public address
    fn main_subaddress(self, conn: &Conn) -> Result<AssignedSubaddress, WalletDbError>;

    /// Get all of the token ids present for the account
    fn get_token_ids(self, conn: &Conn) -> Result<Vec<TokenId>, WalletDbError>;

    /// Get the next sequentially unassigned subaddress index for the account
    /// (reserved addresses are not included)
    fn next_subaddress_index(self, conn: &Conn) -> Result<u64, WalletDbError>;

    fn account_key(&self) -> Result<Option<AccountKey>, WalletDbError>;

    fn view_account_key(&self) -> Result<ViewAccountKey, WalletDbError>;

    fn view_private_key(&self) -> Result<RistrettoPrivate, WalletDbError>;
}

impl AccountModel for Account {
    fn create_from_mnemonic(
        mnemonic: &Mnemonic,
        first_block_index: Option<u64>,
        import_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        name: &str,
        fog_report_url: String,
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<(AccountID, String), WalletDbError> {
        let fog_enabled = !fog_report_url.is_empty();

        let slip_10_key = mnemonic.clone().derive_slip10_key(0);
        let account_key: AccountKey = slip_10_key.into();
        let account_key_with_fog = account_key.with_fog(
            &fog_report_url,
            fog_report_id,
            base64::decode(fog_authority_spki)?,
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
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<(AccountID, String), WalletDbError> {
        let fog_enabled = !fog_report_url.is_empty();

        let root_id = RootIdentity {
            root_entropy: entropy.clone(),
            fog_report_url,
            fog_report_id,
            fog_authority_spki: base64::decode(fog_authority_spki).expect("invalid spki"),
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
        conn: &Conn,
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
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<Account, WalletDbError> {
        let (account_id, _public_address_b58) = Account::create_from_mnemonic(
            mnemonic,
            first_block_index,
            Some(import_block_index),
            next_subaddress_index,
            &name.unwrap_or_default(),
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
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
        fog_report_id: String,
        fog_authority_spki: String,
        conn: &Conn,
    ) -> Result<Account, WalletDbError> {
        let (account_id, _public_address_b58) = Account::create_from_root_entropy(
            root_entropy,
            first_block_index,
            Some(import_block_index),
            next_subaddress_index,
            &name.unwrap_or_default(),
            fog_report_url,
            fog_report_id,
            fog_authority_spki,
            conn,
        )?;
        Account::get(&account_id, conn)
    }

    fn import_view_only(
        view_private_key: &RistrettoPrivate,
        spend_public_key: &RistrettoPublic,
        name: Option<String>,
        import_block_index: u64,
        first_block_index: Option<u64>,
        next_subaddress_index: Option<u64>,
        conn: &Conn,
    ) -> Result<Account, WalletDbError> {
        use crate::db::schema::accounts;

        let view_account_key = ViewAccountKey::new(*view_private_key, *spend_public_key);
        let account_id = AccountID::from(&view_account_key);

        if Account::get(&account_id, conn).is_ok() {
            return Err(WalletDbError::AccountAlreadyExists(account_id.to_string()));
        }

        let first_block_index = first_block_index.unwrap_or(DEFAULT_FIRST_BLOCK_INDEX) as i64;
        let next_block_index = first_block_index;

        let next_subaddress_index =
            next_subaddress_index.unwrap_or(DEFAULT_NEXT_SUBADDRESS_INDEX) as i64;

        let new_account = NewAccount {
            id: &account_id.to_string(),
            account_key: &mc_util_serial::encode(&view_account_key),
            entropy: None,
            key_derivation_version: MNEMONIC_KEY_DERIVATION_VERSION as i32,
            first_block_index,
            next_block_index,
            import_block_index: Some(import_block_index as i64),
            name: &name.unwrap_or_default(),
            fog_enabled: false,
            view_only: true,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .execute(conn)?;

        AssignedSubaddress::create_for_view_only_account(
            &view_account_key,
            DEFAULT_SUBADDRESS_INDEX,
            "Main",
            conn,
        )?;

        AssignedSubaddress::create_for_view_only_account(
            &view_account_key,
            LEGACY_CHANGE_SUBADDRESS_INDEX,
            "Legacy Change",
            conn,
        )?;

        AssignedSubaddress::create_for_view_only_account(
            &view_account_key,
            CHANGE_SUBADDRESS_INDEX,
            "Change",
            conn,
        )?;

        for subaddress_index in DEFAULT_NEXT_SUBADDRESS_INDEX..next_subaddress_index as u64 {
            AssignedSubaddress::create_for_view_only_account(
                &view_account_key,
                subaddress_index,
                "",
                conn,
            )?;
        }

        Account::get(&account_id, conn)
    }

    fn list_all(
        conn: &Conn,
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

    fn get(account_id: &AccountID, conn: &Conn) -> Result<Account, WalletDbError> {
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

    fn get_by_txo_id(txo_id_hex: &str, conn: &Conn) -> Result<Vec<Account>, WalletDbError> {
        let txo = Txo::get(txo_id_hex, conn)?;

        let mut accounts: Vec<Account> = Vec::<Account>::new();

        if let Some(account_id) = txo.account_id {
            let account = Account::get(&AccountID(account_id), conn)?;
            accounts.push(account);
        }

        Ok(accounts)
    }

    fn update_name(&self, new_name: String, conn: &Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts;

        diesel::update(accounts::table.filter(accounts::id.eq(&self.id)))
            .set(accounts::name.eq(new_name))
            .execute(conn)?;
        Ok(())
    }

    fn update_next_block_index(
        &self,
        next_block_index: u64,
        conn: &Conn,
    ) -> Result<(), WalletDbError> {
        use crate::db::schema::accounts;
        diesel::update(accounts::table.filter(accounts::id.eq(&self.id)))
            .set(accounts::next_block_index.eq(next_block_index as i64))
            .execute(conn)?;
        Ok(())
    }

    fn delete(self, conn: &Conn) -> Result<(), WalletDbError> {
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

    fn change_subaddress(self, conn: &Conn) -> Result<AssignedSubaddress, WalletDbError> {
        AssignedSubaddress::get_for_account_by_index(&self.id, CHANGE_SUBADDRESS_INDEX as i64, conn)
    }

    fn main_subaddress(self, conn: &Conn) -> Result<AssignedSubaddress, WalletDbError> {
        AssignedSubaddress::get_for_account_by_index(
            &self.id,
            DEFAULT_SUBADDRESS_INDEX as i64,
            conn,
        )
    }

    fn get_token_ids(self, conn: &Conn) -> Result<Vec<TokenId>, WalletDbError> {
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

    fn next_subaddress_index(self, conn: &Conn) -> Result<u64, WalletDbError> {
        use crate::db::schema::assigned_subaddresses;

        let highest_subaddress_index: i64 = assigned_subaddresses::table
            .filter(assigned_subaddresses::account_id.eq(&self.id))
            .order_by(assigned_subaddresses::subaddress_index.desc())
            .select(diesel::dsl::max(assigned_subaddresses::subaddress_index))
            .select(assigned_subaddresses::subaddress_index)
            .first(conn)?;

        Ok(highest_subaddress_index as u64 + 1)
    }

    fn account_key(&self) -> Result<Option<AccountKey>, WalletDbError> {
        if self.view_only {
            return Ok(None);
        }

        let account_key: AccountKey = mc_util_serial::decode(&self.account_key)?;
        Ok(Some(account_key))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::WalletDbTestContext;
    use mc_account_keys::RootIdentity;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};
    use std::{collections::HashSet, convert::TryFrom, iter::FromIterator};

    #[test_with_logger]
    fn test_account_crud(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let account_id_hex = {
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's Main Account",
                "".to_string(),
                "".to_string(),
                "".to_string(),
                &conn,
            )
            .unwrap();
            account_id_hex
        };

        {
            let conn = wallet_db.get_conn().unwrap();
            let res = Account::list_all(&conn, None, None).unwrap();
            assert_eq!(res.len(), 1);
        }

        let acc = Account::get(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
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
        };
        assert_eq!(expected_account, acc);

        // Verify that the subaddress table entries were updated for main and change
        let subaddresses = AssignedSubaddress::list_all(
            Some(account_id_hex.to_string()),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
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
                &wallet_db.get_conn().unwrap(),
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
                "".to_string(),
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();
        let res = Account::list_all(&wallet_db.get_conn().unwrap(), None, None).unwrap();
        assert_eq!(res.len(), 2);

        let acc_secondary =
            Account::get(&account_id_hex_secondary, &wallet_db.get_conn().unwrap()).unwrap();
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
        };
        assert_eq!(expected_account_secondary, acc_secondary);

        // Update the name for the secondary account
        acc_secondary
            .update_name(
                "Alice's Secondary Account".to_string(),
                &wallet_db.get_conn().unwrap(),
            )
            .unwrap();
        let acc_secondary2 =
            Account::get(&account_id_hex_secondary, &wallet_db.get_conn().unwrap()).unwrap();
        expected_account_secondary.name = "Alice's Secondary Account".to_string();
        assert_eq!(expected_account_secondary, acc_secondary2);

        // Delete the secondary account
        acc_secondary
            .delete(&wallet_db.get_conn().unwrap())
            .unwrap();

        let res = Account::list_all(&wallet_db.get_conn().unwrap(), None, None).unwrap();
        assert_eq!(res.len(), 1);

        // Attempt to get the deleted account
        let res = Account::get(&account_id_hex_secondary, &wallet_db.get_conn().unwrap());
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
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's Main Account",
                "".to_string(),
                "".to_string(),
                "".to_string(),
                &conn,
            )
            .unwrap();
            account_id_hex
        };
        let account = Account::get(&account_id, &wallet_db.get_conn().unwrap()).unwrap();
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
            let conn = wallet_db.get_conn().unwrap();
            let (account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
                &root_id.root_entropy,
                Some(0),
                None,
                None,
                "Alice's FOG Account",
                "fog//some.fog.url".to_string(),
                "".to_string(),
                "DefinitelyARealFOGAuthoritySPKI".to_string(),
                &conn,
            )
            .unwrap();
            account_id_hex
        };

        {
            let conn = wallet_db.get_conn().unwrap();
            let res = Account::list_all(&conn, None, None).unwrap();
            assert_eq!(res.len(), 1);
        }

        let acc = Account::get(&account_id_hex, &wallet_db.get_conn().unwrap()).unwrap();
        let expected_account = Account {
            id: account_id_hex.to_string(),
            account_key: [
                10, 34, 10, 32, 129, 223, 141, 215, 200, 104, 120, 117, 123, 154, 151, 210, 253,
                23, 148, 151, 2, 18, 182, 100, 83, 138, 144, 99, 225, 74, 214, 14, 175, 68, 167, 4,
                18, 34, 10, 32, 24, 98, 18, 92, 9, 50, 142, 184, 114, 99, 34, 125, 211, 54, 146,
                33, 98, 71, 179, 56, 136, 67, 98, 97, 230, 228, 31, 194, 119, 169, 189, 8, 26, 17,
                102, 111, 103, 47, 47, 115, 111, 109, 101, 46, 102, 111, 103, 46, 117, 114, 108,
                42, 23, 13, 231, 226, 158, 43, 94, 151, 32, 17, 121, 169, 69, 56, 96, 46, 182, 26,
                43, 138, 220, 146, 60, 162,
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

        let account = {
            let conn = wallet_db.get_conn().unwrap();

            Account::import_view_only(
                &view_private_key,
                &spend_public_key,
                Some("View Only Account".to_string()),
                12,
                None,
                None,
                &conn,
            )
            .unwrap()
        };

        {
            let conn = wallet_db.get_conn().unwrap();
            let res = Account::list_all(&conn, None, None).unwrap();
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
        };
        assert_eq!(expected_account, account);
    }
}
