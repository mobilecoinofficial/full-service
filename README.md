# wallet-service
A MobileCoin service for wallet implementations.

## Build and Run

1. Get the appropriate published enclave measurement, and save to `$(pwd)/consensus-enclave.css`

    ```sh
    NAMESPACE=test
    SIGNED_ENCLAVE_URI=$(curl -s https://enclave-distribution.${NAMESPACE}.mobilecoin.com/production.json | grep consensus-enclave.css | awk '{print $2}' | tr -d \" | tr -d ,)
    curl -O https://enclave-distribution.${NAMESPACE}.mobilecoin.com/${SIGNED_ENCLAVE_URI}
    ```

1. Build

    ```sh
    SGX_MODE=HW IAS_MODE=PROD CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo build --release
    ```

1. Run

    ```sh
    ./target/release/wallet-service \
        --wallet-db /tmp/wallet-db/wallet.db \
        --ledger-db /tmp/ledger-db/ \
        --peer mc://node1.test.mobilecoin.com/ \
        --peer mc://node2.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node1.test.mobilecoin.com/ \
        --tx-source-url https://s3-us-west-1.amazonaws.com/mobilecoin.chain/node2.test.mobilecoin.com/
    ```

## API

### Accounts

#### Create Account

Create a new account in the wallet.

```sh
curl -s localhost:9090/wallet \
    -d '{
          "method": "create_account",
          "params": {
            "name": "Alice"
          }
        }' \
    -X POST -H 'Content-type: application/json' | jq

{
  "method": "create_account",
  "result": {
    "public_address": "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU",
    "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
    "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
  }
}
```

#### Import Account

Import an existing account from the secret entropy.

```sh
curl -s localhost:9090/wallet \
    -d '{
          "method": "import_account",
          "params": {
            "entropy": "c593274dc6f6eb94242e34ae5f0ab16bc3085d45d49d9e18b8a8c6f057e6b56b",
            "name": "Alice"
          }
        }' \
     -X POST -H 'Content-type: application/json' | jq

{
 "method": "import_account",
 "result": {
   "public_address": "8JtpPPh9mV2PTLrrDz4f2j4PtUpNWnrRg8HKpnuwkZbj5j8bGqtNMNLC9E3zjzcw456215yMjkCVYK4FPZTX4gijYHiuDT31biNHrHmQmsU",
   "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
 }
}
```

#### List Accounts

```sh
curl -s localhost:9090/wallet \
    -d '{"method": "list_accounts"}' \
    -X POST -H 'Content-type: application/json' | jq

{
  "method": "list_accounts",
  "result": {
    "accounts": [
      "c7155cb1660f6dfe778dd52f6381ad3a25f35bd9f502ec337b17478f51abaade",
      "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
    ]
  }
}
```

#### Get Account

```sh
curl -s localhost:9090/wallet \
    -d '{
          "method": "get_account",
          "params": {
            "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
          }
        }' \
    -X POST -H 'Content-type: application/json'  | jq

{
  "method": "get_account",
  "result": {
    "name": "Alice",
    "balance": "0"
  }
}
```

#### Update Account Name

```sh
curl -s localhost:9090/wallet \
    -d '{
          "method": "update_account_name",
          "params": {
            "acount_id": "2b2d5cce6e24f4a396402fcf5f036890f9c06660f5d29f8420b8c89ef9074cd6",
            "name": "Eve"
          }
        }' \
    -X POST -H 'Content-type: application/json'  | jq
{
  "method": "update_account_name",
  "result": {
    "success": true
  }
}
```

#### Delete Account

```sh
curl -s localhost:9090/wallet \
    -d '{
          "method": "delete_account",
          "params": {
            "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
          }
        }' \
    -X POST -H 'Content-type: application/json' | jq

{
  "method": "delete_account",
  "result": {
    "success": true
  }
}
```

### TXOs

#### List TXOs for a given account

