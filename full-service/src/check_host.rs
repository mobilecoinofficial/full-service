// Copyright (c) 2018-2023 MobileCoin, Inc.

//! Utility for IP check related logic.

use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
};

use displaydoc::Display;

/// The Errors that may occur when checking if host is allowed
#[derive(Display, Debug)]
pub enum CheckHostError {
    /// Error parsing json {0}
    Json(serde_json::Error),

    /// Error handling reqwest {0}
    Reqwest(reqwest::Error),

    /// Invalid country
    InvalidCountry,

    /// Data missing in the response {0}
    DataMissing(String),
}

impl From<serde_json::Error> for CheckHostError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<reqwest::Error> for CheckHostError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

/// Ensure local IP address is valid.
///
/// Uses ipinfo.io for getting details about IP address.
///
/// Note, both of these services are free tier and rate-limited.
pub fn check_host_is_allowed_country_and_region() -> Result<(), CheckHostError> {
    let client = Client::builder().gzip(true).use_rustls_tls().build()?;
    let mut json_headers = HeaderMap::new();
    json_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let response = client
        .get("https://ipinfo.io/json/")
        .headers(json_headers)
        .send()?
        .error_for_status()?;
    let data = response.text()?;
    let data_json: serde_json::Value = serde_json::from_str(&data)?;

    let data_missing_err = Err(CheckHostError::DataMissing(data_json.to_string()));
    let country: &str = match data_json["country"].as_str() {
        Some(c) => c,
        None => return data_missing_err,
    };
    let region: &str = match data_json["region"].as_str() {
        Some(r) => r,
        None => return data_missing_err,
    };

    let err = Err(CheckHostError::InvalidCountry);
    match country {
        "IR" | "SY" | "CU" | "KP" | "RU" => err,
        "UA" => match region {
            "Crimea" => err,
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}
