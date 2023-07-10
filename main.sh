#!/usr/bin/env bash
set -e

for f in ./scripts/*.sh; do source $f; done

parse_args "$@"

echo_logo
