// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use crate::service::models::tx_proposal::{InputTxo, TxProposal, UnsignedTxProposal};
use ledger_mob::{
    apdu::tx::{TxMemoSig, TxMemoSign},
    tx::TransactionHandle,
    Device, DeviceHandle, Filters, LedgerHandle, LedgerProvider, Transport,
};
use mc_account_keys::{PublicAddress, ShortAddressHash, ViewAccountKey};
use mc_common::logger::global_log;
use mc_core::{
    account::{RingCtAddress, ViewAccount, ViewSubaddress},
    subaddress::Subaddress,
};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_transaction_core::{tx::TxOut, NewMemoError};
use mc_transaction_extra::AuthenticatedMemoHmacSigner;
use mc_transaction_signer::types::TxoSynced;
use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    sync::Mutex,
    time::Duration,
};
use strum::Display;

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum HardwareWalletServiceError {
    NoHardwareWalletsFound,
    LedgerMob(ledger_mob::Error),
    PresignedRingsNotSupported,
    KeyImageNotFoundForSignedInput,
    RingCT(mc_transaction_core::ring_ct::Error),
    CryptoKeys(mc_crypto_keys::KeyError),
    CredentialMismatch,
}

impl From<mc_transaction_core::ring_ct::Error> for HardwareWalletServiceError {
    fn from(src: mc_transaction_core::ring_ct::Error) -> Self {
        HardwareWalletServiceError::RingCT(src)
    }
}

impl From<mc_crypto_keys::KeyError> for HardwareWalletServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        HardwareWalletServiceError::CryptoKeys(src)
    }
}

impl From<ledger_mob::Error> for HardwareWalletServiceError {
    fn from(src: ledger_mob::Error) -> Self {
        HardwareWalletServiceError::LedgerMob(src)
    }
}

pub async fn get_device_handle() -> Result<DeviceHandle<LedgerHandle>, HardwareWalletServiceError> {
    let mut ledger_provider = LedgerProvider::init().await;
    let devices = ledger_provider
        .list(Filters::Hid)
        .await
        .map_err(ledger_mob::Error::from)?;

    // Get the first device found, or error if none are found.
    let device = devices
        .first()
        .ok_or(HardwareWalletServiceError::NoHardwareWalletsFound)?;

    global_log::info!("Found devices: {:04x?}", devices);

    let handle = ledger_provider
        .connect(device.clone())
        .await
        .map_err(ledger_mob::Error::from)?
        .into();

    Ok(handle)
}

pub async fn sync_txos(
    unsynced_txos: Vec<(TxOut, u64)>,
    view_account: &ViewAccountKey,
) -> Result<Vec<TxoSynced>, HardwareWalletServiceError> {
    let mut device_handle = get_device_handle().await?;

    // Check device and requested accounts match
    let device_keys = device_handle.account_keys(0).await?;
    if device_keys.view_private_key() != view_account.view_private_key()
        || device_keys.spend_public_key() != view_account.spend_public_key()
    {
        return Err(HardwareWalletServiceError::CredentialMismatch);
    }

    let mut synced_txos = vec![];
    for unsynced_txo in unsynced_txos {
        let tx_public_key = (&unsynced_txo.0.public_key).try_into()?;
        let key_image = device_handle
            .key_image(0, unsynced_txo.1, tx_public_key)
            .await?;

        synced_txos.push(TxoSynced {
            tx_out_public_key: tx_public_key.into(),
            key_image,
        });
    }

    Ok(synced_txos)
}

pub async fn get_view_only_account_keys() -> Result<ViewAccount, HardwareWalletServiceError> {
    let mut device_handle = get_device_handle().await?;
    Ok(device_handle.account_keys(0).await?)
}

pub async fn get_view_only_subaddress_keys(
    subaddress_index: u64,
) -> Result<ViewSubaddress, HardwareWalletServiceError> {
    let mut device_handle = get_device_handle().await?;
    Ok(device_handle.subaddress_keys(0, subaddress_index).await?)
}

