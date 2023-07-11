#!/usr/bin/env bash
set -e

for f in ./functions/*.sh; do source $f; done

# Print MiniMina ASCII logo
print_logo

# Parse arguments used to define the network topology
parse_arguments "$@"

# Validate the configuration
validate_config

# Generate the relevant keys pair according the network topology
# generate-network-keypairs

# Generate the Genesis Ledger
# generate-genesis-ledger

# Generate Nginx configuration
generate-nginx-config
