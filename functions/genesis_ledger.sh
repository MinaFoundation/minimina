#!/usr/bin/env bash
set -e

# ================================================
# Functions related to Genesis Ledger



# Upsert the Genesis Ledger timestamp
upsert-genesis-ledger-timestamp() {
  if ${UPDATE_GENESIS_TIMESTAMP}; then
    if test -f "${CONFIG}"; then
      jq ".genesis.genesis_state_timestamp=\"$(date +"%Y-%m-%dT%H:%M:%S%z")\"" ${CONFIG}
    else
      echo "Error: ${CONFIG} not found."
      exit 1
    fi
  fi
}

generate-genesis-ledger() {
  CONFIG=${LEDGER_FOLDER}/genesis_ledger.json
  python3 scripts/generate-genesis-ledger.py \
    --num-whale-accounts ${WHALES} \
    --num-fish-accounts ${FISH} \
    --offline-whale-accounts-directory ${LEDGER_FOLDER}/offline_whale_keys \
    --offline-fish-accounts-directory ${LEDGER_FOLDER}/offline_fish_keys \
    --online-whale-accounts-directory ${LEDGER_FOLDER}/online_whale_keys \
    --online-fish-accounts-directory ${LEDGER_FOLDER}/online_fish_keys \
    --snark-coordinator-accounts-directory ${LEDGER_FOLDER}/snark_coordinator_keys

  mv -f scripts/genesis_ledger.json ${CONFIG}
  rm -f scripts/*_ledger.json

  printf "\n"
  echo "================================"
  printf "\n"

  upsert-genesis-ledger-timestamp

}

reset-genesis-ledger-timestamp() {
  CONFIG=${LEDGER_FOLDER}/daemon.json
  if ${RESET}; then
    reset-genesis-ledger ${LEDGER_FOLDER} ${CONFIG}
  fi

  if ${UPDATE_GENESIS_TIMESTAMP}; then
    if test -f "${CONFIG}"; then
      echo 'Updating Genesis State timestamp...'
      printf "\n"

      tmp=$(mktemp)
      jq ".genesis.genesis_state_timestamp=\"$(date +"%Y-%m-%dT%H:%M:%S%z")\"" ${CONFIG} >"$tmp" && mv -f "$tmp" ${CONFIG}
    else
      reset-genesis-ledger ${LEDGER_FOLDER} ${CONFIG}
    fi
  fi
}
