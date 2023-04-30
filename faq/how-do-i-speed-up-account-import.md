# How do I speed up account import?

When
calling [`import_account`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/account/account/import_account),
scanning starts at the Origin block (#1). By using the **first_block_index** param, a user can specify the starting
block for that account if it is known. Newly created accounts are given a starting block index and any transactions that
occurred before the **first_block_index** will not be scanned.

###

\
\
