// Copyright (c) 2020-2021 MobileCoin Inc.

//! A builder for transactions from the wallet. Note that we have a
//! TransactionBuilder in the MobileCoin transaction crate, but that is a lower
//! level of building, once you have already obtained all of the materials that
//! go into a transaction.
//!
//! This module, on the other hand, builds a transaction within the context of
//! the wallet.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, Txo},
        txo::{TxoID, TxoModel},
        Conn,
    },
    error::WalletTransactionBuilderError,
    fog_resolver::{FullServiceFogResolver, FullServiceFullyValidatedFogPubkey},
    service::models::tx_proposal::{InputTxo, OutputTxo, TxProposal},
    unsigned_tx::UnsignedTx,
    util::b58::b58_encode_public_address,
};
use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::{
    logger::{log, Logger},
    HashMap, HashSet,
};
use mc_crypto_keys::RistrettoPublic;
use mc_crypto_ring_signature_signer::NoKeysRingSigner;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_core::{
    constants::RING_SIZE,
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{TxIn, TxOut, TxOutMembershipProof},
    Amount, BlockVersion, Token, TokenId,
};
use mc_transaction_std::{
    InputCredentials, RTHMemoBuilder, ReservedSubaddresses, SenderMemoCredential,
    TransactionBuilder,
};
use mc_util_uri::FogUri;

use rand::Rng;
use std::{collections::BTreeMap, convert::TryFrom, str::FromStr, sync::Arc};

/// Default number of blocks used for calculating transaction tombstone block
/// number.
// TODO support for making this configurable
pub const DEFAULT_NEW_TX_BLOCK_ATTEMPTS: u64 = 10;

/// A builder of transactions constructed from this wallet.
pub struct WalletTransactionBuilder<FPR: FogPubkeyResolver + 'static> {
    /// Account ID (hex-encoded) from which to construct a transaction.
    account_id_hex: String,

    /// The ledger DB.
    ledger_db: LedgerDB,

    /// Optional inputs specified to use to construct the transaction.
    inputs: Vec<Txo>,

    /// Vector of (PublicAddress, Amounts) for the recipients of this
    /// transaction.
    outlays: Vec<(PublicAddress, u64, TokenId)>,

    /// The block after which this transaction is invalid.
    tombstone: u64,

    /// The fee for the transaction.
    fee: Option<(u64, TokenId)>,

    /// The block version for the transaction
    block_version: Option<BlockVersion>,

    /// Fog resolver maker, used when constructing outputs to fog recipients.
    /// This is abstracted because in tests, we don't want to form grpc
    /// connections to fog.
    fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync>,

    /// Logger.
    logger: Logger,
}