pub async fn sign_tx_proposal(
    unsigned_tx_proposal: UnsignedTxProposal,
    view_account: &ViewAccountKey,
) -> Result<TxProposal, HardwareWalletServiceError> {
    let mut device_handle = get_device_handle().await?;

    // Check device and requested accounts match
    let device_keys = device_handle.account_keys(0).await?;
    if device_keys.view_private_key() != view_account.view_private_key()
        || device_keys.spend_public_key() != view_account.spend_public_key()
    {
        return Err(HardwareWalletServiceError::CredentialMismatch);
    }

    // Sign transaction proposal
    global_log::debug!("Signing tx proposal with hardware device");
    let (tx, txos_synced) = device_handle
        .transaction(0, 60, unsigned_tx_proposal.unsigned_tx)
        .await?;

    let mut input_txos = vec![];

    for txo in unsigned_tx_proposal.unsigned_input_txos {
        let tx_out_public_key = RistrettoPublic::try_from(&txo.tx_out.public_key)?;
        let key_image = txos_synced
            .iter()
            .find(|txo| txo.tx_out_public_key == tx_out_public_key)
            .ok_or(HardwareWalletServiceError::KeyImageNotFoundForSignedInput)?
            .key_image;

        input_txos.push(InputTxo {
            tx_out: txo.tx_out,
            subaddress_index: txo.subaddress_index,
            key_image,
            amount: txo.amount,
        });
    }

    Ok(TxProposal {
        tx,
        input_txos,
        payload_txos: unsigned_tx_proposal.payload_txos,
        change_txos: unsigned_tx_proposal.change_txos,
    })
}

/// An implementation of AuthenticatedMemoHmacSigner that uses a hardware
/// wallet. This can be used with the RTHMemoBuilder to sign memos using a
/// hardware wallet.
pub struct HardwareWalletAuthenticatedMemoHmacSigner {
    /// The short address hash of the address that we are identifying as
    address_hash: ShortAddressHash,
    /// The subaddress index of the subaddress that we are identifying as
    subaddress_index: u64,
    /// The signer for the hardware wallet
    signer: Mutex<TransactionHandle<LedgerHandle>>,
    /// The timeout for requests to the hardware wallet
    request_timeout: Duration,
}

impl Debug for HardwareWalletAuthenticatedMemoHmacSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HardwareWalletAuthenticatedMemoHmacSigner({}, {})",
            self.address_hash, self.subaddress_index
        )
    }
}

impl HardwareWalletAuthenticatedMemoHmacSigner {
    pub fn new(
        identify_as_address: &PublicAddress,
        subaddress_index: u64,
    ) -> Result<Self, HardwareWalletServiceError> {
        let address_hash = ShortAddressHash::from(identify_as_address);

        futures::executor::block_on(async {
            let mut device_handle = get_device_handle().await?;

            let hardware_wallet_pub_addr = device_handle
                .account_keys(0)
                .await?
                .subaddress(subaddress_index);
            if *identify_as_address.view_public_key() != hardware_wallet_pub_addr.view_public_key()
                || *identify_as_address.spend_public_key()
                    != hardware_wallet_pub_addr.spend_public_key()
            {
                return Err(HardwareWalletServiceError::CredentialMismatch);
            }

            // Note: We pass num_memos = 1 and num_rings = 0.
            // The hardware wallet does not care about the number of memos we pass - this is
            // unused in the firmware. In reality we only sign zero or one memos per
            // transaction: the only memo that gets signed at the moment is the
            // AuthenticatedSenderMemo, and each transaction will have at most one of these.
            // We are not signing any rings and we will be abandoning the TransactionHandle
            // once we are done signing memos. A new TransactionHandle will be created by
            // `sign_tx_proposal` and that will have the correct number of rings.
            let signer = device_handle.transaction_handle(0, 1, 0, 60).await?;

            Ok(Self {
                address_hash,
                subaddress_index,
                signer: Mutex::new(signer),
                request_timeout: device_handle.request_timeout(),
            })
        })
    }
}

impl AuthenticatedMemoHmacSigner for HardwareWalletAuthenticatedMemoHmacSigner {
    fn sender_address_hash(&self) -> ShortAddressHash {
        self.address_hash
    }

    fn compute_category1_hmac(
        &self,
        receiving_subaddress_view_public_key: &RistrettoPublic,
        tx_out_public_key: &CompressedRistrettoPublic,
        memo_type_bytes: [u8; 2],
        memo_data_sans_hmac: &[u8; 48],
    ) -> Result<[u8; 16], NewMemoError> {
        let mut signer = self.signer.lock().unwrap();

        futures::executor::block_on(async {
            let mut buff = [0u8; 256];

            let key: RistrettoPublic = tx_out_public_key.try_into().map_err(|e| {
                NewMemoError::Other(format!("tx_out_pub_key conversion error: {}", e))
            })?;

            let tx_memo_sign = TxMemoSign::new(
                self.subaddress_index,
                key.into(),
                (*receiving_subaddress_view_public_key).into(),
                memo_type_bytes,
                *memo_data_sans_hmac,
            );

            debug!("Request memo sign");
            let r = signer
                .request::<TxMemoSig>(tx_memo_sign, &mut buff, self.request_timeout)
                .await
                .map_err(|e| {
                    NewMemoError::Other(format!("hardware wallet memo sign error: {}", e))
                })?;

            Ok(r.hmac)
        })
    }
}
