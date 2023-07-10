#!/usr/bin/env bash
set -e

# ================================================
# Constants

MINA_EXE=./mina.exe
ARCHIVE_EXE=./archive.exe
LOGPROC_EXE=./logproc.exe
ZKAPP_EXE=./zkapp_test_transaction.exe

export MINA_PRIVKEY_PASS='naughty blue worm'
export MINA_LIBP2P_PASS="${MINA_PRIVKEY_PASS}"
