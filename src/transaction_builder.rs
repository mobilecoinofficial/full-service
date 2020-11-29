// Copyright (c) 2020 MobileCoin Inc.

//! A builder for transactions from the wallet. Note that we have a TransactionBuilder
//! in the MobileCoin transaction crate, but this is a lower level of building, once you
//! have already obtained all of the materials that go into a transaction.
//!
//! This module, on the other hand, builds a transaction within the context of the wallet.

use crate::db::WalletDb;
use crate::error::WalletTransactionBuilderError;
use crate::models::{Account, Txo};
use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::logger::{log, Logger};
use mc_common::{HashMap, HashSet};
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_connection::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_mobilecoind::payments::{Outlay, TxProposal};
use mc_mobilecoind::UnspentTxOut;
use mc_transaction_core::constants::{MAX_INPUTS, MINIMUM_FEE, RING_SIZE};
use mc_transaction_core::onetime_keys::recover_onetime_private_key;
use mc_transaction_core::ring_signature::KeyImage;
use mc_transaction_core::tx::{TxOut, TxOutMembershipProof};
use mc_transaction_core::BlockIndex;
use mc_transaction_std::{InputCredentials, TransactionBuilder};
use rand::Rng;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::sync::Arc;

/// Default number of blocks used for calculating transaction tombstone block number.
// TODO support for making this configurable
pub const DEFAULT_NEW_TX_BLOCK_ATTEMPTS: u64 = 50;

pub struct WalletTransactionBuilder<FPR: FogPubkeyResolver + Send + Sync + 'static> {
    account_id_hex: String,
    wallet_db: WalletDb,
    ledger_db: LedgerDB,
    transaction_builder: TransactionBuilder,
    inputs: Vec<Txo>,
    outlays: Vec<(PublicAddress, u64)>,
    tombstone: u64,
    /// Fog pub key resolver, used when constructing outputs to fog recipients.
    fog_pubkey_resolver: Option<Arc<FPR>>,
    logger: Logger,
}

