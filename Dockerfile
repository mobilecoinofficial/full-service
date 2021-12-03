# syntax=docker/dockerfile:1.2

# Full Build - This container will do a full compile and create a minimal runtime image.

# This build requires BuildKit

# To build testnet
#   DOCKER_BUILDKIT=1 docker build -t mobilecoin/full-service:0.0.0-testnet \
#   --progress=plain --build-arg NAMESPACE=test --build-arg BUILD_OPTS=--no-default-features .

# To build mainnet
#   DOCKER_BUILDKIT=1 docker build -t mobilecoin/full-service:0.0.0 \
#   --progress=plain --build-arg NAMESPACE=prod .

# Build Args:
# BUILD_OPTS: - '--no-default-features' - Additonal options to cargo build command
# NAMESPACE: - test|prod - specifies enclave.css files to download.
# SGX_MODE: - HW|SW - See README
# IAS_MODE: - PROD|DEV - See README
# RUSTFLAGS: - '-C target-cpu=penryn' - RUSTFLAGS Environment Variable
# SIGNED_ENCLAVE_BASE: - enclave-distribution.${NAMESPACE}.mobilecoin.com - base domain for CSS files.
# SIGSTRUCT_JSON_LOCATION: - production.json - Json file where CSS file paths are located.

# IMPORTANT: Do not add or update OS packages or components in the builder section.
# In order to create a consistent and verifiable the build environment, only add
# or update in the mobilecoin/rust-sgx-base image and refer to the image by its hash.

FROM mobilecoin/rust-sgx-base@sha256:cf4ff6d68e937d1f57a8e445a06e257949b515a241e06c32c41820ec697c2ddb as builder

ARG NAMESPACE=test
ARG SIGNED_ENCLAVE_BASE=enclave-distribution.${NAMESPACE}.mobilecoin.com
ARG SIGSTRUCT_JSON_LOCATION=production.json

ENV INGEST_ENCLAVE_CSS=/app/ingest-enclave.css
ENV CONSENSUS_ENCLAVE_CSS=/app/consensus-enclave.css

WORKDIR /app

ADD https://${SIGNED_ENCLAVE_BASE}/${SIGSTRUCT_JSON_LOCATION} /app/${SIGSTRUCT_JSON_LOCATION}

# Get enclave sigstruct
RUN  export CONSENSUS_CSS_URL=$(cat /app/${SIGSTRUCT_JSON_LOCATION} | jq -r .consensus.sigstruct) \
  && export INGEST_CSS_URL=$(cat /app/${SIGSTRUCT_JSON_LOCATION} | jq -r .ingest.sigstruct) \
  && curl https://${SIGNED_ENCLAVE_BASE}/${CONSENSUS_CSS_URL} -o ${CONSENSUS_ENCLAVE_CSS} \
  && curl https://${SIGNED_ENCLAVE_BASE}/${INGEST_CSS_URL} -o ${INGEST_ENCLAVE_CSS}

COPY . /app/

ARG BUILD_OPTS
ARG SGX_MODE=HW
ARG IAS_MODE=PROD
ARG RUSTFLAGS='-C target-cpu=penryn'

# Build full-service
RUN  --mount=type=cache,target=/root/.cargo/git \
     --mount=type=cache,target=/root/.cargo/registry \
     --mount=type=cache,target=/app/target \
     cargo build --release -p mc-full-service ${BUILD_OPTS} \
  && cp /app/target/release/full-service /usr/local/bin/full-service


# This is the runtime container.
# Adding/updating OS will not affect the ability to verify the build environment.
FROM ubuntu:bionic-20210416

RUN  addgroup --system --gid 1000 app \
  && adduser --system --ingroup app --uid 1000 app \
  && mkdir /data \
  && chown app:app /data

RUN  apt-get update \
  && apt-get upgrade -y \
  && apt-get install -y ca-certificates \
  && apt-get clean \
  && rm -r /var/lib/apt/lists \
  && mkdir -p /usr/share/grpc \
  && ln -s /etc/ssl/certs/ca-certificates.crt /usr/share/grpc/roots.pem

COPY --from=builder /usr/local/bin/full-service /usr/local/bin/full-service
COPY --from=builder /app/*.css /usr/local/bin/

USER app
VOLUME /data
EXPOSE 9090

ENV RUST_LOG=info,rustls=warn,hyper=warn,tokio_reactor=warn,mio=warn,want=warn,rusoto_core=error,h2=error,reqwest=error,rocket=error,<unknown>=error
ENV INGEST_ENCLAVE_CSS=/usr/local/bin/ingest-enclave.css
ENV CONSENSUS_ENCLAVE_CSS=/usr/local/bin/consensus-enclave.css

ENTRYPOINT ["/usr/local/bin/full-service", "--wallet-db=/data/wallet.db", "--ledger-db=/data/ledger.db", "--listen-host=0.0.0.0", "--fog-ingest-enclave-css=/usr/local/bin/ingest-enclave.css"]

CMD [ "--help" ]