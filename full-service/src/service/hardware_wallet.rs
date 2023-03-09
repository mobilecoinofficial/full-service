// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing ledger materials and MobileCoin protocol objects.

use std::convert::TryFrom;

use ledger_mob::{
    transport::TransportNativeHID, tx::TxConfig, Connect, DeviceHandle, LedgerProvider,
};

use mc_account_keys::ViewAccountKey;
use mc_common::logger::global_log;
use mc_core::keys::TxOutPublic;
use mc_crypto_keys::RistrettoPublic;
use mc_crypto_rand::rand_core::OsRng;
use mc_transaction_core::{ring_ct::InputRing, tx::Tx};
use mc_transaction_signer::types::TxoSynced;
use mc_transaction_summary::verify_tx_summary;

use strum::Display;

use crate::service::models::tx_proposal::{InputTxo, TxProposal, UnsignedTxProposal};

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum HardwareWalletServiceError {
    NoHardwareWalletsFound,
    LedgerHID,
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

async fn get_device_handle() -> Result<DeviceHandle<TransportNativeHID>, HardwareWalletServiceError>
{
    let ledger_provider = LedgerProvider::new().unwrap();
    let devices: Vec<_> = ledger_provider.list_devices().collect();

    if devices.len() == 0 {
        return Err(HardwareWalletServiceError::NoHardwareWalletsFound);
    }

    global_log::info!("Found devices: {:04x?}", devices);

    // Connect to the first device
    //
    // This CBB - we should iterate through each device if signing fails on the
    // current one and more are available
    Ok(
        Connect::<TransportNativeHID>::connect(&ledger_provider, &devices[0])
            .await
            .map_err(|_| HardwareWalletServiceError::LedgerHID)?,
    )
}

pub async fn sign_tx_proposal(
    unsigned_tx_proposal: UnsignedTxProposal,
    view_account_key: &ViewAccountKey,
) -> Result<TxProposal, HardwareWalletServiceError> {
    let device_handle = get_device_handle().await?;

    // Start device transaction
    global_log::info!("Starting transaction signing");
    let signer = device_handle
        .transaction(TxConfig {
            account_index: 0,
            num_memos: 0,
            num_rings: unsigned_tx_proposal.unsigned_tx.rings.len(),
        })
        .await
        .map_err(|_| HardwareWalletServiceError::LedgerHID)?;

    // TODO: sign any memos

    // Build the digest for ring signing
    // TODO: this will move when TxSummary is complete
    global_log::info!("Building TX digest");
    let (signing_data, summary, unblinding_data, digest) = unsigned_tx_proposal
        .unsigned_tx
        .get_signing_data(&mut OsRng {})?;

    // match unblinding_data {
    //     None => {
    //         debug!("Setting tx message");
    //         signer
    //             .set_message(&digest.0)
    //             .await
    //             .map_err(|_| HardwareWalletServiceError::LedgerHID)?;
    //     }
    //     Some(unblinding_data) => {
    debug!("Loading tx summary");
    signer
        .set_tx_summary(
            unsigned_tx_proposal.unsigned_tx.block_version,
            &digest.0,
            &summary,
            &unblinding_data,
        )
        .await
        .map_err(|_| HardwareWalletServiceError::LedgerHID)?;

    // TODO: check signing_data matches computed mlsag_signing_digest
    let mut m = [0u8; 32];
    m.copy_from_slice(&digest.0[..]);

    let (expected_digest, _report) = verify_tx_summary(
        &m,
        &summary,
        &unblinding_data,
        *view_account_key.view_private_key(),
    )
    .map_err(|_| HardwareWalletServiceError::LedgerHID)?;

    assert_eq!(
        &expected_digest[..],
        &signing_data.mlsag_signing_digest[..],
        "summary generated digest mismatch"
    );
    //     }
    // }

    // Await user input
    global_log::info!("Awaiting user confirmation");
    signer
        .await_approval(120)
        .await
        .map_err(|_| HardwareWalletServiceError::LedgerHID)?;

    // Execute signing (signs rings etc.)
    global_log::info!("Executing signing operation");
    let signature = signing_data.sign(
        &unsigned_tx_proposal.unsigned_tx.rings,
        &signer,
        &mut OsRng {},
    )?;

    signer
        .complete()
        .await
        .map_err(|_| HardwareWalletServiceError::LedgerHID)?;

    // Map key images to real inputs via public key
    let mut txos = vec![];
    for (i, r) in unsigned_tx_proposal.unsigned_tx.rings.iter().enumerate() {
        let tx_out_public_key = match r {
            InputRing::Signable(r) => r.members[r.real_input_index].public_key,
            InputRing::Presigned(_) => {
                return Err(HardwareWalletServiceError::PresignedRingsNotSupported)
            }
        };

        txos.push(TxoSynced {
            tx_out_public_key: TxOutPublic::from(RistrettoPublic::try_from(&tx_out_public_key)?),
            key_image: signature.ring_signatures[i].key_image,
        });
    }

    let mut input_txos = vec![];

    for txo in unsigned_tx_proposal.unsigned_input_txos {
        let tx_out_public_key = RistrettoPublic::try_from(&txo.tx_out.public_key)?;
        let key_image = txos
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
        tx: Tx {
            prefix: unsigned_tx_proposal.unsigned_tx.tx_prefix.clone(),
            signature,
            fee_map_digest: vec![],
        },
        input_txos,
        payload_txos: unsigned_tx_proposal.payload_txos,
        change_txos: unsigned_tx_proposal.change_txos,
    })
}
