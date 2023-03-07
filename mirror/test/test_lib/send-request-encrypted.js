// Minimum supported NodeJS version: v12.9.0
const NODE_MAJOR_VERSION = process.versions.node.split('.')[0]
if (NODE_MAJOR_VERSION < 12) {
  throw new Error('Requires Node 12 (or higher)')
}

// Imports
const http = require('http')
const fs = require('fs')
const crypto = require('crypto')

// Logger
const pino = require('pino')
const pretty = require('pino-pretty')
const logLevel = process.env.LOG_LEVEL || 'info'
const logger = pino({ level: logLevel }, pretty())

const KEY_SIZE = 512

class resError extends Error {
  constructor (message) {
    super(message)
    this.name = this.constructor.name
  }
}

class ResponseError extends resError {
  constructor (response) {
    super('Error in successful http response')
    this.name = this.constructor.name
    this.response = response
  }
}

function sendRequestPromise (url, params, postData, processSuccessfulResponse) {
  return new Promise(function (resolve, reject) {
    const req = http.request(url, params, (response) => {
      // cumulate data
      const buf = []
      let result
      response.on('data', function (chunk) {
        buf.push(chunk)
      })
      // resolve on end
      response.on('end', function () {
        if (response.statusCode === 200) {
          try {
            result = processSuccessfulResponse(buf)
          } catch (error) {
            reject(new Error('Failed to process a successful request', { cause: error }))
          }
          resolve(result)
        } else {
          reject(new Error(`Http error, status: ${response.statusCode}: ${buf}`))
        }
      })
      // reject on response error
      response.on('error', (error) => {
        reject(new Error('Error reading response', { cause: error }))
      })
    })
    // reject on request error
    req.on('error', function (error) {
      reject(new Error('Error sending request', { cause: error }))
    })
    req.write(postData)
    req.end()
  })
}

function sendRequest (url, keyFile, request) {
  return sendRequestEncrypted(url, '/encrypted-request', keyFile, request)
}

function sendRequestEncrypted (url, path, keyFile, msg) {
  // Load key
  const keyBytes = fs.readFileSync(keyFile)
  if (!keyBytes) {
    return Promise.reject(new Error('Failed loading key'))
  }
  const key = crypto.createPublicKey(keyBytes)
  if (!key) {
    return Promise.reject(new Error('Failed creating key'))
  }

  // Ensure the key is 4096 bits (outputs 512-byte chunks).
  const testData = encrypt(key, [1, 2, 3])
  if (testData.length !== KEY_SIZE) {
    return Promise.reject(new Error(`Key is not 4096-bit, encrypted output chunk size returned was ${testData.length}`))
  }

  // Prepare request
  const encryptedMsg = encrypt(key, msg)

  // Send request to server
  const params = {
    timeout: 120000,
    path,
    method: 'POST',
    headers: {
      'Content-Type': 'application/octet-stream',
      'Content-Length': Buffer.byteLength(encryptedMsg)
    }
  }
  const processSuccessfulResponse = (buf) => {
    const result = decrypt(key, Buffer.concat(buf)).toString()
    const resultJSON = JSON.parse(result)
    const log = JSON.stringify({ response: resultJSON }, null, 4)
    logger.debug(log)
    if (resultJSON.error) {
      return Promise.reject(new ResponseError(resultJSON))
    }
    return result
  }
  return sendRequestPromise(url, params, encryptedMsg, processSuccessfulResponse)
}

function sendRequestUnencrypted (url, path, msg) {
  // Send request to server
  const params = {
    timeout: 120000,
    path,
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(msg)
    }
  }
  const processSuccessfulResponse = (buf) => {
    const result = Buffer.concat(buf).toString()
    const resultJSON = JSON.parse(result)
    const log = JSON.stringify({ response: resultJSON }, null, 4)
    logger.debug(log)
    if (resultJSON.error) {
      return Promise.reject(new ResponseError(resultJSON))
    }
    return result
  }
  return sendRequestPromise(url, params, msg, processSuccessfulResponse)
}

// Crypto utilities
function encrypt (key, buf) {
  const res = []

  // Each encrypted chunk must be no longer than the length of the public modulus minus padding size.
  // PKCS1 is 11 bytes of padding (which is also defined as PKCS1_PADDING_LEN in the rust code).
  const MAX_CHUNK_SIZE = KEY_SIZE - 11

  while (buf.length > 0) {
    const data = buf.slice(0, MAX_CHUNK_SIZE)
    buf = buf.slice(data.length)

    res.push(crypto.publicEncrypt({
      key,
      padding: crypto.constants.RSA_PKCS1_PADDING
    }, Buffer.from(data)))
  }

  return Buffer.concat(res)
}

function decrypt (key, buf) {
  const res = []

  while (buf.length > 0) {
    const data = buf.slice(0, KEY_SIZE)
    buf = buf.slice(data.length)

    res.push(crypto.publicDecrypt({
      key,
      padding: crypto.constants.RSA_PKCS1_PADDING
    }, Buffer.from(data)))
  }

  return Buffer.concat(res)
}

module.exports = {
  ResponseError,
  sendRequest,
  sendRequestUnencrypted,
  sendRequestEncrypted
}
