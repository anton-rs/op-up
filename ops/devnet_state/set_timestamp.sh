#!/usr/bin/env bash

set -e

TIMESTAMP_UNIX=$(date +%s | xargs printf '%d')
TIMESTAMP=$(date +%s | xargs printf '0x%x')

echo "Setting devnet timestamp to the current date"

mkdir -p $OP_UP_DIR/.devnet/state

cat "$OP_UP_DIR/ops/devnet_state/genesis-l1.json" | jq -r ".timestamp = \"$TIMESTAMP\"" > $OP_UP_DIR/.devnet/state/genesis-l1.json
cat "$OP_UP_DIR/ops/devnet_state/genesis-l2.json" | jq -r ".timestamp = \"$TIMESTAMP\"" > $OP_UP_DIR/.devnet/state/genesis-l2.json
cat "$OP_UP_DIR/ops/devnet_state/rollup.json" | jq -r ".l2_time = \"$TIMESTAMP\"" > $OP_UP_DIR/.devnet/state/rollup.json

echo "Done setting devnet timestamp"
