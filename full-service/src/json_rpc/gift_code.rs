// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the GiftCode object.

/// An gift code created by this wallet to share.
///
/// A gift code is a self-contained account which has been funded with MOB.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GiftCode {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The base58-encoded gift code string to share.
    pub gift_code: String,

    /// The entropy for the account in this gift code.
    pub entropy: String,

    /// The amount of MOB contained in the gift code account.
    pub value: String,

    /// A memo associated with this gift code.
    pub memo: String,
}
