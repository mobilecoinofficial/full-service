// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use std::convert::{TryInto, TryFrom};

use ledger_mob::{
    Connect, DeviceHandle, Filter, LedgerProvider,
    transport::{GenericTransport},
};

use mc_common::logger::global_log;
use mc_core::{account::ViewAccount, keys::TxOutPublic};
use mc_crypto_keys::RistrettoPublic;
use mc_transaction_core::tx::TxOut;
use mc_transaction_signer::types::TxoSynced;

use strum::Display;

use crate::service::models::tx_proposal::{InputTxo, TxProposal, UnsignedTxProposal};

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum HardwareWalletServiceError {
    NoHardwareWalletsFound,
    Ledger(ledger_mob::Error),
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

async fn get_device_handle() -> Result<DeviceHandle<GenericTransport>, HardwareWalletServiceError>
{
    let ledger_provider = LedgerProvider::new().unwrap();
    let devices: Vec<_> = ledger_provider.list_devices(Filter::Any).await;

    if devices.len() == 0 {
        return Err(HardwareWalletServiceError::NoHardwareWalletsFound);
    }

    global_log::info!("Found devices: {:04x?}", devices);

    // Connect to the first device
    //
    // This CBB - we should iterate through each device if signing fails on the
    // current one and more are available
    Ok(
        Connect::<GenericTransport>::connect(&ledger_provider, &devices[0])
            .await
            .map_err(HardwareWalletServiceError::Ledger)?
    )
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
            .await
            .map_err(HardwareWalletServiceError::Ledger)?;

        synced_txos.push(TxoSynced {
            tx_out_public_key: tx_public_key.into(),
            key_image,
        });
    }

    Ok(synced_txos)
}

pub async fn get_view_only_account_keys() -> Result<ViewAccount, HardwareWalletServiceError> {
    let device_handle = get_device_handle().await?;
    Ok(device_handle
        .account_keys(0)
        .await
        .map_err(HardwareWalletServiceError::Ledger)?)
}

pub async fn sign_tx_proposal(
    unsigned_tx_proposal: UnsignedTxProposal,
) -> Result<TxProposal, HardwareWalletServiceError> {
    let device_handle = get_device_handle().await?;

    // Start device transaction
    global_log::info!("Starting transaction signing");
    let (tx, tx_outs) = device_handle
        .transaction(0, 90, unsigned_tx_proposal.unsigned_tx)
        .await
        .map_err(HardwareWalletServiceError::Ledger)?;

    let mut input_txos = vec![];
    for txo in unsigned_tx_proposal.unsigned_input_txos {
        let tx_out_public_key = RistrettoPublic::try_from(&txo.tx_out.public_key)?;
        let key_image = tx_outs
            .iter()
            .find(|txo| txo.tx_out_public_key == TxOutPublic::from(tx_out_public_key))
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