impl<FPR: FogPubkeyResolver + 'static> WalletTransactionBuilder<FPR> {
    pub fn new(
        account_id_hex: String,
        ledger_db: LedgerDB,
        fog_resolver_factory: Arc<dyn Fn(&[FogUri]) -> Result<FPR, String> + Send + Sync + 'static>,
        logger: Logger,
    ) -> Self {
        WalletTransactionBuilder {
            account_id_hex,
            ledger_db,
            inputs: vec![],
            outlays: vec![],
            tombstone: 0,
            fee: None,
            block_version: None,
            fog_resolver_factory,
            logger,
        }
    }

    /// Sets inputs to the txos associated with the given txo_ids. Only unspent
    /// txos are included.
    pub fn set_txos(
        &mut self,
        conn: &Conn,
        input_txo_ids: &[String],
    ) -> Result<(), WalletTransactionBuilderError> {
        let txos = Txo::select_by_id(input_txo_ids, conn)?;

        let unspent: Vec<Txo> = txos
            .iter()
            .filter(|txo| txo.spent_block_index == None)
            .cloned()
            .collect();

        if unspent.iter().map(|t| t.value as u128).sum::<u128>() > u64::MAX as u128 {
            return Err(WalletTransactionBuilderError::OutboundValueTooLarge);
        }

        self.inputs = unspent;

        Ok(())
    }

    /// Selects Txos from the account.
    pub fn select_txos(
        &mut self,
        conn: &Conn,
        max_spendable_value: Option<u64>,
    ) -> Result<(), WalletTransactionBuilderError> {
        let mut outlay_value_sum_map: BTreeMap<TokenId, u128> =
            self.outlays
                .iter()
                .fold(BTreeMap::new(), |mut acc, (_, value, token_id)| {
                    acc.entry(*token_id)
                        .and_modify(|v| *v += *value as u128)
                        .or_insert(*value as u128);
                    acc
                });

        let (fee, token_id) = self.fee.unwrap_or((Mob::MINIMUM_FEE, Mob::ID));
        outlay_value_sum_map
            .entry(token_id)
            .and_modify(|v| *v += fee as u128)
            .or_insert(fee as u128);

        for (token_id, target_value) in outlay_value_sum_map {
            if target_value > u64::MAX as u128 {
                return Err(WalletTransactionBuilderError::OutboundValueTooLarge);
            }

            self.inputs = Txo::select_spendable_txos_for_value(
                &self.account_id_hex,
                target_value as u64,
                max_spendable_value,
                *token_id,
                conn,
            )?;
        }

        Ok(())
    }

    pub fn add_recipient(
        &mut self,
        recipient: PublicAddress,
        value: u64,
        token_id: TokenId,
    ) -> Result<(), WalletTransactionBuilderError> {
        // Verify that the maximum output value of this transaction remains under
        // u64::MAX for the given Token Id
        let cur_sum = self
            .outlays
            .iter()
            .filter_map(|(_r, v, t)| {
                if *t == token_id {
                    Some(*v as u128)
                } else {
                    None
                }
            })
            .sum::<u128>();
        if cur_sum > u64::MAX as u128 {
            return Err(WalletTransactionBuilderError::OutboundValueTooLarge);
        }
        self.outlays.push((recipient, value, token_id));
        Ok(())
    }

    pub fn set_fee(
        &mut self,
        fee: u64,
        token_id: TokenId,
    ) -> Result<(), WalletTransactionBuilderError> {
        if fee < 1 {
            return Err(WalletTransactionBuilderError::InsufficientFee(
                "1".to_string(),
            ));
        }
        self.fee = Some((fee, token_id));
        Ok(())
    }

    pub fn set_block_version(&mut self, block_version: BlockVersion) {
        self.block_version = Some(block_version);
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

    pub fn get_fs_fog_resolver(
        &self,
        conn: &Conn,
    ) -> Result<FullServiceFogResolver, WalletTransactionBuilderError> {
        let account = Account::get(&AccountID(self.account_id_hex.clone()), conn)?;
        let change_subaddress = account.change_subaddress(conn)?;
        let change_public_address = change_subaddress.public_address()?;

        let fog_resolver = {
            let fog_uris = core::slice::from_ref(&change_public_address)
                .iter()
                .chain(self.outlays.iter().map(|(receiver, _, _)| receiver))
                .filter_map(|x| extract_fog_uri(x).transpose())
                .collect::<Result<Vec<_>, _>>()?;
            (self.fog_resolver_factory)(&fog_uris)
                .map_err(WalletTransactionBuilderError::FogPubkeyResolver)?
        };

        let mut fully_validated_fog_pubkeys: HashMap<String, FullServiceFullyValidatedFogPubkey> =
            HashMap::default();

        for (public_address, _, _) in self.outlays.iter() {
            let fog_pubkey = match fog_resolver.get_fog_pubkey(public_address) {
                Ok(fog_pubkey) => Some(fog_pubkey),
                Err(_) => None,
            };

            if let Some(fog_pubkey) = fog_pubkey {
                let fs_fog_pubkey = FullServiceFullyValidatedFogPubkey::from(fog_pubkey);
                let b58_public_address = b58_encode_public_address(public_address)?;
                fully_validated_fog_pubkeys.insert(b58_public_address, fs_fog_pubkey);
            }
        }

        Ok(FullServiceFogResolver(fully_validated_fog_pubkeys))
    }

    pub fn build_unsigned(&self) -> Result<UnsignedTx, WalletTransactionBuilderError> {
        if self.tombstone == 0 {
            return Err(WalletTransactionBuilderError::TombstoneNotSet);
        }

        if self.inputs.is_empty() {
            return Err(WalletTransactionBuilderError::NoInputs);
        }

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

        let mut inputs_and_real_indices_and_subaddress_indices: Vec<(TxIn, u64, u64)> = Vec::new();

        for (utxo, proof) in inputs_and_proofs.iter() {
            let db_tx_out: TxOut = mc_util_serial::decode(&utxo.txo)?;
            let (mut ring, mut membership_proofs) = rings_and_proofs
                .pop()
                .ok_or(WalletTransactionBuilderError::RingsAndProofsEmpty)?;
            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            // Add the input to the ring.
            let position_opt = ring.iter().position(|txo| *txo == db_tx_out);
            let real_index = match position_opt {
                Some(position) => {
                    // The input is already present in the ring.
                    // This could happen if ring elements are sampled
                    // randomly from the             // ledger.
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
                    // The real input is always the first element. This is
                    // safe because TransactionBuilder sorts each ring.
                    0
                }
            };

            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            let tx_in = TxIn {
                ring,
                proofs: membership_proofs,
                input_rules: None,
            };

            inputs_and_real_indices_and_subaddress_indices.push((
                tx_in,
                real_index as u64,
                utxo.subaddress_index.unwrap() as u64,
            ));
        }

        let mut outlays_string = Vec::new();
        for (receiver, amount, token_id) in self.outlays.clone().into_iter() {
            let b58_address = b58_encode_public_address(&receiver)?;
            outlays_string.push((b58_address, amount, *token_id));
        }

        let (fee, fee_token_id) = self.fee.unwrap_or((Mob::MINIMUM_FEE, Mob::ID));

        Ok(UnsignedTx {
            inputs_and_real_indices_and_subaddress_indices,
            outlays: outlays_string,
            fee,
            fee_token_id: *fee_token_id,
            tombstone_block_index: self.tombstone,
            block_version: self.block_version.unwrap_or(BlockVersion::MAX),
        })
    }

    /// Consumes self
    pub fn build(&self, conn: &Conn) -> Result<TxProposal, WalletTransactionBuilderError> {
        if self.inputs.is_empty() {
            return Err(WalletTransactionBuilderError::NoInputs);
        }

        if self.tombstone == 0 {
            return Err(WalletTransactionBuilderError::TombstoneNotSet);
        }

        let account: Account = Account::get(&AccountID(self.account_id_hex.to_string()), conn)?;
        let from_account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;

        // Collect all required FogUris from public addresses, then pass to resolver
        // factory
        let fog_resolver = {
            let change_address =
                from_account_key.subaddress(account.change_subaddress_index as u64);
            let fog_uris = core::slice::from_ref(&change_address)
                .iter()
                .chain(
                    self.outlays
                        .iter()
                        .map(|(receiver, _amount, _token_id)| receiver),
                )
                .filter_map(|x| extract_fog_uri(x).transpose())
                .collect::<Result<Vec<_>, _>>()?;
            (self.fog_resolver_factory)(&fog_uris)
                .map_err(WalletTransactionBuilderError::FogPubkeyResolver)?
        };

        // Create transaction builder.
        let mut memo_builder = RTHMemoBuilder::default();
        memo_builder.set_sender_credential(SenderMemoCredential::from(&from_account_key));
        memo_builder.enable_destination_memo();
        let block_version = self.block_version.unwrap_or(BlockVersion::MAX);
        let (fee, token_id) = self.fee.unwrap_or((Mob::MINIMUM_FEE, Mob::ID));
        let fee = Amount::new(fee, token_id);
        let mut transaction_builder =
            TransactionBuilder::new(block_version, fee, fog_resolver, memo_builder)?;

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
            let db_tx_out: TxOut = mc_util_serial::decode(&utxo.txo)?;
            let (mut ring, mut membership_proofs) = rings_and_proofs
                .pop()
                .ok_or(WalletTransactionBuilderError::RingsAndProofsEmpty)?;
            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            // Add the input to the ring.
            let position_opt = ring.iter().position(|txo| *txo == db_tx_out);
            let real_key_index = match position_opt {
                Some(position) => {
                    // The input is already present in the ring.
                    // This could happen if ring elements are sampled randomly from the
                    // ledger.
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
                    // The real input is always the first element. This is safe because
                    // TransactionBuilder sorts each ring.
                    0
                }
            };

            if ring.len() != membership_proofs.len() {
                return Err(WalletTransactionBuilderError::RingSizeMismatch);
            }

            let public_key = RistrettoPublic::try_from(&db_tx_out.public_key).unwrap();

            let subaddress_index = if let Some(s) = utxo.subaddress_index {
                s
            } else {
                return Err(WalletTransactionBuilderError::NullSubaddress(
                    utxo.id.to_string(),
                ));
            };

            let onetime_private_key = recover_onetime_private_key(
                &public_key,
                from_account_key.view_private_key(),
                &from_account_key.subaddress_spend_private(subaddress_index as u64),
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

            transaction_builder.add_input(InputCredentials::new(
                ring,
                membership_proofs,
                real_key_index,
                onetime_private_key,
                *from_account_key.view_private_key(),
            )?);
        }

        // Add outputs to our destinations.
        // Note that we make an assumption currently when logging submitted Txos that
        // they were built  with only one recip ient, and one change txo.
        let mut total_value_per_token: BTreeMap<TokenId, u64> = BTreeMap::new();
        total_value_per_token.insert(
            transaction_builder.get_fee_token_id(),
            transaction_builder.get_fee(),
        );
        let mut payload_txos: Vec<OutputTxo> = Vec::new();
        let mut change_txos: Vec<OutputTxo> = Vec::new();
        let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
        let mut outlay_confirmation_numbers = Vec::default();
        let mut rng = rand::thread_rng();
        for (i, (recipient, out_value, token_id)) in self.outlays.iter().enumerate() {
            let tx_out_context = transaction_builder.add_output(
                Amount::new(*out_value, *token_id),
                recipient,
                &mut rng,
            )?;

            payload_txos.push(OutputTxo {
                tx_out: tx_out_context.tx_out.clone(),
                recipient_public_address: recipient.clone(),
                confirmation_number: tx_out_context.confirmation.clone(),
                value: *out_value,
                token_id: *token_id,
            });

            tx_out_to_outlay_index.insert(tx_out_context.tx_out, i);
            outlay_confirmation_numbers.push(tx_out_context.confirmation);

            total_value_per_token
                .entry(*token_id)
                .and_modify(|v| *v += *out_value)
                .or_insert(*out_value);
        }

        // Figure out if we have change.
        let input_value_per_token =
            inputs_and_proofs
                .iter()
                .fold(BTreeMap::new(), |mut acc, (utxo, _proof)| {
                    acc.entry(TokenId::from(utxo.token_id as u64))
                        .and_modify(|v| *v += utxo.value as u64)
                        .or_insert(utxo.value as u64);
                    acc
                });

        for (token_id, total_value) in total_value_per_token.iter() {
            let input_value = input_value_per_token.get(token_id).ok_or(
                WalletTransactionBuilderError::MissingInputsForTokenId(token_id.to_string()),
            )?;
            if total_value > input_value {
                return Err(WalletTransactionBuilderError::InsufficientInputFunds(
                    format!(
                        "Total value required to send transaction {:?}, but only {:?} in inputs for token_id {:?}",
                        total_value,
                        input_value,
                        token_id.to_string()
                    ),
                ));
            }

            let change = input_value - total_value;
            let reserved_subaddresses = ReservedSubaddresses::from(&from_account_key);
            let tx_out_context = transaction_builder.add_change_output(
                Amount::new(change, *token_id),
                &reserved_subaddresses,
                &mut rng,
            )?;

            change_txos.push(OutputTxo {
                tx_out: tx_out_context.tx_out,
                recipient_public_address: reserved_subaddresses.change_subaddress,
                confirmation_number: tx_out_context.confirmation,
                value: change,
                token_id: *token_id,
            });
        }

        // Set tombstone block.
        transaction_builder.set_tombstone_block(self.tombstone);

        // Build tx.
        let tx = transaction_builder.build(&NoKeysRingSigner {}, &mut rng)?;

        // Map each TxOut in the constructed transaction to its respective outlay.
        let outlay_index_to_tx_out_index: HashMap<usize, usize> = tx
            .prefix
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(tx_out_index, tx_out)| {
                tx_out_to_outlay_index
                    .get(tx_out)
                    .map(|outlay_index| (*outlay_index, tx_out_index))
            })
            .collect();

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
        // FIXME: WS-27 - I would prefer to provide just the txo_id_hex per txout, but
        // this at least preserves some interoperability between
        // mobilecoind and wallet-service. However, this is
        // pretty clunky and I would rather not expose a storage
        // type from mobilecoind just to get around having to write a bunch of
        // tedious json conversions.
        // Return the TxProposal
        let input_txos = inputs_and_proofs
            .iter()
            .map(|(utxo, _membership_proof)| {
                let decoded_tx_out = mc_util_serial::decode(&utxo.txo).unwrap();
                let decoded_key_image =
                    mc_util_serial::decode(&utxo.key_image.clone().unwrap()).unwrap();

                InputTxo {
                    tx_out: decoded_tx_out,
                    key_image: decoded_key_image,
                    value: utxo.value as u64,
                    token_id: TokenId::from(utxo.token_id as u64),
                }
            })
            .collect();

        Ok(TxProposal {
            tx,
            input_txos,
            payload_txos,
            change_txos,
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

        // Randomly sample `num_requested` TxOuts, without replacement and convert into
        // a Vec<u64>
        let mut rng = rand::thread_rng();
        let mut sampled_indices: HashSet<u64> = HashSet::default();
        while sampled_indices.len() < num_requested {
            let index = rng.gen_range(0..num_txos);
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
}

// Helper which extracts FogUri from PublicAddress or returns None, or returns
// an error
fn extract_fog_uri(addr: &PublicAddress) -> Result<Option<FogUri>, WalletTransactionBuilderError> {
    if let Some(string) = addr.fog_report_url() {
        Ok(Some(FogUri::from_str(string)?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::WalletDbError,
        service::sync::SyncThread,
        test_utils::{
            builder_for_random_recipient, get_test_ledger, random_account_with_seed_values,
            WalletDbTestContext, MOB,
        },
    };
    use mc_common::logger::{test_with_logger, Logger};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_build_with_utxos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![11 * MOB, 11 * MOB, 11 * MOB, 111111 * MOB],
            &mut rng,
            &logger,
        );

        // Construct a transaction
        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        // Send value specifically for your smallest Txo size. Should take 2 inputs
        // and also make change.
        let value = 11 * MOB;
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();

        // Select the txos for the recipient
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].value, value);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2);
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2);
    }

    // Test that large values are handled correctly.
    #[test_with_logger]
    fn test_big_values(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        // Give ourselves enough MOB that we have more than u64::MAX, 18_446_745 MOB
        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![7_000_000 * MOB, 7_000_000 * MOB, 7_000_000 * MOB],
            &mut rng,
            &logger,
        );

        // Check balance
        let unspent = Txo::list_unspent(
            Some(&AccountID::from(&account_key).to_string()),
            None,
            Some(0),
            None,
            None,
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();
        let balance: u128 = unspent.iter().map(|t| t.value as u128).sum::<u128>();
        assert_eq!(balance, 21_000_000 * MOB as u128);

        // Now try to send a transaction with a value > u64::MAX
        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        let value = u64::MAX;
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();

        // Select the txos for the recipient - should error because > u64::MAX
        match builder.select_txos(&conn, None) {
            Ok(_) => panic!("Should not be allowed to construct outbound values > u64::MAX"),
            Err(WalletTransactionBuilderError::OutboundValueTooLarge) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    // Users should be able to set the txos specifically that they want to send
    #[test_with_logger]
    fn test_setting_txos(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        // Get our TXO list
        let txos: Vec<Txo> = Txo::list_for_account(
            &AccountID::from(&account_key).to_string(),
            None,
            None,
            None,
            Some(0),
            &wallet_db.get_conn().unwrap(),
        )
        .unwrap();

        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        // Setting value to exactly the input will fail because you need funds for fee
        builder
            .add_recipient(recipient.clone(), txos[0].value as u64, Mob::ID)
            .unwrap();

        builder.set_txos(&conn, &vec![txos[0].id.clone()]).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.build(&conn) {
            Ok(_) => {
                panic!("Should not be able to construct Tx with > inputs value as output value")
            }
            Err(WalletTransactionBuilderError::InsufficientInputFunds(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Now build, setting to multiple TXOs
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        // Set value to just slightly more than what fits in the one TXO
        builder
            .add_recipient(recipient.clone(), txos[0].value as u64 + 10, Mob::ID)
            .unwrap();

        builder
            .set_txos(&conn, &vec![txos[0].id.clone(), txos[1].id.clone()])
            .unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].value, txos[0].value as u64 + 10);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2); // need one more for fee
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // self and change
    }

    // Test max_spendable correctly filters out txos above max_spendable
    #[test_with_logger]
    fn test_max_spendable(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        // Setting value to exactly the input will fail because you need funds for fee
        builder
            .add_recipient(recipient.clone(), 80 * MOB, Mob::ID)
            .unwrap();

        // Test that selecting Txos with max_spendable < all our txo values fails
        match builder.select_txos(&conn, Some(10)) {
            Ok(_) => panic!("Should not be able to construct tx when max_spendable < all txos"),
            Err(WalletTransactionBuilderError::WalletDb(WalletDbError::NoSpendableTxos)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // We should be able to try again, with max_spendable at 70, but will not hit
        // our outlay target (80 * MOB)
        match builder.select_txos(&conn, Some(70 * MOB)) {
            Ok(_) => panic!("Should not be able to construct tx when max_spendable < all txos"),
            Err(WalletTransactionBuilderError::WalletDb(
                WalletDbError::InsufficientFundsUnderMaxSpendable(_),
            )) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Now, we should succeed if we set max_spendable = 80 * MOB, because we will
        // pick up both 70 and 80
        builder.select_txos(&conn, Some(80 * MOB)).unwrap();
        builder.set_tombstone(0).unwrap();
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].value, 80 * MOB);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2); // uses both 70 and 80
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // self and change
    }

    // Test setting and not setting tombstone block
    #[test_with_logger]
    fn test_tombstone(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);
        let conn = wallet_db.get_conn().unwrap();

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB],
            &mut rng,
            &logger,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();

        // Sanity check that our ledger is the height we think it is
        assert_eq!(ledger_db.num_blocks().unwrap(), 13);

        // We must set tombstone block before building
        match builder.build(&conn) {
            Ok(_) => panic!("Expected TombstoneNotSet error"),
            Err(WalletTransactionBuilderError::TombstoneNotSet) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();

        // Set to default
        builder.set_tombstone(0).unwrap();

        // Not setting the tombstone results in tombstone = 0. This is an acceptable
        // value,
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.tombstone_block, 23);

        // Build a transaction and explicitly set tombstone
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();

        // Set to default
        builder.set_tombstone(20).unwrap();

        // Not setting the tombstone results in tombstone = 0. This is an acceptable
        // value,
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.tombstone_block, 20);
    }

    // Test setting and not setting the fee
    #[test_with_logger]
    fn test_fee(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB],
            &mut rng,
            &logger,
        );

        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);

        // You cannot set fee to 0
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.set_fee(0, Mob::ID) {
            Ok(_) => panic!("Should not be able to set fee to 0"),
            Err(WalletTransactionBuilderError::InsufficientFee(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Verify that not setting fee results in default fee
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);

        // Setting fee less than minimum fee should fail
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        match builder.set_fee(0, Mob::ID) {
            Ok(_) => panic!("Should not be able to set fee to 0"),
            Err(WalletTransactionBuilderError::InsufficientFee(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }

        // Setting fee greater than MINIMUM_FEE works
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();
        builder.set_fee(Mob::MINIMUM_FEE * 10, Mob::ID).unwrap();
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE * 10);
    }

    // Even if change is zero, we should still have a change output
    #[test_with_logger]
    fn test_change_zero_mob(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB],
            &mut rng,
            &logger,
        );

        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        // Set value to consume the whole TXO and not produce change
        let value = 70 * MOB - Mob::MINIMUM_FEE;
        builder
            .add_recipient(recipient.clone(), value, Mob::ID)
            .unwrap();
        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.payload_txos.len(), 1);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].value, value);
        assert_eq!(proposal.tx.prefix.inputs.len(), 1); // uses just one input
        assert_eq!(proposal.tx.prefix.outputs.len(), 2); // two outputs to
                                                         // self
    }

    // We should be able to add multiple TxOuts to the same recipient, not to
    // multiple
    #[test_with_logger]
    fn test_add_multiple_outputs_to_same_recipient(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        let conn = wallet_db.get_conn().unwrap();
        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 20 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 30 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 40 * MOB, Mob::ID)
            .unwrap();

        builder.select_txos(&conn, None).unwrap();
        builder.set_tombstone(0).unwrap();

        // Verify that not setting fee results in default fee
        let proposal = builder.build(&conn).unwrap();
        assert_eq!(proposal.tx.prefix.fee, Mob::MINIMUM_FEE);
        assert_eq!(proposal.payload_txos.len(), 4);
        assert_eq!(proposal.payload_txos[0].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[0].value, 10 * MOB);
        assert_eq!(proposal.payload_txos[1].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[1].value, 20 * MOB);
        assert_eq!(proposal.payload_txos[2].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[2].value, 30 * MOB);
        assert_eq!(proposal.payload_txos[3].recipient_public_address, recipient);
        assert_eq!(proposal.payload_txos[3].value, 40 * MOB);
        assert_eq!(proposal.tx.prefix.inputs.len(), 2);
        assert_eq!(proposal.tx.prefix.outputs.len(), 5); // outlays + change
    }

    // Adding multiple values that exceed u64::MAX should fail
    #[test_with_logger]
    fn test_add_multiple_outputs_integer_overflow(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![
                7_000_000 * MOB,
                7_000_000 * MOB,
                7_000_000 * MOB,
                7_000_000 * MOB,
            ],
            &mut rng,
            &logger,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 7_000_000 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 7_000_000 * MOB, Mob::ID)
            .unwrap();
        builder
            .add_recipient(recipient.clone(), 7_000_000 * MOB, Mob::ID)
            .unwrap();

        match builder.select_txos(&wallet_db.get_conn().unwrap(), None) {
            Ok(_) => panic!("Should not be able to select txos with > u64::MAX output value"),
            Err(WalletTransactionBuilderError::OutboundValueTooLarge) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        }
    }

    // We should be able to add multiple TxOuts to multiple recipients.
    #[test_with_logger]
    fn test_add_multiple_recipients(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger.clone());
        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        // Start sync thread
        let _sync_thread = SyncThread::start(ledger_db.clone(), wallet_db.clone(), logger.clone());

        let account_key = random_account_with_seed_values(
            &wallet_db,
            &mut ledger_db,
            &vec![70 * MOB, 80 * MOB, 90 * MOB],
            &mut rng,
            &logger,
        );

        let (recipient, mut builder) =
            builder_for_random_recipient(&account_key, &ledger_db, &mut rng, &logger);

        builder
            .add_recipient(recipient.clone(), 10 * MOB, Mob::ID)
            .unwrap();

        // Create a new recipient
        let second_recipient = AccountKey::random(&mut rng).subaddress(0);
        builder
            .add_recipient(second_recipient.clone(), 40 * MOB, Mob::ID)
            .unwrap();
    }
}
