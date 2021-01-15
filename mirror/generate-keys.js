// Minimum supported NodeJS version: v12.9.0
const NODE_MAJOR_VERSION = process.versions.node.split('.')[0];
if (NODE_MAJOR_VERSION < 12) {
  throw new Error('Requires Node 12 (or higher)');
}

// Imports
const fs = require('fs');
const crypto = require('crypto');

const { privateKey, publicKey } = crypto.generateKeyPairSync('rsa', {
    modulusLength: 4096,
    publicKeyEncoding: {
        type: 'pkcs1',
        format: 'pem',
    },
    privateKeyEncoding: {
        type: 'pkcs1',
        format: 'pem',
        cipher: 'aes-256-cbc',
        passphrase: '',
    },
})

fs.writeFileSync('mirror-client.pem', privateKey, {mode:0o600})
console.log('Wrote mirror-client.pem - use this file with a client, see example-client.js for example')

fs.writeFileSync('mirror-private.pem', publicKey)
console.log('Write mirror-private.pem - use this file with the private side of the mirror. See README.md for more details')
