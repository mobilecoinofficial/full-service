// Copyright (c) 2020 MobileCoin Inc.

//! Service for managing gift codes.
//!
//! Gift codes are onetime accounts that contain a single TXO. They provide
//! a means to send MOB in a way that can be "claimed," for example, by pasting
//! a QR code for a gift code into a group chat, and the first person to
//! consume the gift code claims the MOB.

use crate::{error::WalletServiceError, service::WalletService};
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_connection::FogPubkeyResolver;

pub trait GiftCodeService {
    /// Builds a new gift code.
    ///
    /// Building a gift code requires the following steps:
    ///  1. Create a new account to receive the funds
    ///  2. Send a transaction to the new account
    ///  3. Wait for the transaction to land
    ///  4. Package the required information into a b58-encoded string
    fn build_gift_code(&self) -> Result<String, WalletServiceError>;

    fn check_gift_code_status(&self) -> Result<(), WalletServiceError>;

    fn consume_gift_code(&self) -> Result<(), WalletServiceError>;
}

impl<T, FPR> GiftCodeService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn build_gift_code(&self) -> Result<String, WalletServiceError> {
        Ok("".to_string())
    }

    fn check_gift_code_status(&self) -> Result<(), WalletServiceError> {
        Ok(())
    }

    fn consume_gift_code(&self) -> Result<(), WalletServiceError> {
        Ok(())
    }
}
