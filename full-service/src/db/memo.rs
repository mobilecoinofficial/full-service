use crate::db::{models::TxoMemo, Conn, WalletDbError};
use byteorder::{BigEndian, ByteOrder};
use diesel::prelude::*;

pub fn get_memo_type_as_i16_from_bytes(memo_type_bytes: &[u8; 2]) -> i16 {
    BigEndian::read_i16(memo_type_bytes.as_slice())
}

impl TxoMemo {
    pub fn get(txo_id_hex: &str, memo_type: i16, conn: Conn) -> Result<TxoMemo, WalletDbError> {
        use crate::db::schema::txo_memos;

        let memo = match txo_memos::table
            .filter(txo_memos::txo_id.eq(txo_id_hex))
            .filter(txo_memos::memo_type.eq(memo_type))
            .get_result::<TxoMemo>(conn)
        {
            Ok(t) => t,
            Err(diesel::result::Error::NotFound) => {
                return Err(WalletDbError::TxoMemoNotFound(txo_id_hex.to_string()));
            }
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(memo)
    }
}
