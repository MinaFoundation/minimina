#!/usr/bin/env bash
set -e

# ================================================
# Functions related to KeyPairs

# Clean directories
clean-dir() {
  rm -rf ${1}
  mkdir -p ${1}
}

# Generate keypair for the daemon
generate-keypair() {
  echo "Generating keypair in ${1}"
  ${MINA_EXE} advanced generate-keypair -privkey-path ${1}
  printf "\n"
}

# Generate libp2p keypair
generate-libp2p-keypair() {
  echo "Generating libp2p keypair in ${1}"
  ${MINA_EXE} libp2p generate-keypair -privkey-path ${1}
  printf "\n"
}

clean-network-keypairs() {

    echo "Reset Genesis Ledger..."
    printf "\n"

    LEDGER_FOLDER="${HOME}/.mina-network/mina-local-network-${WHALES}-${FISH}-${NODES}"

    clean-dir ${LEDGER_FOLDER}/offline_whale_keys
    clean-dir ${LEDGER_FOLDER}/offline_fish_keys
    clean-dir ${LEDGER_FOLDER}/online_whale_keys
    clean-dir ${LEDGER_FOLDER}/online_fish_keys
    clean-dir ${LEDGER_FOLDER}/snark_coordinator_keys
    clean-dir ${LEDGER_FOLDER}/service-keys
    clean-dir ${LEDGER_FOLDER}/libp2p_keys
    clean-dir ${LEDGER_FOLDER}/zkapp_keys

}

# Generate the Genesis Ledger
generate-network-keypairs() {

  clean-network-keypairs

  echo "Generating Genesis Ledger..."
  printf "\n"

  if ${ZKAPP_TRANSACTIONS}; then
    generate-keypair ${LEDGER_FOLDER}/zkapp_keys/zkapp_account
  fi

  generate-keypair ${LEDGER_FOLDER}/snark_coordinator_keys/snark_coordinator_account
  for ((i = 0; i < ${FISH}; i++)); do
    generate-keypair ${LEDGER_FOLDER}/offline_fish_keys/offline_fish_account_${i}
    generate-keypair ${LEDGER_FOLDER}/online_fish_keys/online_fish_account_${i}
    generate-libp2p-keypair ${LEDGER_FOLDER}/libp2p_keys/fish_${i}
  done
  for ((i = 0; i < ${WHALES}; i++)); do
    generate-keypair ${LEDGER_FOLDER}/offline_whale_keys/offline_whale_account_${i}
    generate-keypair ${LEDGER_FOLDER}/online_whale_keys/online_whale_account_${i}
    generate-libp2p-keypair ${LEDGER_FOLDER}/libp2p_keys/whale_${i}
  done
  for ((i = 0; i < ${NODES}; i++)); do
    generate-keypair ${LEDGER_FOLDER}/offline_whale_keys/offline_whale_account_${i}
    generate-keypair ${LEDGER_FOLDER}/online_whale_keys/online_whale_account_${i}
    generate-libp2p-keypair ${LEDGER_FOLDER}/libp2p_keys/node_${i}
  done

  if ${ZKAPP_TRANSACTIONS}; then
    chmod -R 0700 ${LEDGER_FOLDER}/zkapp_keys
  fi
  chmod -R 0700 ${LEDGER_FOLDER}/offline_fish_keys
  chmod -R 0700 ${LEDGER_FOLDER}/online_fish_keys
  chmod -R 0700 ${LEDGER_FOLDER}/offline_whale_keys
  chmod -R 0700 ${LEDGER_FOLDER}/online_whale_keys
  chmod -R 0700 ${LEDGER_FOLDER}/snark_coordinator_keys
  chmod -R 0700 ${LEDGER_FOLDER}/service-keys
  chmod -R 0700 ${LEDGER_FOLDER}/libp2p_keys
}
