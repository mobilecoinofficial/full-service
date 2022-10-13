FROM ubuntu:18.04

ENV mirversion="1.5.0"

RUN apt-get update && apt-get -yy install curl openssl

# Install NVM
SHELL ["/bin/bash", "-c"]
# nvm environment variables
ENV NVM_DIR /usr/local/nvm
ENV NODE_VERSION 16.14.0

# install nvm
# https://github.com/creationix/nvm#install-script
RUN curl --silent -o- https://raw.githubusercontent.com/creationix/nvm/v0.31.2/install.sh | bash

# install node and npm
RUN source $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default

# add node and npm to path so the commands are available
ENV NODE_PATH $NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

# confirm installation
RUN node -v
RUN npm -v
SHELL ["/bin/sh", "-c"]

# Download and install the full service mirror package
RUN curl -L -o full-service-mirror.tar.gz https://github.com/mobilecoinofficial/full-service-mirror/releases/download/v$mirversion/linux-v$mirversion-testnet.tar.gz
RUN tar xf full-service-mirror.tar.gz && rm full-service-mirror.tar.gz
WORKDIR /linux-v$mirversion-testnet

# Copy the useful test scripts into the docker image.
COPY ./run.sh .
COPY ./test_lib ./test_lib
COPY ./test_suite ./test_suite
WORKDIR /linux-v$mirversion-testnet/test_lib
RUN npm install
WORKDIR /linux-v$mirversion-testnet/test_suite
RUN npm install
WORKDIR /linux-v$mirversion-testnet

# Run the script
ENTRYPOINT ["/bin/bash", "./run.sh"]
