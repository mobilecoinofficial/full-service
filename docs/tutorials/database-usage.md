# Database Usage

The MobileCoin wallet service stores its data in two places:

* The first database is the ledger database, which is an append-only data store, recording the state of the public ledger. This contains no readable secret information, because all transaction outputs are stored on the ledger in an encrypted format. Its location is specified with the command line option `--ledger-db`.
* The second database is the wallet database, which stores the private keys and private transaction information for the accounts in the wallet. It should be encrypted using our password features, or stored on an encrypted partition. Its location is specified with `--wallet-db`.

## Database Encryption

The wallet database can be encrypted using SQLCipher, an open source plugin for encrypting SQLite databases.

Please keep in mind that there is no way to recover the contents of an encrypted database without the password. Always backup your database password and the recovery mnemonic of your accounts in a secure location.

## Setting and changing the password

In order to enable encryption, set the password for the database with the environment variable `MC_PASSWORD`. In order to re-encrypt the database with a new password, also set the environment variable `MC_CHANGE_PASSWORD`.

To manually set an environment variable without writing it to the shell history, you can do the following:

```text
$ read -rs MC_PASSWORD
$ export MC_PASSWORD
```

## Encrypting an unencrypted database

When migrating an existing plain-text wallet database to the encrypted format, there is a manual process using the SQLCipher command line tool. The following commands open the unencrypted database in `sqlcipher`, export an encrypted copy of the database, then replace the original database with the encrypted copy.

```text
$ cp wallet.db wallet.db.backup
$ sqlcipher wallet.db
sqlite> ATTACH DATABASE 'encrypted.db' AS encrypted KEY '<your password here>';
sqlite> SELECT sqlcipher_export('encrypted');
sqlite> DETACH DATABASE encrypted;
sqlite> .exit
$ mv encrypted.db wallet.db
```

Once you have verified that you can access the encrypted database, then you can safely delete the unencrypted backup `wallet.db.backup`.

For more information, look at the [SQLCipher documentation](https://www.zetetic.net/sqlcipher/sqlcipher-api/#sqlcipher_export) about this process.

