// Minimum supported NodeJS version: v12.9.0
const NODE_MAJOR_VERSION = process.versions.node.split('.')[0];
if (NODE_MAJOR_VERSION < 12) {
    throw new Error('Requires Node 12 (or higher)');
}

// Imports
const http = require('http');
const fs = require('fs');
const crypto = require('crypto');
const KEY_SIZE = 512;


function sendRequestPromise(params, postData, processSuccessfulResponse) {
    return new Promise(function(resolve, reject) {
        var req = http.request(params, (response) => {
            // cumulate data
            var buf = [];
            var result;
            response.on('data', function(chunk) {
                buf.push(chunk);
            });
            // resolve on end
            response.on('end', function() {
                if (response.statusCode == 200) {
                try {
                    result = processSuccessfulResponse(buf);
                } catch(error) {
                    reject(`Failed to process a successful request: ${error}`);
                }
                resolve(result);
                } else {
                    reject (`Http error, status: ${response.statusCode}: ${buf}`);
                }
            });
            //reject on response error
            response.on('error', (error) => {
                reject(`Error reading response: ${error}`);
            });
            
        });
        // reject on request error
        req.on('error', function(error) {
            reject(`Error sending request: ${error}`);
        });        
        req.write(postData);
        req.end();
    });
}



function sendRequest(host, port, key_file, request) {
    return sendRequestEncrypted(host, port, "/encrypted-request", key_file, request);
}


function sendRequestEncrypted(host, port, path, key_file, msg) {
    // Load key
    let key_bytes = fs.readFileSync(key_file)
    if (!key_bytes) {
        throw 'Failed loading key';
    }
    let key = crypto.createPublicKey(key_bytes);
    if (!key) {
        throw 'Failed creating key';
    }


    // Ensure the key is 4096 bits (outputs 512-byte chunks).
    let test_data = encrypt(key, [1, 2, 3]);
    if (test_data.length != KEY_SIZE) {
        throw `Key is not 4096-bit, encrypted output chunk size returned was ${test_data.length}`;
    }

    // Prepare request
    let encrypted_msg = encrypt(key, msg);
    
    // Send request to server
    let params = {
        host: host,
        port: port,
        timeout: 120000,
        path: path,
        method: 'POST',
        headers: {
            'Content-Type': 'application/octet-stream',
            'Content-Length': Buffer.byteLength(encrypted_msg)
        }
    };
    let processSuccessfulResponse = (buf) => {
        let result = decrypt(key, Buffer.concat(buf)).toString();
        console.log(result);
        return result;
    };   
    return sendRequestPromise(params, encrypted_msg, processSuccessfulResponse);
}

function sendRequestUnencrypted(host, port, path, msg) {
    // Send request to server
    let params = {
        host: host,
        port: port,
        timeout: 120000,
        path: path,
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(msg)
        }
    };
    let processSuccessfulResponse = (buf) => {
            let result = Buffer.concat(buf).toString();
            console.log(result);
            return result;
        };   
    return sendRequestPromise(params, msg, processSuccessfulResponse);
}



// Crypto utilities
function encrypt(key, buf) {
    let res = [];

    // Each encrypted chunk must be no longer than the length of the public modulus minus padding size.
    // PKCS1 is 11 bytes of padding (which is also defined as PKCS1_PADDING_LEN in the rust code).
    const MAX_CHUNK_SIZE = KEY_SIZE - 11;

    while (buf.length > 0) {
        let data = buf.slice(0, MAX_CHUNK_SIZE);
        buf = buf.slice(data.length);

        res.push(crypto.publicEncrypt({
            key: key,
            padding: crypto.constants.RSA_PKCS1_PADDING,
        }, Buffer.from(data)));
    }

    return Buffer.concat(res)
}

function decrypt(key, buf) {
    let res = [];

    while (buf.length > 0) {
        let data = buf.slice(0, KEY_SIZE);
        buf = buf.slice(data.length);

        res.push(crypto.publicDecrypt({
            key,
            padding: crypto.constants.RSA_PKCS1_PADDING,
        }, Buffer.from(data)));
    }

    return Buffer.concat(res)
}

function sign(buf) {
    return crypto.sign(null, Buffer.from(buf), { key, passphrase: '' })
}

exports.sendRequest = sendRequest;
exports.sendRequestUnencrypted = sendRequestUnencrypted;
exports.sendRequestEncrypted = sendRequestEncrypted;