```sh
curl -s localhost:9090/wallet \
    -d '{
          "method": "list_txos",
          "params": {
            "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
          }
        }' \
     -X POST -H 'Content-type: application/json'  | jq

{
  "method": "list_txos",
  "result": {
    "txos": [
      {
        "txo_id": "000d688cfe28ab128a7514148f700dc6872e97c1498753fdef4fdd8b90601cd1",
        "value": "97582349900010990",
        "txo_type": "received",
        "txo_status": "spent"
      },
      {
        "txo_id": "00a92e639f2601e9af3ba796c62087cc1c6b9d1bc7c4921df4b136d134ff4027",
        "value": "1",
        "txo_type": "received",
        "txo_status": "spent"
      },
      {
        "txo_id": "00ae2c1a638296dbfe0514019e4efa03b0c714c45b391f1d2180a2c50a38ffad",
        "value": "1",
        "txo_type": "received",
        "txo_status": "spent"
      },
      {
        "txo_id": "00d4f35588ed694edaf58762be9edf3a3cb6941f2a9de3ee779f7c91c3a064a0",
        "value": "97584329900010990",
        "txo_type": "received",
        "txo_status": "spent"
      },
    ]
  }
}
```

#### Get Balance for a given account

```
curl -s localhost:9090/wallet \
    -d '{
          "method": "get_balance",
          "params": {
             "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde"
          }
        }' \
    -X POST -H 'Content-type: application/json' | jq

{
  "method": "get_balance",
  "result": {
    "balance": "97580449900010991"
  }
}
```

### Transactions

#### Build Transaction

You can build a transaction to confirm its contents before submitting it to the network.

