// Copyright (c) 2020-2021 MobileCoin Inc.

//! DB impl for Memo models.

use crate::db::models::AuthenticatedSenderMemo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Memo {
    UnknownType,
    AuthenticatedSender(AuthenticatedSenderMemo),
}

pub trait AuthenticatedSenderMemoModel {
}

impl AuthenticatedSenderMemoModel for AuthenticatedSenderMemo {
}
