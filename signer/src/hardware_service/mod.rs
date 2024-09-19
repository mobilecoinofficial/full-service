// Copyright (c) 2020-2023 MobileCoin Inc.

use anyhow::{anyhow, Result};
use mc_account_keys::ViewAccountKey;
use mc_transaction_signer::types::{AccountInfo, TxoSynced, TxoUnsynced};
use mc_full_service::{
    service::{
        hardware_wallet::{get_device_handle, get_view_only_account_keys, sign_tx_proposal},
        models::tx_proposal::{UnsignedTxProposal, TxProposal},
    },
    db::account::AccountID,
};

pub mod api;

pub async fn get_account() -> Result<AccountInfo> {
    let view_account = get_view_only_account_keys()
        .await.map_err(|e| anyhow!(e))?;
    Ok(AccountInfo {
        view_private: view_account.view_private_key().clone(),
        spend_public: view_account.spend_public_key().clone(),
        account_index: 0,
    })
}

pub fn get_account_id(account_info: AccountInfo) -> String {
    let view_account_keys = ViewAccountKey::new(
        *account_info.view_private.as_ref(), 
        *account_info.spend_public.as_ref(),
    );
    return AccountID::from(&view_account_keys).to_string();
}   

pub async fn sync_txos(account_id: String, txos: Vec<TxoUnsynced>) -> Result<Vec<TxoSynced>> {
    let account = get_account().await?;
    let hardware_account_id = get_account_id(account);
    if account_id != hardware_account_id {
        return Err(anyhow!("Hardware Credentials Mismatch"));
    }

    let mut device_handle = get_device_handle().await.map_err(|e|anyhow!(e))?;
    let mut synced: Vec<TxoSynced> = Vec::new();

    println!("Generating key images for {} txos", txos.len());
    let mut i:u64 = 0;
    for TxoUnsynced {
        subaddress,
        tx_out_public_key,
    } in txos
    {
        let public_key = *tx_out_public_key.as_ref();
        let key_image = device_handle.key_image(0, subaddress, public_key).await?;

        synced.push(TxoSynced {
            tx_out_public_key,
            key_image,
        });
        i += 1;
        if (i % 100) == 0 {
            println!("Generated key images for {} txos", i);
        }
    }

    Ok(synced)
}

pub async fn sign_tx(account_id: String, unsigned_tx_proposal: UnsignedTxProposal) -> Result<TxProposal> {
    let account = get_account().await?;
    let hardware_account_id = get_account_id(account.clone());
    if account_id != hardware_account_id {
        return Err(anyhow!("Hardware Credentials Mismatch"));
    }

    let account_key = ViewAccountKey::new(
        *account.view_private.as_ref(),
        *account.spend_public.as_ref(),
    );

    let tx_proposal = sign_tx_proposal(unsigned_tx_proposal, &account_key).await.map_err(|e|anyhow!(e))?;

    Ok(tx_proposal)
}