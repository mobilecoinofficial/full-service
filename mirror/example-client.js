
const client = require('./test/test_lib/send-request-encrypted');

// Command line parsing
if (process.argv.length != 6) {
    console.log(`Usage: node example-client.js <public mirror host> <public mirror port> <key file> <request>`);
    console.log(`For example: node example-client.js 127.0.0.1 9091 mirror-client.pem '{"method": "get_block", "params": {"block_index": "0"}, "jsonrpc": "2.0", "id": 1}'`);
    console.log('To generate keys please run the generate-rsa-keypair binary. See README.md for more details')
    throw "invalid arguments";
}

let public_mirror_host = process.argv[2];
let public_mirror_port = process.argv[3];
let key_file = process.argv[4];
let request = process.argv[5];

client.sendRequest(public_mirror_host, public_mirror_port, key_file, request);
