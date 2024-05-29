// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing balances.
use std::{collections::BTreeMap, convert::TryFrom, ops::DerefMut};

use crate::{
    config::NetworkConfig,
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, Txo},
        txo::TxoModel,
        Conn, WalletDbError,
    },
    service::{
        account::{AccountService, AccountServiceError},
        ledger::{LedgerService, LedgerServiceError},
        WalletService,
    },
};
use displaydoc::Display;
use mc_blockchain_types::BlockVersion;
use mc_common::HashMap;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::{tokens::Mob, FeeMap, FeeMapError, Token, TokenId};

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum BalanceServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error getting network block height: {0}
    NetworkBlockHeight(LedgerServiceError),

    /// Unexpected Account Txo Status: {0}
    UnexpectedAccountTxoStatus(String),

    /// AccountServiceError
    AccountServiceError(AccountServiceError),

    /// FeeMapError: {0}
    FeeMap(FeeMapError),
}

impl From<WalletDbError> for BalanceServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for BalanceServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_ledger_db::Error> for BalanceServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<LedgerServiceError> for BalanceServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::NetworkBlockHeight(src)
    }
}

impl From<AccountServiceError> for BalanceServiceError {
    fn from(src: AccountServiceError) -> Self {
        Self::AccountServiceError(src)
    }
}

impl From<FeeMapError> for BalanceServiceError {
    fn from(src: FeeMapError) -> Self {
        Self::FeeMap(src)
    }
}

/// The balance object returned by balance services.
///
/// This must be a service object because there is no "Balance" table in our
/// data model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Balance {
    pub max_spendable: u128,
    pub unverified: u128,
    pub unspent: u128,
    pub pending: u128,
    pub spent: u128,
    pub secreted: u128,
    pub orphaned: u128,
}

impl Default for &Balance {
    fn default() -> &'static Balance {
        &Balance {
            max_spendable: 0,
            unverified: 0,
            unspent: 0,
            pending: 0,
            spent: 0,
            secreted: 0,
            orphaned: 0,
        }
    }
}

/// The Network Status object.
/// This holds the number of blocks in the ledger, on the network and locally.
pub struct NetworkStatus {
    pub network_block_height: u64,
    pub local_block_height: u64,
    pub local_num_txos: u64,
    pub fees: FeeMap,
    pub block_version: u32,
    pub network_info: NetworkConfig,
}

/// The Wallet Status object returned by balance services.
///
/// This must be a service object because there is no "WalletStatus" table in
/// our data model.
///
/// It shares several fields with balance, but also returns details about the
/// accounts in the wallet.
pub struct WalletStatus {
    pub balance_per_token: BTreeMap<TokenId, Balance>,
    pub network_block_height: u64,
    pub local_block_height: u64,
    pub min_synced_block_index: u64,
    pub account_ids: Vec<AccountID>,
    pub account_map: HashMap<AccountID, Account>,
}

impl WalletStatus {
    pub fn percent_synced(&self) -> u64 {
        self.min_synced_block_index * 100 / self.local_block_height
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// balances.
#[rustfmt::skip]
pub trait BalanceService {
    /// Gets the balance for a given account. Balance consists of the sums of the various txo states in our wallet
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                      | Notes                             |
    ///|--------------|----------------------------------------------|-----------------------------------|
    ///| `account_id` | The account on which to perform this action. | Account must exist in the wallet. |
    ///
    fn get_balance_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<BTreeMap<TokenId, Balance>, BalanceServiceError>;

    /// Get the current balance for a given address.
    ///
    /// # Arguments
    ///
    ///| Name      | Purpose                                      | Notes                                                  |
    ///|-----------|----------------------------------------------|--------------------------------------------------------|
    ///| `address` | The address on which to perform this action. | Address must be assigned for an account in the wallet. |
    ///
    fn get_balance_for_address(
        &self,
        address: &str,
    ) -> Result<BTreeMap<TokenId, Balance>, BalanceServiceError>;

    /// Get the current status of the network.
    fn get_network_status(&self) -> Result<NetworkStatus, BalanceServiceError>;

    /// Get the current status of a wallet. **Note that pmob calculations do not include view-only-accounts**
    fn get_wallet_status(&self) -> Result<WalletStatus, BalanceServiceError>;
}

impl<T, FPR> BalanceService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_balance_for_account(
        &self,
        account_id: &AccountID,
    ) -> Result<BTreeMap<TokenId, Balance>, BalanceServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let account = self.get_account(account_id)?;
        let distinct_token_ids = account.get_token_ids(conn)?;

        let network_status = self.get_network_status()?;

        let balances = distinct_token_ids
            .into_iter()
            .map(|token_id| {
                let default_token_fee = network_status
                    .fees
                    .get_fee_for_token(&token_id)
                    .unwrap_or(0);
                let balance = Self::get_balance_inner(
                    Some(&account_id.to_string()),
                    None,
                    token_id,
                    &default_token_fee,
                    conn,
                )?;
                Ok((token_id, balance))
            })
            .collect::<Result<BTreeMap<TokenId, Balance>, BalanceServiceError>>()?;

        Ok(balances)
    }

