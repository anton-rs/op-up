#!/usr/bin/env bash

TIMESTAMP=$(date +%s | xargs printf '0x%x')
echo "Setting devnet timestamp to the current date"

cat $OP_UP_DIR/ops/devnet_state/genesis-l1.json | jq ".timestamp = $TIMESTAMP" > $OP_UP_DIR/ops/devnet_state/genesis-l1.json
cat $OP_UP_DIR/ops/devnet_state/genesis-l2.json | jq ".timestamp = $TIMESTAMP" > $OP_UP_DIR/ops/devnet_state/genesis-l2.json

echo "Done setting devnet timestamp"
