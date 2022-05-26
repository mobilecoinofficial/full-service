// Copyright (c) 2020-2021 MobileCoin Inc.

//! A subaddress assigned to a particular contact for the purpose of tracking
//! funds received from that contact.

use crate::db::{
    models::{NewViewOnlySubaddress, ViewOnlyAccount, ViewOnlySubaddress, ViewOnlyTxo},
    view_only_txo::ViewOnlyTxoModel,
};

use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_transaction_core::{onetime_keys::recover_public_subaddress_spend_key, tx::TxOut};

use crate::db::{Conn, WalletDbError};
use diesel::prelude::*;
use std::convert::TryFrom;

pub trait ViewOnlySubaddressModel {
    fn create(
        account: &ViewOnlyAccount,
        public_address_b58: &str,
        subaddress_index: u64,
        comment: &str,
        public_spend_key: &RistrettoPublic,
        conn: &Conn,
    ) -> Result<String, WalletDbError>;

    /// Get the Subaddress for a given subaddress_b58
    fn get(public_address_b58: &str, conn: &Conn) -> Result<ViewOnlySubaddress, WalletDbError>;

    fn get_for_account_by_index(
        account_id: &str,
        subaddress_index: u64,
        conn: &Conn,
    ) -> Result<ViewOnlySubaddress, WalletDbError>;

    fn list_all(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlySubaddress>, WalletDbError>;

    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError>;
}

impl ViewOnlySubaddressModel for ViewOnlySubaddress {
    fn create(
        account: &ViewOnlyAccount,
        public_address_b58: &str,
        subaddress_index: u64,
        comment: &str,
        public_spend_key: &RistrettoPublic,
        conn: &Conn,
    ) -> Result<String, WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        let new_subaddress = NewViewOnlySubaddress {
            view_only_account_id_hex: &account.account_id_hex,
            public_address_b58,
            subaddress_index: subaddress_index as i64,
            comment,
            public_spend_key: &public_spend_key.to_bytes(),
        };

        diesel::insert_into(view_only_subaddresses::table)
            .values(&new_subaddress)
            .execute(conn)?;

        let orphaned_txos_with_key_images =
            ViewOnlyTxo::list_orphaned_with_key_images(&account.account_id_hex, conn)?;

        let view_private_key: RistrettoPrivate = mc_util_serial::decode(&account.view_private_key)?;

        for txo in orphaned_txos_with_key_images {
            let tx_out: TxOut = mc_util_serial::decode(&txo.txo)?;

            let txo_subaddress_spk = recover_public_subaddress_spend_key(
                &view_private_key,
                &RistrettoPublic::try_from(&tx_out.target_key)?,
                &RistrettoPublic::try_from(&tx_out.public_key)?,
            );

            if txo_subaddress_spk == *public_spend_key {
                txo.update_subaddress_index(subaddress_index, conn)?;
            }
        }

        Ok(public_address_b58.to_string())
    }

    fn get(public_address_b58: &str, conn: &Conn) -> Result<ViewOnlySubaddress, WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        let subaddress: ViewOnlySubaddress = match view_only_subaddresses::table
            .filter(view_only_subaddresses::public_address_b58.eq(public_address_b58))
            .get_result::<ViewOnlySubaddress>(conn)
        {
            Ok(t) => t,
            // Match on NotFound to get a more informative NotFound Error
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::AssignedSubaddressNotFound(
                    public_address_b58.to_string(),
                ));
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(subaddress)
    }

    fn get_for_account_by_index(
        account_id: &str,
        subaddress_index: u64,
        conn: &Conn,
    ) -> Result<ViewOnlySubaddress, WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        let subaddress: ViewOnlySubaddress = view_only_subaddresses::table
            .filter(view_only_subaddresses::view_only_account_id_hex.eq(account_id))
            .filter(view_only_subaddresses::subaddress_index.eq(subaddress_index as i64))
            .get_result::<ViewOnlySubaddress>(conn)?;

        Ok(subaddress)
    }

    fn list_all(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlySubaddress>, WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        let addresses_query = view_only_subaddresses::table
            .filter(view_only_subaddresses::view_only_account_id_hex.eq(account_id_hex))
            .select(view_only_subaddresses::all_columns);

        let subaddresses: Vec<ViewOnlySubaddress> = if let (Some(o), Some(l)) = (offset, limit) {
            addresses_query
                .offset(o as i64)
                .limit(l as i64)
                .load(conn)?
        } else {
            addresses_query.load(conn)?
        };

        Ok(subaddresses)
    }

    fn delete_all_for_account(account_id_hex: &str, conn: &Conn) -> Result<(), WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        diesel::delete(
            view_only_subaddresses::table
                .filter(view_only_subaddresses::view_only_account_id_hex.eq(account_id_hex)),
        )
        .execute(conn)?;
        Ok(())
    }
}
