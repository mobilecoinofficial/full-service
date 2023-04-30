---
description: >-
  The Block is an important primitive in the MobileCoin blockchain, and consists
  of TXOs and Key Images.
---

# Block

## Attributes

| Name             | Type                     | Description                                                                           |
|------------------|--------------------------|---------------------------------------------------------------------------------------|
| `object`         | string, value is "block" | String representing the object's type. Objects of the same type share the same value. |
| `block`          | JSON object              | Contains the block header information for the block                                   |
| `block_contents` | JSON object              | Contains the key_images and TXOs (outputs) for the block.                             |
