#!/usr/bin/env bash
set -e

# ================================================
# Functions related to Mina Daemon

generate-keypair() {
  ${MINA_EXE} advanced generate-keypair -privkey-path ${1}
}

generate-libp2p-keypair() {
  ${MINA_EXE} libp2p generate-keypair -privkey-path ${1}
}

# Executes the Mina Daemon, exposing all 5 ports in
# sequence starting with provided base port
exec-daemon() {
  BASE_PORT=${1}
  shift
  CLIENT_PORT=${BASE_PORT}
  REST_PORT=$((${BASE_PORT} + 1))
  EXTERNAL_PORT=$((${BASE_PORT} + 2))
  DAEMON_METRICS_PORT=$((${BASE_PORT} + 3))
  LIBP2P_METRICS_PORT=$((${BASE_PORT} + 4))

  exec ${MINA_EXE} daemon \
    -client-port ${CLIENT_PORT} \
    -rest-port ${REST_PORT} \
    -insecure-rest-server \
    -external-port ${EXTERNAL_PORT} \
    -metrics-port ${DAEMON_METRICS_PORT} \
    -libp2p-metrics-port ${LIBP2P_METRICS_PORT} \
    -config-file ${CONFIG} \
    -log-json \
    -log-level ${LOG_LEVEL} \
    -file-log-level ${FILE_LOG_LEVEL} \
    $@
}

# Executes the Mina Snark Worker
exec-worker-daemon() {
  COORDINATOR_PORT=${1}
  shift
  SHUTDOWN_ON_DISCONNECT="false"
  COORDINATOR_HOST_AND_PORT="localhost:${COORDINATOR_PORT}"

  exec ${MINA_EXE} internal snark-worker \
    -proof-level ${PROOF_LEVEL} \
    -shutdown-on-disconnect ${SHUTDOWN_ON_DISCONNECT} \
    -daemon-address ${COORDINATOR_HOST_AND_PORT} \
    $@
}

# Spawns the Node in background
spawn-node() {
  FOLDER=${1}
  shift
  exec-daemon $@ -config-directory ${FOLDER} &>${FOLDER}/log.txt &
}

# Spawns worker in background
spawn-worker() {
  FOLDER=${1}
  shift
  exec-worker-daemon $@ -config-directory ${FOLDER} &>${FOLDER}/log.txt &
}
