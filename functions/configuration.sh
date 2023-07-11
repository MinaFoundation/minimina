# ================================================
# Parse inputs from arguments

parse_arguments() {
    for ARG in "$@"; do
    if [[ "${ARG}" == "-h" || "${ARG}" == "--help" ]]; then
        show_help
    fi
    done

    while [[ "$#" -gt 0 ]]; do
    case ${1} in
    -w | --whales)
        WHALES="${2}"
        shift
        ;;
    -f | --fish)
        FISH="${2}"
        shift
        ;;
    -n | --nodes)
        NODES="${2}"
        shift
        ;;
    -a | --archive) ARCHIVE=true ;;
    -sp | --seed-start-port)
        SEED_START_PORT="${2}"
        shift
        ;;
    -scp | --snark-coordinator-start-port)
        SNARK_COORDINATOR_PORT="${2}"
        shift
        ;;
    -swc | --snark-workers-count)
        SNARK_WORKERS_COUNT="${2}"
        shift
        ;;
    -wp | --whale-start-port)
        WHALE_START_PORT="${2}"
        shift
        ;;
    -fp | --fish-start-port)
        FISH_START_PORT="${2}"
        shift
        ;;
    -np | --node-start-port)
        NODE_START_PORT="${2}"
        shift
        ;;
    -ap | --archive-server-port)
        ARCHIVE_SERVER_PORT="${2}"
        shift
        ;;
    -ll | --log-level)
        LOG_LEVEL="${2}"
        shift
        ;;
    -fll | --file-log-level)
        FILE_LOG_LEVEL="${2}"
        shift
        ;;
    -ph | --pg-host)
        PG_HOST="${2}"
        shift
        ;;
    -pp | --pg-port)
        PG_PORT="${2}"
        shift
        ;;
    -pu | --pg-user)
        PG_USER="${2}"
        shift
        ;;
    -ppw | --pg-passwd)
        PG_PASSWD="${2}"
        shift
        ;;
    -pd | --pg-db)
        PG_DB="${2}"
        shift
        ;;
    -vt | --value-transfer-txns) VALUE_TRANSFERS=true ;;
    -zt | --zkapp-transactions) ZKAPP_TRANSACTIONS=true ;;
    -tf | --transactions-frequency)
        TRANSACTION_FREQUENCY="${2}"
        shift
        ;;
    -sf | --snark-worker-fee)
        SNARK_WORKER_FEE="${2}"
        shift
        ;;
    -pl | --proof-level)
        PROOF_LEVEL="${2}"
        shift
        ;;
    -r | --reset) RESET=true ;;
    -u | --update-genesis-timestamp) UPDATE_GENESIS_TIMESTAMP=true ;;
    *)
        echo "Unknown parameter passed: ${1}"
        exit 1
        ;;
    esac
    shift
    done
}

validate_config() {

  if ${VALUE_TRANSFERS}; then
    if [ "${FISH}" -eq "0" ]; then
      echo "Sending transactions requires at least one 'Fish' node running!"
      printf "\n"
      exit 1
    fi
  fi

  if ${ZKAPP_TRANSACTIONS}; then
    if [ "${WHALES}" -lt "2" ] || [ "${FISH}" -eq "0" ]; then
      echo "Send zkApp transactions requires at least one 'Fish' node running and at least 2 Whale accounts acting as the fee payer and sender account!"
      printf "\n"
      exit 1
    fi
  fi

  echo "Network starting with the following topology:"
  echo -e "- 1 seed node"
  echo -e "- 1 snark coordinator"
  echo -e "- ${SNARK_WORKERS_COUNT} snark worker(s)"

  if ${ARCHIVE}; then
    echo -e "- 1 archive"
  fi

  echo -e "- ${WHALES} whales (block producers with big stake)"
  echo -e "- ${FISH} fish (block producers with small stake)"
  echo -e "- ${NODES} non block-producing nodes"
  echo -e "- Sending transactions: ${VALUE_TRANSFERS}"
  echo -e "- Sending zkApp transactions: ${ZKAPP_TRANSACTIONS}"
  printf "\n"
  echo "================================"
  printf "\n"
}
