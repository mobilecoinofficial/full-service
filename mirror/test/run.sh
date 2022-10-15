#!/bin/bash
set -eu
export RUST_LOG=DEBUG

mnemonic="MNEMONIC HERE"

echo "generate private key for tls"

openssl req -x509 -sha256 -nodes -newkey rsa:2048 -days 365 -keyout server.key -out server.crt -subj "/C=US/ST=CA/L=SF/O=MobileCoin/OU=IT/CN=localhost"

echo "generate keypair for mirror"

./bin/generate-rsa-keypair


echo "Calling wallet-service-mirror-public-tls.sh. This starts the public mirror."
./wallet-service-mirror-public-tls.sh \
    > /tmp/mobilecoin-public-mirror.log 2>&1 &


echo "Calling wallet-service-mirror-private-tls-encrypted.sh. This starts the validator and full service as well as the private mirror."
./wallet-service-mirror-private-tls-encrypted.sh localhost \
    > /tmp/mobilecoin-private-mirror.log 2>&1 &


echo "Checking that these methods are unsupported."
declare -a MethodList=("assign_address_for_account" "build_and_submit_transaction" "build_gift_code" "build_split_txo_transaction" "build_transaction" "check_b58_type" "check_gift_code_status" "check_receiver_receipt_status" "claim_gift_code" "create_account"  "create_receiver_receipts" "export_account_secrets" "get_all_addresses_for_account" "get_all_gift_codes" "get_all_transaction_logs_for_account" "get_all_transaction_logs_ordered_by_block" "get_all_txos_for_account" "get_all_txos_for_address" "get_gift_code" "get_mc_protocol_transaction" "get_mc_protocol_txo" "get_txo" "get_txos_for_account" "import_account" "import_account_from_legacy_root_entropy" "remove_account" "remove_gift_code" "submit_gift_code" "submit_transaction" "update_account_name")
for method in ${MethodList[@]}; do
response=$(node example-client.js 0.0.0.0 9091 mirror-client.pem "{
    \"method\": \"${method}\",
    \"params\": {},
    \"jsonrpc\": \"2.0\", 
    \"api_version\": \"2\", 
    \"id\": 1
    }")
if [ "$response" != 'Http error, status: 400: Unsupported request' ] 
then
echo "$method return $response which was not 'Http error, status: 400: Unsupported request'"
exit 42
fi
done
echo "Unsupported methods have been verified, testing support methods"

response=$(node ./test_suite/test_script.js 0.0.0.0 9091 127.0.0.1 5554 mirror-client.pem "${mnemonic}")
echo "Test result: $response"    

exit 0


