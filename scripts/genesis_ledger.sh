#!/usr/bin/env bash
set -e

# ================================================
# Functions related to Genesis Ledger

# Resets genesis ledger
reset-genesis-ledger() {
  GENESIS_LEDGER_FOLDER=${1}
  DAEMON_CONFIG=${2}
  echo 'Resetting Genesis Ledger...'
  printf "\n"

  jq "{genesis: {genesis_state_timestamp:\"$(date +"%Y-%m-%dT%H:%M:%S%z")\"}, ledger:.}" \
    <${GENESIS_LEDGER_FOLDER}/genesis_ledger.json \
    >${DAEMON_CONFIG}
}
