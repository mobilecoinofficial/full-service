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

pub fn get_account_by_mnemonic(mnemonic: &str) -> Result<AccountInfo> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    get_account(mnemonic)
}

pub fn get_account_by_bip39_entropy(bip39_entropy: &str) -> Result<AccountInfo> {
    let mut entropy = [0u8; 32];
    hex::decode_to_slice(bip39_entropy, &mut entropy)?;
    let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)?;
    get_account(mnemonic)
}

fn get_account(mnemonic: Mnemonic) -> Result<AccountInfo> {
    let account = get_account_from_mnemonic(mnemonic);

    Ok(AccountInfo {
        view_private: account.view_private_key().clone(),
        spend_public: account.spend_public_key(),
        account_index: 0,
    })
}
pub fn sync_txos_by_mnemonic(mnemonic: &str, txos: Vec<TxoUnsynced>) -> Result<Vec<TxoSynced>> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    sync_txos(mnemonic, txos)
}

pub fn sync_txos_by_bip39_entropy(
    bip39_entropy: &str,
    txos: Vec<TxoUnsynced>,
) -> Result<Vec<TxoSynced>> {
    let mut entropy = [0u8; 32];
    hex::decode_to_slice(bip39_entropy, &mut entropy)?;
    let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)?;
    sync_txos(mnemonic, txos)
}

pub fn sync_txos(mnemonic: Mnemonic, txos: Vec<TxoUnsynced>) -> Result<Vec<TxoSynced>> {
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

pub fn sign_tx_with_mnemonic(
    mnemonic: &str,
    unsigned_tx_proposal: UnsignedTxProposal,
) -> Result<TxProposal> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    sign_tx(mnemonic, unsigned_tx_proposal)
}

pub fn sign_tx_with_bip39_entropy(
    bip39_entropy: &str,
    unsigned_tx_proposal: UnsignedTxProposal,
) -> Result<TxProposal> {
    let mut entropy = [0u8; 32];
    hex::decode_to_slice(bip39_entropy, &mut entropy)?;
    let mnemonic = Mnemonic::from_entropy(&entropy, Language::English)?;
    sign_tx(mnemonic, unsigned_tx_proposal)
}

pub fn sign_tx(mnemonic: Mnemonic, unsigned_tx_proposal: UnsignedTxProposal) -> Result<TxProposal> {
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
