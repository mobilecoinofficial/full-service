// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Block object.

use mc_mobilecoind_json::data_types::{JsonTxOut, JsonTxOutMembershipElement};
use serde::Serialize as SerdeSerialize;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct Block {
    pub id: String,
    pub version: String,
    pub parent_id: String,
    pub index: String,
    pub cumulative_txo_count: String,
    pub root_element: JsonTxOutMembershipElement,
    pub contents_hash: String,
}

impl Block {
    pub fn new(block: &mc_blockchain_types::Block) -> Self {
        let membership_element_proto =
            mc_api::external::TxOutMembershipElement::from(&block.root_element);
        Self {
            id: hex::encode(block.id.clone()),
            version: block.version.to_string(),
            parent_id: hex::encode(block.parent_id.clone()),
            index: block.index.to_string(),
            cumulative_txo_count: block.cumulative_txo_count.to_string(),
            root_element: JsonTxOutMembershipElement::from(&membership_element_proto),
            contents_hash: hex::encode(block.contents_hash.0),
        }
    }
}

impl From<&mc_blockchain_types::Block> for Block {
    fn from(src: &mc_blockchain_types::Block) -> Self {
        Self::new(src)
    }
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct WithId<T: SerdeSerialize> {
    pub id: String,
    #[serde(flatten)]
    pub inner: T,
}

impl<T: SerdeSerialize> WithId<T> {
    pub fn new(inner: T, id: String) -> Self {
        Self { id, inner }
    }
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct BlockContents {
    pub key_images: Vec<String>,
    pub outputs: Vec<WithId<JsonTxOut>>,
}

impl BlockContents {
    pub fn new(block_contents: &mc_blockchain_types::BlockContents) -> Self {
        Self {
            key_images: block_contents
                .key_images
                .iter()
                .map(|k| hex::encode(mc_util_serial::encode(k)))
                .collect::<Vec<String>>(),
            outputs: block_contents
                .outputs
                .iter()
                .map(|txo| {
                    let proto_txo = mc_api::external::TxOut::from(txo);
                    let json_txo = JsonTxOut::from(&proto_txo);
                    let id = crate::db::txo::TxoID::from(txo).to_string();
                    WithId::new(json_txo, id)
                })
                .collect::<Vec<WithId<JsonTxOut>>>(),
        }
    }
}
impl From<&mc_blockchain_types::BlockContents> for BlockContents {
    fn from(src: &mc_blockchain_types::BlockContents) -> Self {
        Self::new(src)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BlockSignature {
    pub signature: String,
    pub signer: String,
    pub signed_at: String,
}
impl From<&mc_blockchain_types::BlockSignature> for BlockSignature {
    fn from(src: &mc_blockchain_types::BlockSignature) -> Self {
        Self {
            signature: hex::encode(src.signature()),
            signer: hex::encode(src.signer()),
            signed_at: src.signed_at().to_string(),
        }
    }
}