impl<FPR: FogPubkeyResolver + Send + Sync + 'static> WalletTransactionBuilder<FPR> {
    pub fn new(
        account_id_hex: String,
        wallet_db: WalletDb,
        ledger_db: LedgerDB,
        fog_pubkey_resolver: Option<Arc<FPR>>,
        logger: Logger,
    ) -> Self {
        let transaction_builder = TransactionBuilder::new();
        WalletTransactionBuilder {
            account_id_hex,
            wallet_db,
            ledger_db,
            transaction_builder,
            inputs: vec![],
            outlays: vec![],
            tombstone: 0,
            fog_pubkey_resolver,
            logger,
        }
    }

    /// Sets inputs to the txos associated with the given txo_ids. Only unspent txos are included.
    pub fn set_txos(
        &mut self,
        input_txo_ids: &Vec<String>,
    ) -> Result<(), WalletTransactionBuilderError> {
        let txos = self
            .wallet_db
            .select_txos_by_id(&self.account_id_hex, input_txo_ids)?;
        let unspent: Vec<Txo> = txos
            .iter()
            .filter(|(_txo, status)| status.txo_status == "unspent")
            .map(|(t, _s)| t.clone())
            .collect();
        self.inputs = unspent;
        Ok(())
    }

    pub fn select_txos(
        &mut self,
        max_spendable_value: Option<u64>,
    ) -> Result<(), WalletTransactionBuilderError> {
        let value = self.outlays.iter().map(|(_r, v)| v).sum();

        // Select utxos for the given value
        let max_spendable = if let Some(msv) = max_spendable_value {
            msv as i64
        } else {
            i64::MAX
        };
        let mut txos: Vec<Txo> = self
            .wallet_db
            .select_unspent_txos_for_value(&self.account_id_hex, max_spendable)?;

        // The maximum spendable is limited by the maximal number of inputs we can use.
        // FIXME: is this really doing what we want? Just because the MAX_INPUTS smallest are < value does
        //        not mean that the whole wallet is fragmented.
        let max_spendable_amount = txos
            .iter()
            .take(MAX_INPUTS as usize)
            .map(|utxo| utxo.value as u64)
            .sum();
        if value > max_spendable_amount {
            // See if we merged the UTXOs we would be able to spend this amount.
            let total_utxos_value: u64 = txos.iter().map(|utxo| utxo.value as u64).sum();
            if total_utxos_value >= value {
                return Err(WalletTransactionBuilderError::InsufficientFundsFragmentedTxos);
            } else {
                return Err(WalletTransactionBuilderError::InsufficientFunds(format!(
                    "Max spendable amount: {:?}",
                    max_spendable_amount
                )));
            }
        }

        // FIXME: can we do this more elegantly with a DB query?
        // Choose utxos to spend.
        let mut selected_utxos: Vec<Txo> = Vec::new();
        let mut total = 0;
        loop {
            if total >= value {
                break;
            }

            // Grab the next (smallest) utxo, in order to opportunistically sweep up dust
            let next_utxo = txos
                .pop()
                .ok_or(WalletTransactionBuilderError::InsufficientFunds(
                    "Not enough utxos".to_string(),
                ))?;
            selected_utxos.push(next_utxo.clone());
            total += next_utxo.value as u64;

            // Cap at maximum allowed inputs.
            if selected_utxos.len() > MAX_INPUTS as usize {
                // Remove the lowest utxo.
                selected_utxos.remove(0);
            }
        }

        // Sanity.
        assert!(!selected_utxos.is_empty());
        assert!(selected_utxos.len() <= MAX_INPUTS as usize);

        // Set our inputs
        self.inputs = selected_utxos;

        Ok(())
    }

    pub fn add_recipient(
        &mut self,
        recipient: PublicAddress,
        value: u64,
    ) -> Result<(), WalletTransactionBuilderError> {
        self.outlays.push((recipient, value));
        Ok(())
    }

    pub fn set_fee(&mut self, fee: u64) -> Result<(), WalletTransactionBuilderError> {
        self.transaction_builder
            .set_fee(std::cmp::min(MINIMUM_FEE, fee));
        Ok(())
    }

    pub fn set_tombstone(&mut self, tombstone: u64) -> Result<(), WalletTransactionBuilderError> {
        let tombstone_block = if tombstone > 0 {
            tombstone
        } else {
            let num_blocks_in_ledger = self.ledger_db.num_blocks()?;
            num_blocks_in_ledger + DEFAULT_NEW_TX_BLOCK_ATTEMPTS
        };
        self.tombstone = tombstone_block;
        Ok(())
    }

    /// Consumes self
    pub fn build(mut self) -> Result<TxProposal, WalletTransactionBuilderError> {
        let account: Account = self.wallet_db.get_account(&self.account_id_hex)?;
        let from_account_key: AccountKey = mc_util_serial::decode(&account.encrypted_account_key)?;

        // Get membership proofs for our inputs
        let indexes = self
            .inputs
            .iter()
            .map(|utxo| {
                let txo: TxOut = mc_util_serial::decode(&utxo.txo)?;
                self.ledger_db.get_tx_out_index_by_hash(&txo.hash())
            })
            .collect::<Result<Vec<u64>, mc_ledger_db::Error>>()?;
        let proofs = self.ledger_db.get_tx_out_proof_of_memberships(&indexes)?;

        let inputs_and_proofs: Vec<(Txo, TxOutMembershipProof)> = self
            .inputs
            .clone()
            .into_iter()
            .zip(proofs.into_iter())
            .collect();

        let excluded_tx_out_indices: Vec<u64> = inputs_and_proofs
            .iter()
            .map(|(utxo, _membership_proof)| {
                let txo: TxOut = mc_util_serial::decode(&utxo.txo)?;
                self.ledger_db
                    .get_tx_out_index_by_hash(&txo.hash())
                    .map_err(WalletTransactionBuilderError::LedgerDB)
            })
            .collect::<Result<Vec<u64>, WalletTransactionBuilderError>>()?;

        let rings = self.get_rings(inputs_and_proofs.len(), &excluded_tx_out_indices)?;

        if rings.len() != inputs_and_proofs.len() {
            return Err(WalletTransactionBuilderError::RingSizeMismatch);
        }

        if self.outlays.is_empty() {
            return Err(WalletTransactionBuilderError::NoRecipient);
        }

        // Unzip each vec of tuples into a tuple of vecs.
        let mut rings_and_proofs: Vec<(Vec<TxOut>, Vec<TxOutMembershipProof>)> = rings
            .into_iter()
            .map(|tuples| tuples.into_iter().unzip())
            .collect();

        // Add inputs to the tx.
        for (utxo, proof) in inputs_and_proofs.iter() {
            let db_tx_out = mc_util_serial::decode(&utxo.txo)?;
            let (mut ring, mut membership_proofs) = rings_and_proofs
                .pop()
                .ok_or_else(|| WalletTransactionBuilderError::RingsAndProofsEmpty)?;
            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            // Add the input to the ring.
            let position_opt = ring.iter().position(|txo| *txo == db_tx_out);
            let real_key_index = match position_opt {
                Some(position) => {
                    // The input is already present in the ring.
                    // This could happen if ring elements are sampled randomly from the ledger.
                    position
                }
                None => {
                    // The input is not already in the ring.
                    if ring.is_empty() {
                        // Append the input and its proof of membership.
                        ring.push(db_tx_out.clone());
                        membership_proofs.push(proof.clone());
                    } else {
                        // Replace the first element of the ring.
                        ring[0] = db_tx_out.clone();
                        membership_proofs[0] = proof.clone();
                    }
                    // The real input is always the first element. This is safe because TransactionBuilder
                    // sorts each ring.
                    0
                }
            };

            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            let public_key = RistrettoPublic::try_from(&db_tx_out.public_key).unwrap();
            let onetime_private_key = recover_onetime_private_key(
                &public_key,
                from_account_key.view_private_key(),
                &from_account_key.subaddress_spend_private(utxo.subaddress_index as u64),
            );

            let key_image = KeyImage::from(&onetime_private_key);
            log::debug!(
                self.logger,
                "Adding input: ring {:?}, utxo index {:?}, key image {:?}, pubkey {:?}",
                ring,
                real_key_index,
                key_image,
                public_key
            );

            self.transaction_builder.add_input(InputCredentials::new(
                ring,
                membership_proofs,
                real_key_index,
                onetime_private_key,
                *from_account_key.view_private_key(),
            )?);
        }

        // Add outputs to our destinations.
        let mut total_value = 0;
        let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
        let mut outlay_confirmation_numbers = Vec::default();
        let mut rng = rand::thread_rng();
        for (i, (recipient, out_value)) in self.outlays.iter().enumerate() {
            let target_acct_pubkey = Self::get_fog_pubkey_for_public_address(
                &recipient,
                &self.fog_pubkey_resolver,
                self.tombstone,
            )?;

            let (tx_out, confirmation_number) = self.transaction_builder.add_output(
                *out_value as u64,
                &recipient,
                target_acct_pubkey.as_ref(),
                &mut rng,
            )?;

            tx_out_to_outlay_index.insert(tx_out, i);
            outlay_confirmation_numbers.push(confirmation_number);

            total_value += *out_value;
        }

        // Figure out if we have change.
        let input_value = inputs_and_proofs
            .iter()
            .fold(0, |acc, (utxo, _proof)| acc + utxo.value);
        // FIXME: Can get subtraction overflow in mobilecoind::payments - needs fixing
        if (total_value + self.transaction_builder.fee) > input_value as u64 {
            return Err(WalletTransactionBuilderError::InsufficientFunds(
                format!("Cannot make change for value {:?}" input_value),
            ));
        }

        let change = input_value as u64 - total_value - self.transaction_builder.fee;

        // If we do, add an output for that as well.
        if change > 0 {
            let change_public_address =
                from_account_key.subaddress(account.change_subaddress_index as u64);
            let main_public_address =
                from_account_key.subaddress(account.main_subaddress_index as u64);

            // Note: The pubkey still needs to be for the main account
            // FIXME: Needs fixing in mobilecoind
            let target_acct_pubkey = Self::get_fog_pubkey_for_public_address(
                &main_public_address,
                &self.fog_pubkey_resolver,
                self.tombstone,
            )?;

            self.transaction_builder.add_output(
                change,
                &change_public_address,
                target_acct_pubkey.as_ref(),
                &mut rng,
            )?; // FIXME: map error to indicate error with change
        }

        // Set tombstone block.
        self.transaction_builder.set_tombstone_block(self.tombstone);

        // Build tx.
        let tx = self.transaction_builder.build(&mut rng)?;

        // Map each TxOut in the constructed transaction to its respective outlay.
        let outlay_index_to_tx_out_index: HashMap<usize, usize> =
            HashMap::from_iter(tx.prefix.outputs.iter().enumerate().filter_map(
                |(tx_out_index, tx_out)| {
                    if let Some(outlay_index) = tx_out_to_outlay_index.get(tx_out) {
                        Some((*outlay_index, tx_out_index))
                    } else {
                        None
                    }
                },
            ));

        // Sanity check: All of our outlays should have a unique index in the map.
        assert_eq!(outlay_index_to_tx_out_index.len(), self.outlays.len());
        let mut found_tx_out_indices: HashSet<&usize> = HashSet::default();
        for i in 0..self.outlays.len() {
            let tx_out_index = outlay_index_to_tx_out_index
                .get(&i)
                .expect("index not in map");
            if !found_tx_out_indices.insert(tx_out_index) {
                panic!("duplicate index {} found in map", tx_out_index);
            }
        }

        // Make the UnspentTxOut for each Txo
        // FIXME: I would prefer to provide just the txo_id_hex per txout, but this at last
        //        preserves some interoperability between mobilecoind and wallet-service.
        //        However, this is pretty clunky and I would rather not expose a storage
        //        type from mobilecoind just to get around having to write a bunch of tedious
        //        json conversions.
        // Return the TxProposal
        let selected_utxos = inputs_and_proofs
            .iter()
            .map(|(utxo, _membership_proof)| {
                let decoded_tx_out = mc_util_serial::decode(&utxo.txo).unwrap();
                let decoded_key_image =
                    mc_util_serial::decode(&utxo.key_image.clone().unwrap()).unwrap();
                UnspentTxOut {
                    tx_out: decoded_tx_out,
                    subaddress_index: utxo.subaddress_index as u64,
                    key_image: decoded_key_image,
                    value: utxo.value as u64,
                    attempted_spend_height: 0, // NOTE: these are null because not tracked here
                    attempted_spend_tombstone: 0,
                }
            })
            .collect();

        Ok(TxProposal {
            utxos: selected_utxos,
            outlays: self
                .outlays
                .iter()
                .map(|(recipient, value)| Outlay {
                    receiver: recipient.clone(),
                    value: *value,
                })
                .collect::<Vec<Outlay>>(),
            tx,
            outlay_index_to_tx_out_index,
            outlay_confirmation_numbers,
        })
    }

    /// Get rings.
    fn get_rings(
        &self,
        num_rings: usize,
        excluded_tx_out_indices: &[u64],
    ) -> Result<Vec<Vec<(TxOut, TxOutMembershipProof)>>, WalletTransactionBuilderError> {
        let num_requested = RING_SIZE * num_rings;
        let num_txos = self.ledger_db.num_txos()?;

        // Check that the ledger contains enough tx outs.
        if excluded_tx_out_indices.len() as u64 > num_txos {
            return Err(WalletTransactionBuilderError::InvalidArgument(
                "excluded_tx_out_indices exceeds amount of tx outs in ledger".to_string(),
            ));
        }

        if num_requested > (num_txos as usize - excluded_tx_out_indices.len()) {
            return Err(WalletTransactionBuilderError::InsufficientTxOuts);
        }

        // Randomly sample `num_requested` TxOuts, without replacement and convert into a Vec<u64>
        let mut rng = rand::thread_rng();
        let mut sampled_indices: HashSet<u64> = HashSet::default();
        while sampled_indices.len() < num_requested {
            let index = rng.gen_range(0, num_txos);
            if excluded_tx_out_indices.contains(&index) {
                continue;
            }
            sampled_indices.insert(index);
        }
        let sampled_indices_vec: Vec<u64> = sampled_indices.into_iter().collect();

        // Get proofs for all of those indexes.
        let proofs = self
            .ledger_db
            .get_tx_out_proof_of_memberships(&sampled_indices_vec)?;

        // Create an iterator that returns (index, proof) elements.
        let mut indexes_and_proofs_iterator =
            sampled_indices_vec.into_iter().zip(proofs.into_iter());

        // Convert that into a Vec<Vec<TxOut, TxOutMembershipProof>>
        let mut rings_with_proofs = Vec::new();

        for _ in 0..num_rings {
            let mut ring = Vec::new();
            for _ in 0..RING_SIZE {
                let (index, proof) = indexes_and_proofs_iterator.next().unwrap();
                let tx_out = self.ledger_db.get_tx_out_by_index(index)?;

                ring.push((tx_out, proof));
            }
            rings_with_proofs.push(ring);
        }

        Ok(rings_with_proofs)
    }

    fn get_fog_pubkey_for_public_address(
        address: &PublicAddress,
        fog_pubkey_resolver: &Option<Arc<FPR>>,
        tombstone_block: BlockIndex,
    ) -> Result<Option<RistrettoPublic>, WalletTransactionBuilderError> {
        if address.fog_report_url().is_none() {
            return Ok(None);
        }

        match fog_pubkey_resolver.as_ref() {
            None => Err(WalletTransactionBuilderError::FogError(format!(
                "{} uses fog but mobilecoind was started without fog support",
                address,
            ))),
            Some(resolver) => resolver
                .get_fog_pubkey(address)
                .map_err(|err| {
                    WalletTransactionBuilderError::FogError(format!(
                        "Failed getting fog public key for{}: {}",
                        address, err
                    ))
                })
                .and_then(|result| {
                    if tombstone_block > result.pubkey_expiry {
                        return Err(WalletTransactionBuilderError::FogError(format!("{} fog public key expiry block ({}) is lower than the provided tombstone block ({})", address, result.pubkey_expiry, tombstone_block)));
                    }
                    Ok(Some(result.pubkey))
                }),
            }
    }
}
