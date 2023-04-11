// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use std::convert::{TryFrom, TryInto};

use ledger_mob::{DeviceHandle, Filters, LedgerHandle, LedgerProvider, Transport};

use mc_common::logger::global_log;
use mc_core::account::{ViewAccount, ViewSubaddress};
use mc_crypto_keys::RistrettoPublic;
use strum::Display;

use crate::service::models::tx_proposal::{InputTxo, TxProposal, UnsignedTxProposal};

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

async fn get_device_handle() -> Result<DeviceHandle<LedgerHandle>, HardwareWalletServiceError> {
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
) -> Result<Vec<TxoSynced>, HardwareWalletServiceError> {
    let device_handle = get_device_handle().await?;

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
) -> Result<TxProposal, HardwareWalletServiceError> {
    let mut device_handle = get_device_handle().await?;

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
