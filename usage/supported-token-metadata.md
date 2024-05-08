# Supported Token Metadata

As tokens are approved by the foundation and integrated into the mobilecoin blockchain, there presents a need for clients to have an updated source of truth for that Token Metadata. As such, a json file and its ed25519 signature are hosted so that clients may retrieve and verify updated Token Metadata without needing to ship an update.

## Details:

The current list of approved tokens can be found at: [https://config.mobilecoin.foundation/token\_metadata.json](https://config.mobilecoin.foundation/token\_metadata.json)

An ed25519 signature over the JSON bytes can be found at:\
[https://config.mobilecoin.foundation/token\_metadata.sig](https://config.mobilecoin.foundation/token\_metadata.sig)

For a guide level explanation of the JSON schema and its usage, see:\
[https://github.com/mobilecoinfoundation/mcips/blob/main/text/0059-token-metadata-document.md](https://github.com/mobilecoinfoundation/mcips/blob/main/text/0059-token-metadata-document.md)

## Current Tokens

| Token ID | Name | Precision     | Approx Fee |
| -------- | ---- | ------------- | ---------- |
| 0        | MOB  | Pico (10^-12) | .0004      |
| 1        | eUSD | Micro (10^-6) | .00256     |
