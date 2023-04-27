# How do I speed up account import?

When calling [`import_account`](https://mobilecoin.gitbook.io/full-service-api/api-endpoints/v2/account/account/import\_account), scanning starts at the Origin block (#1). By using the **first\_block\_index** param, a user can specify the starting block for that account if it is known. Newly created accounts are given a starting block index and any transactions that occurred before the **first\_block\_index** will not be scanned.

###

\
\
