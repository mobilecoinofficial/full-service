// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use std::convert::TryFrom;

use ledger_mob::{transport::GenericTransport, Connect, DeviceHandle, LedgerProvider};

use mc_common::logger::global_log;
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

async fn get_device_handle() -> Result<DeviceHandle<GenericTransport>, HardwareWalletServiceError> {
    let ledger_provider = LedgerProvider::new().unwrap();
    let devices = ledger_provider.list_devices(ledger_mob::Filter::Hid).await;

    if devices.is_empty() {
        return Err(HardwareWalletServiceError::NoHardwareWalletsFound);
    }

    global_log::info!("Found devices: {:04x?}", devices);

    // Connect to the first device
    //
    // This CBB - we should iterate through each device if signing fails on the
    // current one and more are available
    Ok(Connect::<GenericTransport>::connect(&ledger_provider, &devices[0]).await?)
}

pub async fn sign_tx_proposal(
    unsigned_tx_proposal: UnsignedTxProposal,
) -> Result<TxProposal, HardwareWalletServiceError> {
    let device_handle = get_device_handle().await?;

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
