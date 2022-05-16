// Copyright (c) 2020-2021 MobileCoin Inc.

//! A subaddress assigned to a particular contact for the purpose of tracking
//! funds received from that contact.

use crate::db::{
    models::{NewViewOnlySubaddress, ViewOnlySubaddress},
    view_only_account::ViewOnlyAccountID,
};

use mc_crypto_keys::RistrettoPublic;

use crate::db::{Conn, WalletDbError};
use diesel::prelude::*;

pub trait ViewOnlySubaddressModel {
    fn create(
        view_only_account_id: &ViewOnlyAccountID,
        public_address_b58: &str,
        subaddress_index: u64,
        comment: &str,
        public_spend_key: &RistrettoPublic,
        conn: &Conn,
    ) -> Result<String, WalletDbError>;

    /// Get the Subaddress for a given subaddress_b58
    fn get(public_address_b58: &str, conn: &Conn) -> Result<ViewOnlySubaddress, WalletDbError>;

    fn list_all(
        account_id_hex: &str,
        offset: Option<u64>,
        limit: Option<u64>,
        conn: &Conn,
    ) -> Result<Vec<ViewOnlySubaddress>, WalletDbError>;
}

impl ViewOnlySubaddressModel for ViewOnlySubaddress {
    fn create(
        view_only_account_id: &ViewOnlyAccountID,
        public_address_b58: &str,
        subaddress_index: u64,
        comment: &str,
        public_spend_key: &RistrettoPublic,
        conn: &Conn,
    ) -> Result<String, WalletDbError> {
        use crate::db::schema::view_only_subaddresses;

        let new_subaddress = NewViewOnlySubaddress {
            view_only_account_id_hex: &view_only_account_id.0,
            public_address_b58,
            subaddress_index: subaddress_index as i64,
            comment,
            public_spend_key: &public_spend_key.to_bytes(),
        };

        diesel::insert_into(view_only_subaddresses::table)
            .values(&new_subaddress)
            .execute(conn)?;

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
}
