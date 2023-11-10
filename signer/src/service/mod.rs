// Copyright (c) 2020-2023 MobileCoin Inc.

use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic, MnemonicType};
use mc_account_keys::AccountKey;
use mc_core::{account::Account, slip10::Slip10KeyGenerator};
use mc_full_service::service::models::tx_proposal::{TxProposal, UnsignedTxProposal};
use mc_transaction_signer::{
    traits::KeyImageComputer,
    types::{AccountInfo, TxoSynced, TxoUnsynced},
};

pub mod api;

pub fn create_account() -> (Mnemonic, AccountInfo) {
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
    let account = get_account_from_mnemonic(mnemonic.clone());
    let account_info = AccountInfo {
        view_private: account.view_private_key().clone(),
        spend_public: account.spend_public_key(),
        account_index: 0,
    };

    (mnemonic, account_info)
}

pub fn get_account(mnemonic: &str) -> Result<AccountInfo> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    let account = get_account_from_mnemonic(mnemonic);

    Ok(AccountInfo {
        view_private: account.view_private_key().clone(),
        spend_public: account.spend_public_key(),
        account_index: 0,
    })
}

pub fn sync_txos(mnemonic: &str, txos: Vec<TxoUnsynced>) -> Result<Vec<TxoSynced>> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    let account = get_account_from_mnemonic(mnemonic);

    let mut synced: Vec<TxoSynced> = Vec::new();
    for TxoUnsynced {
        subaddress,
        tx_out_public_key,
    } in txos
    {
        let key_image = account.compute_key_image(subaddress, &tx_out_public_key)?;

        synced.push(TxoSynced {
            tx_out_public_key,
            key_image,
        });
    }

    Ok(synced)
}

pub fn sign_tx(mnemonic: &str, unsigned_tx_proposal: UnsignedTxProposal) -> Result<TxProposal> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    let account = get_account_from_mnemonic(mnemonic);
    let account_key = AccountKey::new(
        account.spend_private_key().as_ref(),
        account.view_private_key().as_ref(),
    );

    unsigned_tx_proposal
        .sign_with_local_signer(&account_key)
        .map_err(|e| anyhow!(e))
}

fn get_account_from_mnemonic(mnemonic: Mnemonic) -> Account {
    let slip_10_key = mnemonic.derive_slip10_key(0);
    Account::from(&slip_10_key)
}
