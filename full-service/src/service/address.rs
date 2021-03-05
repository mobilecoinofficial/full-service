// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing addresses.

use crate::{
    db::{assigned_subaddress::AssignedSubaddressModel, b58_decode, models::AssignedSubaddress},
    error::WalletServiceError,
    service::WalletService,
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

use diesel::Connection;

/// Trait defining the ways in which the wallet can interact with and manage
/// addresses.
pub trait AddressService {
    /// Creates a new address with default values.
    fn assign_address_for_account(
        &self,
        account_id_hex: &str,
        metadata: Option<&str>,
        // FIXME: FS-32 - add "sync from block"
    ) -> Result<AssignedSubaddress, WalletServiceError>;

    /// Gets all the addresses for the given account.
    fn get_all_addresses_for_account(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<AssignedSubaddress>, WalletServiceError>;

    /// Verifies whether an address can be decoded from b58.
    fn verify_address(&self, public_address: &str) -> Result<bool, WalletServiceError>;
}

impl<T, FPR> AddressService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn assign_address_for_account(
        &self,
        account_id_hex: &str,
        metadata: Option<&str>,
        // FIXME: WS-32 - add "sync from block"
    ) -> Result<AssignedSubaddress, WalletServiceError> {
        let conn = &self.wallet_db.get_conn()?;

        Ok(
            conn.transaction::<AssignedSubaddress, WalletServiceError, _>(|| {
                let (public_address_b58, _subaddress_index) =
                    AssignedSubaddress::create_next_for_account(
                        account_id_hex,
                        metadata.unwrap_or(""),
                        &conn,
                    )?;

                Ok(AssignedSubaddress::get(&public_address_b58, &conn)?)
            })?,
        )
    }

    fn get_all_addresses_for_account(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<AssignedSubaddress>, WalletServiceError> {
        Ok(AssignedSubaddress::list_all(
            account_id_hex,
            &self.wallet_db.get_conn()?,
        )?)
    }

    fn verify_address(&self, public_address: &str) -> Result<bool, WalletServiceError> {
        match b58_decode(public_address) {
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
        db::b58_encode,
        test_utils::{get_test_ledger, setup_wallet_service},
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
            b58_encode(&public_address).expect("Could not encode public address");

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

        // B58 encoding of public address should fail
        let account_key = AccountKey::random(&mut rng);
        let public_address = account_key.subaddress(rng.next_u64());
        let public_address_b58 =
            bs58::encode(mc_util_serial::encode(&public_address)).into_string();
        assert!(!service
            .verify_address(&public_address_b58)
            .expect("Could not verify address"));
    }
}
