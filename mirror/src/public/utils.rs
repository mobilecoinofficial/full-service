// Copyright (c) 2018-2020 MobileCoin Inc.

//! Misc utility methods.

use mc_util_uri::ConnectionUri;
use x509_parser::{error::X509Error, parse_x509_der, pem::pem_to_der};

/// Checks if an optionally-provided TLS certificate is self-signed. Returns false if no TLS is
/// configured for the URI.
pub fn is_tls_self_signed(uri: &impl ConnectionUri) -> Result<bool, String> {
    // Short-circuit if no TLS is configured.
    if !uri.use_tls() {
        return Ok(false);
    }

    // Must have a TLS certificate, and must be able to read it.
    let cert_pem_bytes = uri.tls_chain()?;

    let (rem, pem) =
        pem_to_der(&cert_pem_bytes).map_err(|err| format!("pem_to_der failed: {}", err))?;
    if !rem.is_empty() || pem.label != "CERTIFICATE" {
        return Err(format!(
            "Failed parsing PEM: rem={:?}, pem.label={}",
            rem, pem.label
        ));
    }

    let (rem, cert) =
        parse_x509_der(&pem.contents).map_err(|err| format!("parse_x509_der failed: {}", err))?;
    if !rem.is_empty() {
        return Err(format!("Failed parsing DER: rem={:?}", rem));
    }

    // Check if we are self signed. If we are, the veritifcation would pass.
    // (Passing None defaults to cert.tbs_certificate.subject_pki)
    match cert.verify_signature(Some(&cert.tbs_certificate.subject_pki)) {
        Ok(()) => Ok(true),
        Err(X509Error::SignatureVerificationError) => Ok(false),
        Err(err) => Err(format!("Error verifying certificate: {:?}", err)),
    }
}
