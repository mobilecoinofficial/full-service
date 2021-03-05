// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing addresses.

use crate::{db::models::AssignedSubaddress, error::WalletServiceError, service::WalletService};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;

use crate::db::assigned_subaddress::AssignedSubaddressModel;
use diesel::Connection;

/// Trait defining the ways in which the wallet can interact with and manage
/// addresses.
pub trait AddressService {
    /// Creates a new account with default values.
    fn assign_address_for_account(
        &self,
        account_id_hex: &str,
        metadata: Option<&str>,
        // FIXME: FS-32 - add "sync from block"
    ) -> Result<AssignedSubaddress, WalletServiceError>;

    fn get_all_addresses_for_account(
        &self,
        account_id_hex: &str,
    ) -> Result<Vec<AssignedSubaddress>, WalletServiceError>;
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
}
