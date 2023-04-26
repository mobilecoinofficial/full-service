# What is the precision of MOB?

The atomic unit for MOB is picoMOB, which is 1e-12. You need u64 to represent MOB, and many frameworks, DBs, and languages top out at u32 or i64. This is why Full-Service json responses are all strings. For i64 issues there is technically no loss of precision, but you need to cast back to u64 when fetching data.

###

\
\
