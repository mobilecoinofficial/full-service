use mc_account_keys::{AccountKey, PublicAddress};
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_crypto_ring_signature_signer::NoKeysRingSigner;

use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::{KeyImage, Scalar},
    tx::{TxIn, TxOut},
    Amount, BlockVersion, TokenId,
};
use mc_transaction_std::{
    BurnRedemptionMemo, BurnRedemptionMemoBuilder, InputCredentials, RTHMemoBuilder,
    ReservedSubaddresses, SenderMemoCredential, TransactionBuilder,
};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::TryFrom};

use crate::{
    error::WalletTransactionBuilderError,
    fog_resolver::FullServiceFogResolver,
    service::{
        models::tx_proposal::{InputTxo, OutputTxo, TxProposal},
        transaction::TransactionMemo,
    },
    util::b58::b58_decode_public_address,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// The fully constructed input rings
    pub inputs_and_real_indices_and_subaddress_indices: Vec<(TxIn, u64, u64)>,

    /// Vector of (PublicAddressB58, Amount, TokenId) for the recipients of this
    /// transaction.
    pub outlays: Vec<(String, u64, u64)>,

    /// The fee to be paid
    pub fee: u64,

    /// The fee's token id
    pub fee_token_id: u64,

    /// The tombstone block index
    pub tombstone_block_index: u64,

    /// The block version
    pub block_version: BlockVersion,

    /// Memo field that indicates what type of transaction this is.
    pub memo: TransactionMemo,
}

impl UnsignedTx {
    pub fn sign(
        self,
        account_key: &AccountKey,
        fog_resolver: FullServiceFogResolver,
    ) -> Result<TxProposal, WalletTransactionBuilderError> {
        let mut rng = rand::thread_rng();

        // Create transaction builder.
        let fee = Amount::new(self.fee, TokenId::from(self.fee_token_id));

        let mut transaction_builder = match self.memo {
            TransactionMemo::RTH => {
                let mut memo_builder = RTHMemoBuilder::default();
                memo_builder.set_sender_credential(SenderMemoCredential::from(account_key));
                memo_builder.enable_destination_memo();
                TransactionBuilder::new(self.block_version, fee, fog_resolver, memo_builder)?
            }
            TransactionMemo::BurnRedemption(redemption_memo_hex) => {
                let mut memo_data = [0; BurnRedemptionMemo::MEMO_DATA_LEN];

                if let Some(redemption_memo_hex) = redemption_memo_hex {
                    hex::decode_to_slice(&redemption_memo_hex, &mut memo_data)?;
                }

                let mut memo_builder = BurnRedemptionMemoBuilder::new(memo_data);
                memo_builder.enable_destination_memo();
                TransactionBuilder::new(self.block_version, fee, fog_resolver, memo_builder)?
            }
        };

        transaction_builder.set_tombstone_block(self.tombstone_block_index);

        let mut input_txos: Vec<InputTxo> = Vec::new();
        let mut input_total_per_token: BTreeMap<TokenId, u64> = BTreeMap::new();

        for (tx_in, real_index, subaddress_index) in
            self.inputs_and_real_indices_and_subaddress_indices
        {
            let tx_out = &tx_in.ring[real_index as usize];
            let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;

            let onetime_private_key = recover_onetime_private_key(
                &tx_public_key,
                account_key.view_private_key(),
                &account_key.subaddress_spend_private(subaddress_index),
            );

            let key_image = KeyImage::from(&onetime_private_key);

            let input_credentials = InputCredentials::new(
                tx_in.ring.clone(),
                tx_in.proofs.clone(),
                real_index as usize,
                onetime_private_key,
                *account_key.view_private_key(),
            )?;

            transaction_builder.add_input(input_credentials);

            let tx_out = &tx_in.ring[real_index as usize];
            let (amount, _) = decode_amount(tx_out, account_key.view_private_key())?;

            let input = InputTxo {
                tx_out: tx_out.clone(),
                key_image,
                amount,
                subaddress_index,
            };

            input_txos.push(input);
            input_total_per_token
                .entry(amount.token_id)
                .and_modify(|x| *x += amount.value)
                .or_insert(amount.value);
        }

        let mut outlays_decoded: Vec<(PublicAddress, Amount)> = Vec::new();

        for (public_address_b58, amount, token_id) in &self.outlays {
            let public_address = b58_decode_public_address(public_address_b58)?;
            let amount = Amount::new(*amount, TokenId::from(*token_id));
            outlays_decoded.push((public_address, amount));
        }

        let payload_txos =
            add_payload_outputs(&outlays_decoded, &mut transaction_builder, &mut rng)?;

        let mut output_total_per_token: BTreeMap<TokenId, u64> = BTreeMap::new();
        output_total_per_token.insert(TokenId::from(self.fee_token_id), self.fee);

        for (_, amount, token_id) in self.outlays.into_iter() {
            output_total_per_token
                .entry(TokenId::from(token_id))
                .and_modify(|x| *x += amount)
                .or_insert(amount);
        }

        let change_txos = input_total_per_token
            .into_iter()
            .map(|(token_id, input_total)| {
                let output_total = output_total_per_token.get(&token_id).unwrap_or(&0);
                add_change_output(
                    account_key,
                    input_total,
                    *output_total,
                    token_id,
                    &mut transaction_builder,
                    &mut rng,
                )
            })
            .collect::<Result<Vec<_>, WalletTransactionBuilderError>>()?;

        let tx = transaction_builder.build(&NoKeysRingSigner {}, &mut rng)?;

        Ok(TxProposal {
            tx,
            input_txos,
            payload_txos,
            change_txos,
        })
    }
}

pub fn decode_amount(
    tx_out: &TxOut,
    view_private_key: &RistrettoPrivate,
) -> Result<(Amount, Scalar), WalletTransactionBuilderError> {
    let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;
    let shared_secret = get_tx_out_shared_secret(view_private_key, &tx_public_key);
    Ok(tx_out.masked_amount.get_value(&shared_secret)?)
}

#[allow(clippy::type_complexity)]
fn add_payload_outputs<RNG: CryptoRng + RngCore>(
    outlays: &[(PublicAddress, Amount)],
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> Result<Vec<OutputTxo>, WalletTransactionBuilderError> {
    // Add outputs to our destinations.
    let mut outputs = Vec::new();
    for (recipient, amount) in outlays.iter() {
        let tx_out_context = transaction_builder.add_output(*amount, recipient, rng)?;

        outputs.push(OutputTxo {
            tx_out: tx_out_context.tx_out,
            recipient_public_address: recipient.clone(),
            amount: *amount,
            confirmation_number: tx_out_context.confirmation,
        });
    }
    Ok(outputs)
}

fn add_change_output<RNG: CryptoRng + RngCore>(
    account_key: &AccountKey,
    total_input_value: u64,
    total_output_value: u64,
    token_id: TokenId,
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> Result<OutputTxo, WalletTransactionBuilderError> {
    let change_value = total_input_value - total_output_value;
    let change_amount = Amount::new(change_value, token_id);

    let reserved_subaddresses = ReservedSubaddresses::from(account_key);
    let tx_out_context =
        transaction_builder.add_change_output(change_amount, &reserved_subaddresses, rng)?;

    Ok(OutputTxo {
        tx_out: tx_out_context.tx_out,
        recipient_public_address: reserved_subaddresses.change_subaddress,
        amount: change_amount,
        confirmation_number: tx_out_context.confirmation,
    })
}
