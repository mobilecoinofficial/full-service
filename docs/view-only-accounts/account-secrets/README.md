---
description: >-
  The secret keys for an account. The account secrets are returned separately
  from other account information, to enable more careful handling of
  cryptographically sensitive information.
---

# View Only Account Secrets

## Attributes

| Name | Type | Description |
| :--- | :--- | :--- |
| `object` | string, value is "view\_only\_account\_secrets" | String representing the object's type. Objects of the same type share the same value. |
| `view_private_key` | string | The private view key for with this account |

## Example

```text
{
  "object": "view_only_account_secrets",
  "account_id": "3407fbbc250799f5ce9089658380c5fe152403643a525f581f359917d8d59d52",
  "view_private_key": "0a207960bd832aae551ee03d6e5ab48baa229acd7ca4d2c6aaf7c8c4e77ac3e92307",
}
```

