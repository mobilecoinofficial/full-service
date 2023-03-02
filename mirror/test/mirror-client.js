
const client = require('./test_lib/send-request-encrypted')
const commander = require('commander')

commander
  .name('mirror test script')
  .requiredOption('--public-mirror-url <public-mirror>', 'http[s] url to the public side of the mirror.')
  .requiredOption('--key-file <path>', 'path to client public key pem file')
  .requiredOption('--request <request>', 'json data to pass in as the request body')

commander.parse()
const options = commander.opts()

client.sendRequest(options.publicMirrorUrl, options.keyFile, options.request)
  .catch((error) => {
    console.log(error)
    throw (error)
  })
