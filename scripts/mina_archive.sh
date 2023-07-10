#!/usr/bin/env bash
set -e

# ================================================
# Functions related to Mina Archive

# Executes the Archive node
exec-archive-node() {
  exec ${ARCHIVE_EXE} run \
    --config-file ${CONFIG} \
    --log-level ${LOG_LEVEL} \
    --postgres-uri postgresql://${PG_USER}:${PG_PASSWD}@${PG_HOST}:${PG_PORT}/${PG_DB} \
    --server-port ${ARCHIVE_SERVER_PORT} \
    $@
}

# Spawns the Archive Node in background
spawn-archive-node() {
  FOLDER=${1}
  shift
  exec-archive-node $@ &>${FOLDER}/log.txt &
}


create-schema() {

  [[ -s "$PG_SCHEMA_PATH" ]] || { echo "Error: File does not exist or is empty." >&2; exit 1; }

  echo "Creating database '${PG_DB}'..."

  psql postgresql://${PG_USER}:${PG_PASSWD}@${PG_HOST}:${PG_PORT} -c "DROP DATABASE IF EXISTS ${PG_DB};"
  psql postgresql://${PG_USER}:${PG_PASSWD}@${PG_HOST}:${PG_PORT} -c "CREATE DATABASE ${PG_DB};"
  psql postgresql://${PG_USER}:${PG_PASSWD}@${PG_HOST}:${PG_PORT} ${PG_DB} < $PG_SCHEMA_PATH

  echo "Schema '${PG_DB}' created successfully."
  printf "\n"
}