    fn get_balance_for_address(
        &self,
        address: &str,
    ) -> Result<BTreeMap<TokenId, Balance>, BalanceServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let assigned_address = AssignedSubaddress::get(address, conn)?;
        let account_id = AccountID::from(assigned_address.account_id);
        let account = self.get_account(&account_id)?;
        let distinct_token_ids = account.get_token_ids(conn)?;
        let network_status = self.get_network_status()?;

        let balances = distinct_token_ids
            .into_iter()
            .map(|token_id| {
                let default_token_fee = network_status
                    .fees
                    .get_fee_for_token(&token_id)
                    .unwrap_or(0);
                let balance = Self::get_balance_inner(
                    None,
                    Some(address),
                    token_id,
                    &default_token_fee,
                    conn,
                )?;
                Ok((token_id, balance))
            })
            .collect::<Result<BTreeMap<TokenId, Balance>, BalanceServiceError>>()?;

        Ok(balances)
    }

    fn get_network_status(&self) -> Result<NetworkStatus, BalanceServiceError> {
        let (network_block_height, fee_map, block_version) = match self.offline {
            true => {
                let mut fees = BTreeMap::new();
                fees.insert(Mob::ID, Mob::MINIMUM_FEE);
                fees.insert(TokenId::from(1), 2560);
                let fee_map = FeeMap::try_from(fees)?;
                (0, fee_map, *BlockVersion::MAX)
            }
            false => {
                let network_block_info = self.get_latest_block_info()?;
                (
                    network_block_info.block_index + 1,
                    FeeMap::try_from(network_block_info.minimum_fees)?,
                    network_block_info.network_block_version,
                )
            }
        };

        Ok(NetworkStatus {
            network_block_height,
            local_block_height: self.ledger_db.num_blocks()?,
            local_num_txos: self.ledger_db.num_txos()?,
            fees: fee_map,
            block_version,
            network_info: self.network_setup_config.clone(),
        })
    }

    // Wallet Status is an overview of the wallet's status
    fn get_wallet_status(&self) -> Result<WalletStatus, BalanceServiceError> {
        let network_status = self.get_network_status()?;

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let accounts = Account::list_all(conn, None, None)?;
        let mut account_map = HashMap::default();

        let mut balance_per_token = BTreeMap::new();

        let mut min_synced_block_index = network_status.network_block_height.saturating_sub(1);
        let mut account_ids = Vec::new();

        for account in accounts {
            let account_id = AccountID(account.id.clone());
            let token_ids = account.clone().get_token_ids(conn)?;

            for token_id in token_ids {
                let default_token_fee = network_status
                    .fees
                    .get_fee_for_token(&token_id)
                    .unwrap_or(0);
                let balance = Self::get_balance_inner(
                    Some(&account_id.to_string()),
                    None,
                    token_id,
                    &default_token_fee,
                    conn,
                )?;
                balance_per_token
                    .entry(token_id)
                    .and_modify(|b: &mut Balance| {
                        b.unverified += balance.unverified;
                        b.unspent += balance.unspent;
                        b.pending += balance.pending;
                        b.spent += balance.spent;
                        b.secreted += balance.secreted;
                        b.orphaned += balance.orphaned;
                    })
                    .or_insert(balance);
            }

            account_map.insert(account_id.clone(), account.clone());

            min_synced_block_index = std::cmp::min(
                min_synced_block_index,
                (account.next_block_index as u64).saturating_sub(1),
            );
            account_ids.push(account_id);
        }

        Ok(WalletStatus {
            balance_per_token,
            network_block_height: network_status.network_block_height,
            local_block_height: self.ledger_db.num_blocks()?,
            min_synced_block_index,
            account_ids,
            account_map,
        })
    }
}

fn sum_query_result(txos: Vec<Txo>) -> u128 {
    txos.iter().map(|t| (t.value as u64) as u128).sum::<u128>()
}

