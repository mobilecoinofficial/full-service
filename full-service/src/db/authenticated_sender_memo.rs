// Copyright (c) 2023-2023 MobileCoin Inc.

use crate::db::{models::AuthenticatedSenderMemo, Conn, WalletDbError};
use diesel::RunQueryDsl;

impl AuthenticatedSenderMemo {
    pub fn list(conn: Conn) -> Result<Vec<AuthenticatedSenderMemo>, WalletDbError> {
        use crate::db::schema::authenticated_sender_memos;
        Ok(authenticated_sender_memos::table.load(conn)?)
    }
}
