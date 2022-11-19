## Wallet Service Mirror

The `wallet-service-mirror` crate consists of two standalone executables, that when used together allow for exposing limited, read-only data from a `full-service` wallet service instance. As explained below, this allows exposing some data from `full-service` from a machine that does not require any incoming connections from the outside world.

The mirror consists of two sides:
   1) A private side. The private side of the mirror runs alongside `full-service` and forms outgoing connections to both `full-service` and to the public side of the mirror. It then proceeds to poll the public side for any requests that should be forwarded to `full-service`, forwards them, and at the next poll opportunity returns any replies. Note how the private side only forms outgoing connections and does not open any listening ports.

   Please Note:
   The set of available requests is defined in the variable `SUPPORTED_ENDPOINTS`, in the [private main file](src/private/main.rs). It is likely you will want to change the `SUPPORTED_ENDPOINTS` to include desired features like sending transactions.
   2) A public side. The public side of the mirror accepts incoming HTTP connections from clients, and poll requests from the private side over GRPC. The client requests are then forwarded over the GRPC channel to the private side, which in turn forwards them to `full-service` and returns the responses.
