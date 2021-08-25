#! /bin/bash

cd "$( dirname "$0" )"

echo
echo "Installing MobileCoin command line interface."
sudo python3 setup.py install

echo
echo "Configuring environment variables."
cp mc_env.sh "$HOME/.mc_env.sh"
echo 'if [ -f "$HOME/.mc_env.sh" ]; then source "$HOME/.mc_env.sh"; fi' >> "$HOME/.bashrc"
source "$HOME/.bashrc"
