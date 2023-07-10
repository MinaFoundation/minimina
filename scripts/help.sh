#!/usr/bin/env bash
set -e

# ================================================
# Helper functions

show_help() {
  echo "-w   |--whales <#>                       | Number of BP Whale Nodes (bigger stake) to spin-up"
  echo "                                         |   Default: ${WHALES}"
  echo "-f   |--fish <#>                         | Number of BP Fish Nodes (less stake) to spin-up"
  echo "                                         |   Default: ${FISH}"
  echo "-n   |--nodes <#>                        | Number of non block-producing nodes to spin-up"
  echo "                                         |   Default: ${NODES}"
  echo "-a   |--archive                          | Whether to run the Archive Node (presence of argument)"
  echo "                                         |   Default: ${ARCHIVE}"
  echo "-sp  |--seed-start-port <#>              | Seed Node range start port"
  echo "                                         |   Default: ${SEED_START_PORT}"
  echo "-swp |--snark-coordinator-start-port <#> | Snark Worker Coordinator Node range start port"
  echo "                                         |   Default: ${SNARK_COORDINATOR_PORT}"
  echo "-swc |--snark-workers-count <#>          | Snark Workers count"
  echo "                                         |   Default: ${SNARK_WORKERS_COUNT}"
  echo "-wp  |--whale-start-port <#>             | Whale Nodes range start port"
  echo "                                         |   Default: ${WHALE_START_PORT}"
  echo "-fp  |--fish-start-port <#>              | Fish Nodes range start port"
  echo "                                         |   Default: ${FISH_START_PORT}"
  echo "-np  |--node-start-port <#>              | Non block-producing Nodes range start port"
  echo "                                         |   Default: ${NODE_START_PORT}"
  echo "-ap  |--archive-server-port <#>          | Archive Node server port"
  echo "                                         |   Default: ${ARCHIVE_SERVER_PORT}"
  echo "-ll  |--log-level <level>                | Console output logging level"
  echo "                                         |   Default: ${LOG_LEVEL}"
  echo "-fll |--file-log-level <level>           | File output logging level"
  echo "                                         |   Default: ${FILE_LOG_LEVEL}"
  echo "-ph  |--pg-host <host>                   | PostgreSQL host"
  echo "                                         |   Default: ${PG_HOST}"
  echo "-pp  |--pg-port <#>                      | PostgreSQL port"
  echo "                                         |   Default: ${PG_PORT}"
  echo "-pu  |--pg-user <user>                   | PostgreSQL user"
  echo "                                         |   Default: ${PG_USER}"
  echo "-ppw |--pg-passwd <password>             | PostgreSQL password"
  echo "                                         |   Default: <empty_string>"
  echo "-pd  |--pg-db <db>                       | PostgreSQL database name"
  echo "                                         |   Default: ${PG_DB}"
  echo "-vt  |--value-transfer-txns              | Whether to execute periodic value transfer transactions (presence of argument)"
  echo "                                         |   Default: ${VALUE_TRANSFERS}"
  echo "-zt  |--zkapp-transactions               | Whether to execute periodic zkapp transactions (presence of argument)"
  echo "                                         |   Default: ${ZKAPP_TRANSACTIONS}"
  echo "-tf  |--transactions-frequency <#>       | Frequency of periodic transactions execution (in seconds)"
  echo "                                         |   Default: ${TRANSACTION_FREQUENCY}"
  echo "-sf  |--snark-worker-fee <#>             | SNARK Worker fee"
  echo "                                         |   Default: ${SNARK_WORKER_FEE}"
  echo "-pl  |--proof-level <proof-level>        | Proof level (currently consumed by SNARK Workers only)"
  echo "                                         |   Default: ${PROOF_LEVEL}"
  echo "-r   |--reset                            | Whether to reset the Mina Local Network storage file-system (presence of argument)"
  echo "                                         |   Default: ${RESET}"
  echo "-u   |--update-genesis-timestamp         | Whether to update the Genesis Ledger timestamp (presence of argument)"
  echo "                                         |   Default: ${UPDATE_GENESIS_TIMESTAMP}"
  echo "-h   |--help                             | Displays this help message"

  printf "\n"
  echo "Available logging levels:"
  echo "  Spam, Trace, Debug, Info, Warn, Error, Faulty_peer, Fatal"
  printf "\n"
  echo "Available proof levels:"
  echo "  full, check, none"
  printf "\n"

  exit
}
