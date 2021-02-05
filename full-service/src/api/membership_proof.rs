// Copyright (c) 2020-2021 MobileCoin Inc.

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct JsonProof {
    pub object: String,
    pub txo_id: String,
    pub proof: String,
}