impl<T, FPR> WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    #[allow(clippy::type_complexity)]
    fn get_balance_inner(
        account_id_hex: Option<&str>,
        public_address_b58: Option<&str>,
        token_id: TokenId,
        default_token_fee: &u64,
        conn: Conn,
    ) -> Result<Balance, BalanceServiceError> {
        let unspent = sum_query_result(Txo::list_unspent(
            account_id_hex,
            public_address_b58,
            Some(*token_id),
            None,
            None,
            None,
            None,
            conn,
        )?);

        let spent = sum_query_result(Txo::list_spent(
            account_id_hex,
            public_address_b58,
            Some(*token_id),
            None,
            None,
            None,
            None,
            conn,
        )?);

        let pending = sum_query_result(Txo::list_pending(
            account_id_hex,
            public_address_b58,
            Some(*token_id),
            None,
            None,
            None,
            None,
            conn,
        )?);

        let unverified = sum_query_result(Txo::list_unverified(
            account_id_hex,
            public_address_b58,
            Some(*token_id),
            None,
            None,
            None,
            None,
            conn,
        )?);

        let secreted = sum_query_result(Txo::list_secreted(account_id_hex, conn)?);

        let orphaned = if public_address_b58.is_some() {
            0
        } else {
            sum_query_result(Txo::list_orphaned(
                account_id_hex,
                Some(*token_id),
                None,
                None,
                None,
                None,
                conn,
            )?)
        };

        let spendable_txos_result = Txo::list_spendable(
            account_id_hex,
            None,
            public_address_b58,
            *token_id,
            *default_token_fee,
            conn,
        )?;

        Ok(Balance {
            max_spendable: spendable_txos_result.max_spendable_in_wallet,
            unverified,
            unspent,
            pending,
            spent,
            secreted,
            orphaned,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        service::{account::AccountService, address::AddressService},
        test_utils::{get_test_ledger, manually_sync_account, setup_wallet_service, MOB},
        util::b58::b58_encode_public_address,
    };
    use mc_account_keys::{AccountKey, PublicAddress, RootEntropy, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_transaction_core::{tokens::Mob, Token};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    // The balance for an address should be accurate.
    #[test_with_logger]
    fn test_address_balance(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let entropy = RootEntropy::from_random(&mut rng);
        let account_key = AccountKey::from(&RootIdentity::from(&entropy));

        // Set up the ledger to be seeded with multiple subaddresses paid
        let public_address0 = account_key.subaddress(0);
        let public_address1 = account_key.subaddress(1);
        let public_address2 = account_key.subaddress(2);
        let public_address3 = account_key.subaddress(3);

        let known_recipients: Vec<PublicAddress> = vec![
            public_address0.clone(),
            public_address1,
            public_address2,
            public_address3.clone(),
        ];
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        let account = service
            .import_account_from_legacy_root_entropy(
                hex::encode(entropy.bytes),
                None,
                None,
                None,
                "".to_string(),
                "".to_string(),
                false,
            )
            .expect("Could not import account entropy");

        let address = service
            .assign_address_for_account(&AccountID(account.id.clone()), None)
            .expect("Could not assign address");
        assert_eq!(address.subaddress_index, 2);

        let _account = manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &AccountID(account.id.to_string()),
            &logger,
        );

        let account_balance = service
            .get_balance_for_account(&AccountID(account.id))
            .expect("Could not get balance for account");
        let account_balance_pmob = account_balance.get(&Mob::ID).unwrap();
        // 3 accounts * 5_000 MOB * 12 blocks
        assert_eq!(account_balance_pmob.unspent, 180_000 * MOB as u128);
        // 5_000 MOB per txo, max 16 txos input - network fee
        // assert_eq!(
        //     account_balance_pmob.max_spendable,
        //     79999999600000000 as u128
        // );
        assert_eq!(account_balance_pmob.pending, 0);
        assert_eq!(account_balance_pmob.spent, 0);
        assert_eq!(account_balance_pmob.secreted, 0);
        assert_eq!(account_balance_pmob.orphaned, 60_000 * MOB as u128); // Public address 3

        let db_account_key: AccountKey =
            mc_util_serial::decode(&account.account_key).expect("Could not decode account key");
        let db_pub_address = db_account_key.default_subaddress();
        assert_eq!(db_pub_address, public_address0);
        let b58_pub_address =
            b58_encode_public_address(&db_pub_address).expect("Could not encode public address");
        let address_balance = service
            .get_balance_for_address(&b58_pub_address)
            .expect("Could not get balance for address");
        let address_balance_pmob = address_balance.get(&Mob::ID).unwrap();
        assert_eq!(address_balance_pmob.unspent, 60_000 * MOB as u128);
        // assert_eq!(
        //     address_balance_pmob.max_spendable,
        //     59999999600000000 as u128
        // );
        assert_eq!(address_balance_pmob.pending, 0);
        assert_eq!(address_balance_pmob.spent, 0);
        assert_eq!(address_balance_pmob.secreted, 0);
        assert_eq!(address_balance_pmob.orphaned, 0);

        let address_balance2 = service
            .get_balance_for_address(&address.public_address_b58)
            .expect("Could not get balance for address");
        let address_balance2_pmob = address_balance2.get(&Mob::ID).unwrap();
        assert_eq!(address_balance2_pmob.unspent, 60_000 * MOB as u128);
        // assert_eq!(
        //     address_balance2_pmob.max_spendable,
        //     59999999600000000 as u128
        // );
        assert_eq!(address_balance2_pmob.pending, 0);
        assert_eq!(address_balance2_pmob.spent, 0);
        assert_eq!(address_balance2_pmob.secreted, 0);
        assert_eq!(address_balance2_pmob.orphaned, 0);

        // Even though subaddress 3 has funds, we are not watching it, so we should get
        // an error.
        let b58_pub_address3 =
            b58_encode_public_address(&public_address3).expect("Could not encode public address");
        match service.get_balance_for_address(&b58_pub_address3) {
            Ok(_) => panic!("Should not get success getting balance for a non-assigned address"),
            Err(BalanceServiceError::Database(WalletDbError::AssignedSubaddressNotFound(_))) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }
}
