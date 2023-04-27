use crate::db::{models::NewSupportedMemoType, schema::supported_memo_types, Conn, WalletDbError};
use diesel::prelude::*;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum SupportedMemoTypes {
    AssociatedSender,
    AssociatedSenderWithPaymentIntentId,
    AssociateSenderWithPaymentRequestId,
}

// WARNING: changing which enum is associated with which value will mess with
// assumptions made in the database schema.
impl SupportedMemoTypes {
    pub fn db_classifier_id(&self) -> i32 {
        match *self {
            SupportedMemoTypes::AssociatedSender => 1,
            SupportedMemoTypes::AssociatedSenderWithPaymentIntentId => 2,
            SupportedMemoTypes::AssociateSenderWithPaymentRequestId => 3,
        }
    }
}

impl fmt::Display for SupportedMemoTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// TODO: once we figure out where the best place to call this (somewhere in
// startup), remove from the inserts from the migration
pub fn ensure_all_supported_memo_types_in_db(conn: Conn) -> Result<(), WalletDbError> {
    for memo_type in SupportedMemoTypes::iter() {
        let new_supported_type = NewSupportedMemoType {
            classifier: memo_type.db_classifier_id(),
            memo_type_name: &memo_type.to_string(),
        };

        // We don't want to blindly update the entry.
        // If the mapping of the memo type to its classifier ID needs to change, we
        // recommend a db migration where this table is recreated.
        // Also, on POSTGRES migration, use "on_conflict_do_nothing" instead of
        // "insert_or_ignore".
        diesel::insert_or_ignore_into(supported_memo_types::table)
            .values(&new_supported_type)
            .execute(conn)?;
    }
    Ok(())
}
