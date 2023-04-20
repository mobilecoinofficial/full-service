---
description: Sync unverified txos
---

# Sync Txos

## Request

| Param           | Requirements                                              |
| --------------- | --------------------------------------------------------- |
| `mnemonic`      | Must be a valid 24 word mnemonic                          |
| `txos_unsynced` | list of unsynced txos from the VO account in full service |

## Response

{% tabs %}
{% tab title="Request Body" %}
```json
{
    "method": "sync_txos",
    "params": {
        "mnemonic": "divorce tortoise note draw forest strike replace cost also crowd front unusual demand south again rather pencil next remind future rally carry keen artefact",
        "txos_unsynced": [
                {
                    "subaddress": 0,
                    "tx_out_public_key": "eaf048498aa9ca4c47a94f6c677bee90c7398eae319cabc2e93f3de3f04b2979"
                }
            ]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}

{% tab title="Response" %}
```json
{
    "method": "sync_txos",
    "result": {
        "txos_synced": [
            {
                "tx_out_public_key": "eaf048498aa9ca4c47a94f6c677bee90c7398eae319cabc2e93f3de3f04b2979",
                "key_image": "46c125d70281d1d31b197080289529d74486a755bdae7499ffaaf9688892c75f"
            }
        ]
    },
    "jsonrpc": "2.0",
    "id": 1
}
```
{% endtab %}
{% endtabs %}
