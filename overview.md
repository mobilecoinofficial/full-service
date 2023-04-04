# Overview

MobileCoin aspires to be the most trusted payment system in the world. We designed and built our network with the most secure open source protocols available. Our encrypted payments are created using secret keys that never leave the user’s device.

The MobileCoin Network is an open-source software ecosystem that introduces several innovations to the cryptocurrency community, including:

* ****[**MobileCoin Ledger**](glossary/ledger.md)**,** a new privacy-preserving blockchain built on a technology foundation that includes CryptoNote and Ring Confidential Transactions (RingCT).
* ****[**MobileCoin Consensus Protocol**](glossary/consensus-protocol.md), a high-performance solution to the Byzantine Agreement Problem that allows new payments to be rapidly confirmed.
* ****[**Secure Enclaves**](glossary/secure-enclave.md), trusted execution environments using Intel’s Software Guard eXtensions (SGX) to provide defense-in-depth improvements to privacy and trust.&#x20;
* ****[**MobileCoin Fog**](glossary/fog.md), a scalable service infrastructure that enables a smartphone to manage a privacy-preserving cryptocurrency with locally-stored cryptographic keys.
* ****[**Full Service**](glossary/full-service.md), a ledger sync and account management tool that provides payment and transaction services, similar to a payment gateway or payment management system.
* ****[**Validator Service**](usage/validator-service/), a ledger sync tool that provides a middleware for Full Service to proxy its calls to consensus and download the ledger.
* ****[**MobileCoin Mirror Wallet**](usage/mirror/), a pair of services that push data from Full Service to a mirror to expose a read-only API that does not require private keys.
* **Python Integration Software**, code written by MobileCoin in Python to enable fast and efficient integration between a MobileCoin Ledger on the MobileCoin Network and the Exchange.

MobileCoin Mirror Wallet, a pair of services that push data from Full Service to a mirror to expose a read-only API that does not require private keys.

<figure><img src=".gitbook/assets/mobilecoin ecosytem.jpg" alt=""><figcaption><p>The MobileCoin Network at a glance</p></figcaption></figure>
