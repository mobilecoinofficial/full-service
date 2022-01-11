// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing addresses.

use crate::{
    db::{
        account::AccountID, assigned_subaddress::AssignedSubaddressModel,
        models::AssignedSubaddress, WalletDbError,
    },
    service::WalletService,
    util::b58::b58_decode_public_address,
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

use diesel::Connection;
use displaydoc::Display;

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AddressServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),
}

impl From<WalletDbError> for AddressServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for AddressServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// addresses.
pub trait AddressService {
    /// Creates a new address with default values.
    fn assign_address_for_account(
        &self,
        account_id: &AccountID,
        metadata: Option<&str>,
        // FIXME: FS-32 - add "sync from block"
    ) -> Result<AssignedSubaddress, AddressServiceError>;

    /// Gets all the addresses for the given account.
    fn get_addresses_for_account(
        &self,
        account_id: &AccountID,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<AssignedSubaddress>, AddressServiceError>;

    fn get_address_for_account(
        &self,
        account_id: &AccountID,
        index: i64,
    ) -> Result<AssignedSubaddress, AddressServiceError>;

    /// Verifies whether an address can be decoded from b58.
    fn verify_address(&self, public_address: &str) -> Result<bool, AddressServiceError>;
}

impl<T, FPR> AddressService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn assign_address_for_account(
        &self,
        account_id: &AccountID,
        metadata: Option<&str>,
    ) -> Result<AssignedSubaddress, AddressServiceError> {
        let conn = &self.wallet_db.get_conn()?;
        conn.transaction(|| {
            let (public_address_b58, _subaddress_index) =
                AssignedSubaddress::create_next_for_account(
                    &account_id.to_string(),
                    metadata.unwrap_or(""),
                    &self.ledger_db,
                    conn,
                )?;

            Ok(AssignedSubaddress::get(&public_address_b58, conn)?)
        })
    }

    fn get_addresses_for_account(
        &self,
        account_id: &AccountID,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<AssignedSubaddress>, AddressServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| {
            Ok(AssignedSubaddress::list_all(
                &account_id.to_string(),
                offset,
                limit,
                &conn,
            )?)
        })
    }

    fn get_address_for_account(
        &self,
        account_id: &AccountID,
        index: i64,
    ) -> Result<AssignedSubaddress, AddressServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| {
            Ok(AssignedSubaddress::get_for_account_by_index(
                &account_id.to_string(),
                index,
                &conn,
            )?)
        })
    }

    fn verify_address(&self, public_address: &str) -> Result<bool, AddressServiceError> {
        match b58_decode_public_address(public_address) {
            Ok(_a) => {
                log::info!(self.logger, "Verified address {:?}", public_address);
                Ok(true)
            }
            Err(e) => {
                log::info!(
                    self.logger,
                    "Address did not verify {:?}: {:?}",
                    public_address,
                    e
                );
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::{get_test_ledger, setup_wallet_service},
        util::b58::b58_encode_public_address,
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    // A properly encoded address should verify.
    #[test_with_logger]
    fn test_verify_address_succeeds(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        let account_key = AccountKey::random(&mut rng);
        let public_address = account_key.subaddress(rng.next_u64());
        let public_address_b58 =
            b58_encode_public_address(&public_address).expect("Could not encode public address");

        assert!(service
            .verify_address(&public_address_b58)
            .expect("Could not verify address"));
    }

    // An improperly encoded address should fail.
    #[test_with_logger]
    fn test_verify_address_fails(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Empty string should fail
        let public_address_b58 = "";
        assert!(!service
            .verify_address(&public_address_b58)
            .expect("Could not verify address"));

        // Basic B58 encoding of public address should fail (should include a checksum)
        let account_key = AccountKey::random(&mut rng);
        let public_address = account_key.subaddress(rng.next_u64());
        let public_address_b58 =
            bs58::encode(mc_util_serial::encode(&public_address)).into_string();
        assert!(!service
            .verify_address(&public_address_b58)
            .expect("Could not verify address"));
    }
}
