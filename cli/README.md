# MobileCoin command-line interface
Command line interface and client library for MobileCoin full-service node.


## Installation

```shell
sudo apt install python3-pip
pip3 install .
```

Check that it is installed.
```shell
mobcli -h
```

## Set up environment variables.

Copy the config file to your home directory.
```shell
cp mc_env.sh ~/.mc_env.sh
```

Add the following lines to your .bashrc:
```shell
if [ -f "$HOME/.mc_env.sh" ]; then
    source "$HOME/.mc_env.sh"
fi
```

The CLI sends its requests to the wallet service executable. Download it from https://github.com/mobilecoinofficial/full-service/releases. Copy the file to the correct location.
```shell
cp full-service ~/.mobilecoin/testnet/full-service-testnet
```

The environment variables file specifies to connect to the test network by default, but
you can change it to connect to the main network if you know what you're doing, and are
confident you will not lose actual funds.


## Start the server

```shell
mobcli start
```


## List of commands

- start
- stop
- create
- rename
- import
- export
- delete
- list
- send