```
curl -s localhost:9090/wallet \
    -d '{
          "method": "build_transaction",
          "params": {
            "account_id": "a8c9c7acb96cf4ad9154eec9384c09f2c75a340b441924847fe5f60a41805bde",
            "recipient_public_address": "CaE5bdbQxLG2BqAYAz84mhND79iBSs13ycQqN8oZKZtHdr6KNr1DzoX93c6LQWYHEi5b7YLiJXcTRzqhDFB563Kr1uxD6iwERFbw7KLWA6",
            "value": "42000000000000"
          }
        }' -X POST -H 'Content-type: application/json' | jq

{
  "method": "build_transaction",
  "result": {
    "tx_proposal": {
      "input_list": [
        {
          "tx_out": {
            "amount": {
              "commitment": "629abf4112819dadfa27947e04ce37d279f568350506e4060e310a14131d3f69",
              "masked_value": "17560205508454890368"
            },
            "target_key": "eec9700ee08358842e16d43fe3df6e346c163b7f6007de4fcf3bafc954847174",
            "public_key": "3209d365b449b577721430d6e0534f5a188dc4bdcefa02be2eeef45b2925bc1b",
            "e_fog_hint": "ae39a969db8ef10daa4f70fa4859829e294ec704b0eb0a15f43ae91bb62bd9ff58ba622e5820b5cdfe28dde6306a6941d538d14c807f9045504619acaafbb684f2040107eb6868c8c99943d02077fa2d090d0100"
          },
          "subaddress_index": 0,
          "key_image": "2a14381de88c3fe2b827f6adaa771f620873009f55cc7743dca676b188508605",
          "value": "1",
          "attempted_spend_height": 0,
          "attempted_spend_tombstone": 0,
          "monitor_id": ""
        },
        {
          "tx_out": {
            "amount": {
              "commitment": "8ccbeaf28bad17ac6c64940aab010fedfdd44fb43c50c594c8fa6e8574b9b147",
              "masked_value": "8257145351360856463"
            },
            "target_key": "2c73db6b914847d124a93691884d2fb181dfcf4d9182686e53c0464cf1c9a711",
            "public_key": "ce43370def13a97830cf6e2e73020b5190d673bd75e0692cd18c850030cc3f06",
            "e_fog_hint": "6b24ceb038ed5c31bfa8f69c73be59eca46612ba8bfea7f53bc52c97cdf549c419fa5a0b2219b1434848197fdbac7880b3a20d92c59c67ec570c7d60e263b4c7c61164f0517c8f774321435c3ec600593d610100"
          },
          "subaddress_index": 0,
          "key_image": "a66fa1c3c35e2c2a56109a901bffddc1129625e4c4b381389f6be1b5bb3c7056",
          "value": "97580449900010990",
          "attempted_spend_height": 0,
          "attempted_spend_tombstone": 0,
          "monitor_id": ""
        }
      ],
      "outlay_list": [
        {
          "value": "42000000000000",
          "receiver": {
            "view_public_key": "5c04cc0de88725f811625b56844aacd789815d43d6df30354939aafd6e683d1a",
            "spend_public_key": "aaf2937c73ef657a529d0f10aaaba394f41bf6f67d8da5ae13284afdb5bc657b",
            "fog_report_url": "",
            "fog_authority_fingerprint_sig": "",
            "fog_report_id": ""
          }
        }
      ],
      "tx": {
        "prefix": {
          "inputs": [
            {
              "ring": [
                {
                  "amount": {
                    "commitment": "3c90eb914a5fe5eb11fab745c9bebfd988de71fa777521099bd442d0eecb765a",
                    "masked_value": "5446626203987095523"
                  },
                  "target_key": "f23c5dd112e5f453cf896294be705f52ee90e3cd15da5ea29a0ca0be410a592b",
                  "public_key": "084c6c6861146672eb2929a0dfc9b9087a49b6531964ca1892602a4e4d2b6d59",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                },
                {
                  "amount": {
                    "commitment": "84dd0a227e497800ba43a05d9e5590eb9e639df5cfd7721c83f826fbbf4b3607",
                    "masked_value": "14735696069314613294"
                  },
                  "target_key": "de433e5438e4b5f0bf04b8315122d2a366996439bcb4f6dcfc45ad6d66e43b65",
                  "public_key": "208f603f433874b5d22feb0a3fb8b6786be271d4ba8e56b41abd88f67df9617a",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                },
                {
                  "amount": {
                    "commitment": "8abefb58b2ecdf859f0ad6cb562c0c2d480444303941a4e4d0e776310d4d3269",
                    "masked_value": "15366570252085605121"
                  },
                  "target_key": "d0c17b3f613a186d4d7dc64060a871a7a7762b93961ff6f469e417de10c0077f",
                  "public_key": "425a787121d7fd4e73d23d71f00f57824d33e807b31d65d86ca086d818fd550f",
                  "e_fog_hint": "98e37814657c1edbffa28c9cb1fb9a742a3c4abe87cb37f6ca4bb582274e600845ca8ced926eeabaf222155336074628532468b853e53eaf6cc1c2a81fa54583ef6fa2de1c93cb6d37559c3a2b8d8ba4fe830100"
                },
                {
                  "amount": {
                    "commitment": "844d17daca3f83e93a29f50e8f488d0024eb7246c3f0845fe6e99de986f6783f",
                    "masked_value": "12041743348595042820"
                  },
                  "target_key": "408a61c91b4e2dc5785c94daf27a478f70631edc04b6f412091bbd57ca60d145",
                  "public_key": "48ee5cc642de8c80906131c81975f244bdbe0819e09441b3a6188f08464ced58",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                },
                {
                  "amount": {
                    "commitment": "36e770cb8d71200cabbabcc40c96e68f7a6b1c9f9d737523ce64b8d5d890a44c",
                    "masked_value": "839934401543324873"
                  },
                  "target_key": "9c0deff66f4986cd029bd8e5fc50431491a77448fafabf051e86858cc8d30862",
                  "public_key": "68c28f99557efaa97cba50406f9f42478217cd4983630cb4ead0b16aaec60868",
                  "e_fog_hint": "22e769472ebf27f8d57984ac7f69f5e9e5b02f6cb30892c334a2ab7aafd57b88cad58291bd44784a895342ff9e6883a863967b184b865c76d453fc30fefbf312b126ba7249ac590bf0035f060231826c8dbf0100"
                },
                {
                  "amount": {
                    "commitment": "3c09c7b9b1b87ad4ca31486eefb849b105f197396f913633936611e331b0432e",
                    "masked_value": "6980360646797576258"
                  },
                  "target_key": "f02988f4b8be25296ecf019a73c7a90b31c5517630727a07979612c3b4e9ae12",
                  "public_key": "78a174e1f95d8789ba741a3def8f7a3a986c4e16fdbbf5ce84932f524e882e36",
                  "e_fog_hint": "a9281d9b8c3d0c6456a35af59092fbadd3b5be00e4e6a239c2c89b12d8b06868dff9c603dfc6e23703976fd8489a4703134d72d76810613b9dd0721677d4ea97d406aa0043434589174396a9b1deddb95d2d0100"
                },
                {
                  "amount": {
                    "commitment": "3c5c9fac8844cdce64080390c758bc849fa7222a66c65182951d8207f1ddcc17",
                    "masked_value": "18070157952463432169"
                  },
                  "target_key": "8e5439f66eec823a6a764fb855fe7018456fe2e0738918506ff34169a3972b65",
                  "public_key": "8a065bb92cf1121543ccddc7925efdfe11c7888c9b19eb4c89450879c60ceb2f",
                  "e_fog_hint": "f7c100c70ccdaab046ef80a46ec1b0637e2b8027109c7f406876303f20dc90ea867b3a2c4f91d1c1a917a23171bb820784e57881b8faf73311233fff2387eb96b3455ece791dd84289fcd839d2437faf358a0100"
                },
                {
                  "amount": {
                    "commitment": "58968b4080a1ab643e26b7514742139508dc03ebc8480fcd35b1df163f0bfe13",
                    "masked_value": "11514381323161746054"
                  },
                  "target_key": "c898adce86d8192196b72b39a591f8d51e0ba1328a505fadb5431e2cba33765e",
                  "public_key": "a2edc63ea9b3b385b9a165bfcdae66a9fda9997c2aa3f3d9387746857b4ffe12",
                  "e_fog_hint": "87334b1d941adcdaf4399cbb70e0e96a1534ba58dedb7c4f71a2218b972c735f15b5826a4969139be16d7e00b299dc65c179b75983ac3b7c2960516d39f4f4d0a35863d24ed0a8acfee007c3107833a292740100"
                },
                {
                  "amount": {
                    "commitment": "e2aaf14615ac602db4dbdefec015ae5c9bdef49b8ac616e6bb2daf0110b65061",
                    "masked_value": "1161724165778401534"
                  },
                  "target_key": "94e13370e55747da32bd40d3942b372a0a399fc0e016583ea66c149bd76e9659",
                  "public_key": "bacdfaf8674fc903fd6661cdd373d9da52687dc25dbeedfebbbc4675bc793360",
                  "e_fog_hint": "d71edf95d5f719d9030a77916bc671a63ceb28c119ef5fc61ab95ca5e31daf2d858d8892b47fa352204eafede634298cbd7428c4c4aa99b4ac00ae76b2f2970c4409a260598ac601ab6bf69cbcc6e9fc5c9a0100"
                },
                {
                  "amount": {
                    "commitment": "825ef5dac2bb1feb8e6371639875e92a060c71a0478378af8104a24cec761d6b",
                    "masked_value": "17689612350133478841"
                  },
                  "target_key": "90790d561d2855b49daa52a603c21422cd47636b8d23f86bd62dd1808beddc3a",
                  "public_key": "c6b7c846e11cd994254ba71c3917775f546d5339ccfc399e59fbd5f2a0e4692e",
                  "e_fog_hint": "45a5921d24e83f78229f0436f0bdf8e8f82d2d318dae644c1c2648036abfdf03b3fc0c8fce087920fe7e54109d30456fbaa56cf869212fecc6edc7252f52e45b615c8412801d18814706c51bfcfceb6036190100"
                },
                {
                  "amount": {
                    "commitment": "8ccbeaf28bad17ac6c64940aab010fedfdd44fb43c50c594c8fa6e8574b9b147",
                    "masked_value": "8257145351360856463"
                  },
                  "target_key": "2c73db6b914847d124a93691884d2fb181dfcf4d9182686e53c0464cf1c9a711",
                  "public_key": "ce43370def13a97830cf6e2e73020b5190d673bd75e0692cd18c850030cc3f06",
                  "e_fog_hint": "6b24ceb038ed5c31bfa8f69c73be59eca46612ba8bfea7f53bc52c97cdf549c419fa5a0b2219b1434848197fdbac7880b3a20d92c59c67ec570c7d60e263b4c7c61164f0517c8f774321435c3ec600593d610100"
                }
              ],
              "proofs": [
                {
                  "index": "24296",
                  "highest_index": "335531",
                  "elements": [
                    {
                      "range": {
                        "from": "24296",
                        "to": "24296"
                      },
                      "hash": "f7217a219665b1dfa3f216191de1c79e7d62f520e83afe256b6b43c64ead7d3f"
                    },
                  }
                  ...
                  ]
                },
                ...
              ]
            },
            {
              "ring": [
                {
                  "amount": {
                    "commitment": "50b46eef8d223824f87316e6f446d50530929c8a758195005fbe9d41ec7fc227",
                    "masked_value": "11687342289991185016"
                  },
                  "target_key": "241d533daf32ed1523561c96c618808a2db9635075776ef42da32b34e7586058",
                  "public_key": "24725d8e47e4b03f6cb893369cc7582ea565dbd5e1914a5ecb3f4ed7910c5a03",
                  "e_fog_hint": "3fba73a6271141aae115148196ad59412b4d703847e0738c460c4d1831c6d44004c4deee4fabf6407c5f801703a31a13f1c70ed18a43a0d0a071b863a529dfbab51634fdf127ba2e7a7d426731ba59dbe3660100"
                },
                {
                  "amount": {
                    "commitment": "629abf4112819dadfa27947e04ce37d279f568350506e4060e310a14131d3f69",
                    "masked_value": "17560205508454890368"
                  },
                  "target_key": "eec9700ee08358842e16d43fe3df6e346c163b7f6007de4fcf3bafc954847174",
                  "public_key": "3209d365b449b577721430d6e0534f5a188dc4bdcefa02be2eeef45b2925bc1b",
                  "e_fog_hint": "ae39a969db8ef10daa4f70fa4859829e294ec704b0eb0a15f43ae91bb62bd9ff58ba622e5820b5cdfe28dde6306a6941d538d14c807f9045504619acaafbb684f2040107eb6868c8c99943d02077fa2d090d0100"
                },
                {
                  "amount": {
                    "commitment": "50eac555171551f62cd74a7fc71f12e7f9027ac02210301ff9f22ad559829360",
                    "masked_value": "4139853849778416114"
                  },
                  "target_key": "46faf3ba187e6a08a6361697e836555e5d51ac9e655b2f263a6ecdf9c649d604",
                  "public_key": "4033a351cb27ac1267cb43ce3798ae944eef7bb2d18ba4fe5f1f762932e7c534",
                  "e_fog_hint": "6cbef6f53ac987a2b0e28a2ecc8c1f2b198b9c3545e71501d5a5454a7faabcf15ddf38f94d5e59932b7ea58ecda397af2a43623e4333838d9c2ad2ab8cbb2096d907c1187c41c6daea58f59b3f0ef6028ce00100"
                },
                {
                  "amount": {
                    "commitment": "402c3a27bc03275f453be05c389fccbde952b9713ab45e7213725351679eaa4f",
                    "masked_value": "13121600085442771776"
                  },
                  "target_key": "1256d3ecfb43caa81977ee1c024a0666a50ff12e1e3e43ee96fbc0f205d98b15",
                  "public_key": "46ca454d59a6c9ee0f371c072e2421bc223e6d57205cf5a75e2ed08bd3ab3445",
                  "e_fog_hint": "af409e483b79b944e16c954dac52e61f269d080dc7a13f38c5ee51b9b2cb10c6462f5471595b347fca9da9b29eb65d3a31f1dc6a8b34fc0c0c4fa76ec8e6107b5f376a6d64acafe6034fc589f7c6a19c26de0100"
                },
                {
                  "amount": {
                    "commitment": "7e785828c27ba9f9570b90a69240df94547d2bb2abb33f33fa7b62d2ae1a9d3a",
                    "masked_value": "2194809152766895478"
                  },
                  "target_key": "b2c533a7ed92dfc575f8ceca3328c6ac42dbd3db71b40e4c432d961dda980c2e",
                  "public_key": "4ac4bfe1f73cd0d6c999d6de90b0698531ae64f6fbfab4e66101817010a3224f",
                  "e_fog_hint": "975f84bc3327f120e629175030b6990773a36e7b7ac0e63baaba6266e8507136f7ff80f38fd779b9f251543a073d68e0db94e6f777b7715f2b1fe475989f0ce2f2364d86c7628a45e8f4816808bc6a7cc7ac0100"
                },
                {
                  "amount": {
                    "commitment": "68542e0852eeb5447e47ebc7aa614077343e0524f57321f6abc8d37868a39678",
                    "masked_value": "11238401550444343835"
                  },
                  "target_key": "3a0f787e9ed8ba64bccafa8a05aa2bc083e3046a87195ece6812cca26aad4415",
                  "public_key": "66271476f85928303401d511198612dc55091288dbbe53602c194aa567e37e0d",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                },
                {
                  "amount": {
                    "commitment": "1eeafbe3e8b2ae7dce21ee9c786b63aaeca64a28f61f35b77ecb35d865975e7b",
                    "masked_value": "10554138114892106222"
                  },
                  "target_key": "0431b361a24689031576cef91a73f387c8a6e3220f713d2ba9fc3b602b8dc266",
                  "public_key": "88ad8e26418f803990d4a096ad332949a001abfa2ec9f08583a433447b3b6474",
                  "e_fog_hint": "015c31354d37624e67179e9712bd7b1953ad1fbb79981ad8b9a6a7cd5c710c3fadb8b0479f5c6fa174c24e3aedda2662d62200f791e157bceaa549fe1234d96a8b2cf60d2ee64fd90bdf67f56dca85851dad0100"
                },
                {
                  "amount": {
                    "commitment": "d07d885ecceb650c80487ec83d1835e1802d9870c00a3d25c9363c19d1c7405c",
                    "masked_value": "9563666937486937886"
                  },
                  "target_key": "1ce2f4bbb62805ca225e082011195fd107bca6ca98c10dec09cd10f471a0aa3a",
                  "public_key": "921a1003672fc9d2e68f3b7dfda5771c0b3583a809c1672f97e01c421b2bea05",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                },
                {
                  "amount": {
                    "commitment": "0a66bd04bf45dbc343cbe8d1f80830d2e18a5a1327dca71cb2eadddfc0b64c27",
                    "masked_value": "9142853588857215074"
                  },
                  "target_key": "d66d76809966e52dcdb8ef8ee86bf22cc538f3810977cefe6389be81df982252",
                  "public_key": "a42fc5d71fe481850f8e2af7752df2006418ba8b6db90d69dcb4bb4548a3283f",
                  "e_fog_hint": "ddc32719a205ae90ccd775fc8e186ac741c2d590bee2d93802c07fdddc00ce9f750daacbc9047e1466ce7cb3d2e2807ccada59bc2efb29198a9a0885977182d82641726c86903192e88ca8cd9c9cbe99c81d0100"
                },
                {
                  "amount": {
                    "commitment": "2e0d2b17b8ed75b7968851a8147a8c1af6c8f499552a0730a0fed0e47bce9d36",
                    "masked_value": "16149192169306033358"
                  },
                  "target_key": "f4c6828c0249c5006af6e72a6c0d164d9a4dd1cf89568e9207da3e0272fd041e",
                  "public_key": "a6ad544ac6e9684c7baa4ba032129e36b26d85cf4c988040a55ff717d3157213",
                  "e_fog_hint": "18c7bddfbb52071ca81032020f40f883df0a1d2e1bd641422293920e2c68a92816d9dac420e43ec971f59c0f3daa73a1b5ec5fd89d30beedd2b449dbf9d1a131d9513033c6a88206e33f61da1ea77c6a307f0100"
                },
                {
                  "amount": {
                    "commitment": "60cfbb2f4364001c876539503ddab4b0bb1e26663a9b1fe32c31b1353839261e",
                    "masked_value": "12552771872975208719"
                  },
                  "target_key": "eeb437ff1034ef1feaddb14dc5eb696ae66b4261392eaa4ae33e421855766713",
                  "public_key": "ee67e3ba8a28b459386e6b77db51281aa6388e7f32cc1289020423412bcc9466",
                  "e_fog_hint": "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                }
              ],
              "proofs": [
                {
                  "index": "173379",
                  "highest_index": "335531",
                  "elements": [
                    {
                      "range": {
                        "from": "173379",
                        "to": "173379"
                      },
                      "hash": "bcb26ff5d1104b8c0d7c9aed9b326c824151461257737e0fc4533d1a39e3a876"
                    },
                    ...
                  ]
                },
                ...
              ]
            }
          ],
          "outputs": [
            {
              "amount": {
                "commitment": "147113bbd5d4fdc5f9266ccdec6d6e6148e8dbc979d7d3bab1a91e99ab256518",
                "masked_value": "3431426060591787774"
              },
              "target_key": "2c6a9c23810e91d8c504dd4fe59f07c2872a8a866c160a58928750eab7328c64",
              "public_key": "0049281368c270eb5a7291fb012e95e776a07c1ff4336be1aa6a61abb1868229",
              "e_fog_hint": "eb5b104677df5bbc22f70027646a448dcffb61eb31580d50f41cb487a87a9545d507d4c5e13a22f7fe3b2daea3f951b8d9901e73794d24650176faca3251dd904d7cac97ee73f50a84701cb4c297b31cbdf80100"
            },
            {
              "amount": {
                "commitment": "78083af2c1682f765c332c1c69af4260a410914962bddb9a30857a36aed75837",
                "masked_value": "17824177895224156943"
              },
              "target_key": "68a193eeb7614e3dec6e980dfab2b14aa9b2c3dcaaf1c52b077fbbf259081d36",
              "public_key": "6cdfd36e11042adf904d89bcf9b2eba950ad25f48ed6e877589c40caa1a0d50d",
              "e_fog_hint": "c0c9fe3a43e237ad2f4ab055532831b95f82141c69c75bc6e913d0f37633cb224ce162e59240ffab51054b13e451bfeccb5a09fa5bfbd477c5a8e809297a38a0cb5233cc5d875067cbd832947ae48555fbc00100"
            }
          ],
          "fee": "10000000000",
          "tombstone_block": "0"
        },
        "signature": {
          "ring_signatures": [
            {
              "c_zero": "27a97dbbcf36257b31a1d64a6d133a5c246748c29e839c0f1661702a07a4960f",
              "responses": [
                "bc703776fd8b6b1daadf7e4df7ca4cb5df2d6498a55e8ff15a4bceb0e808ca06",
                ...
              ],
              "key_image": "a66fa1c3c35e2c2a56109a901bffddc1129625e4c4b381389f6be1b5bb3c7056"
            },
            {
              "c_zero": "421cc5527eae6519a8f20871996db99ffd91522ae7ed34e401249e262dfb2702",
              "responses": [
                "322852fd40d5bbd0113a6e56d8d6692200bcedbc4a7f32d9911fae2e5170c50e",
                ...
              ],
              "key_image": "2a14381de88c3fe2b827f6adaa771f620873009f55cc7743dca676b188508605"
            }
          ],
          "pseudo_output_commitments": [
            "1a79f311e74027bdc11fb479ce3a5c8feed6794da40e6ccbe45d3931cb4a3239",
            "5c3406600fbf8e93dbf5b7268dfc43273f93396b2d4976b73cb935d5619aed7a"
          ],
          "range_proofs": [
            ...
          ]
        }
      },
      "fee": 10000000000,
      "outlay_index_to_tx_out_index": [
        [
          0,
          0
        ]
      ],
      "outlay_confirmation_numbers": [
        [...]
      ]
    }
  }
}
```

## Contributing

### Database Schema

To add or edit tables:

1. Create a migration with `diesel migration generate <migration_name>`
1. Edit the migrations/<migration_name>/up.sql and down.sql.
1. Run the migration with `diesel migration run`, and test delete with `diesel migration redo`

### Running Tests

    FIXME: I'm not sure why we need to provide these vars for cargo test...

    ```
    SGX_MODE=HW IAS_MODE=DEV CONSENSUS_ENCLAVE_CSS=$(pwd)/consensus-enclave.css cargo test
    ```