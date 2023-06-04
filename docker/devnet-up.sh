#!/usr/bin/env bash

# This script is a modified version of the one found in the Optimism monorepo:
# https://github.com/ethereum-optimism/optimism/tree/develop/ops-bedrock/devnet-up.sh

set -eu

NETWORK=devnetL1
DEVNET="../.devnet"
CONTRACTS_BEDROCK="../optimism/packages/contracts-bedrock"
OP_NODE="../optimism/op-node"

L2OO_ADDRESS="0x6900000000000000000000000000000000000000"
L1_URL="http://localhost:8545"
L2_URL="http://localhost:9545"
ROLLUP_CLIENT_URL="http://localhost:7545"

mkdir -p "$DEVNET"

# # Make sure the contracts have been built
# if [ ! -f "$DEVNET/contracts" ]; then
#   echo "Building contracts..."

#   (
#     cd "$CONTRACTS_BEDROCK"
#     yarn
#     yarn build
#     touch "../../$DEVNET/contracts"
#   )
  
#   echo "Done building contracts"
# fi

# Regenerate the L1 genesis file if necessary. The existence of the genesis
# file is used to determine if we need to recreate the devnet's state folder.
if [ ! -f "$DEVNET/done" ]; then
  echo "Regenerating genesis files"

  TIMESTAMP=$(date +%s | xargs printf '0x%x')
  cat "$CONTRACTS_BEDROCK/deploy-config/devnetL1.json" | jq -r ".l1GenesisBlockTimestamp = \"$TIMESTAMP\"" > /tmp/bedrock-devnet-deploy-config-int.json
  cat /tmp/bedrock-devnet-deploy-config-int.json | jq -r ".l1StartingBlockTag = \"earliest\"" > /tmp/bedrock-devnet-deploy-config.json

  (
    cd "$OP_NODE"
    go run cmd/main.go genesis devnet \
        --deploy-config /tmp/bedrock-devnet-deploy-config.json \
        --outfile.l1 "../$DEVNET/genesis-l1.json" \
        --outfile.l2 "../$DEVNET/genesis-l2.json" \
        --outfile.rollup "../$DEVNET/rollup.json"
    touch "../$DEVNET/done"
  )

  echo "Done regenerating genesis files"
fi

# Helper method that waits for a given URL to be up. Can't use
# cURL's built-in retry logic because connection reset errors
# are ignored unless you're using a very recent version of cURL
function wait_up {
  echo -n "Waiting for $1 to come up..."
  i=0
  until curl -s -f -o /dev/null "$1"
  do
    echo -n .
    sleep 0.25

    ((i=i+1))
    if [ "$i" -eq 300 ]; then
      echo " Timeout!" >&2
      exit 1
    fi
  done
  echo "Done!"
}

# Bring up L1.
(
  echo "Bringing up L1..."
  L1_CLIENT_CHOICE="$L1_CLIENT_CHOICE" L2OO_ADDRESS="$L2OO_ADDRESS" \
    docker-compose up -d --no-deps --build l1
  wait_up $L1_URL
)

# Bring up L2.
(
  echo "Bringing up L2..."
  L2_CLIENT_CHOICE="$L2_CLIENT_CHOICE" L2OO_ADDRESS="$L2OO_ADDRESS" \
    docker-compose up -d --no-deps --build l2
  wait_up $L2_URL
)

# Bring up the rollup client
(
  echo "Bringing up rollup client..."
  ROLLUP_CLIENT_CHOICE="$ROLLUP_CLIENT_CHOICE" L2OO_ADDRESS="$L2OO_ADDRESS" \
    docker-compose up -d --no-deps --build rollup-client
  wait_up $ROLLUP_CLIENT_URL
)

# Bring up the L2 proposer
(
  echo "Bringing up L2 proposer..."
  L2OO_ADDRESS="$L2OO_ADDRESS" \
    docker-compose up -d --no-deps --build proposer
)

# Bring up the L2 batcher
(
  echo "Bringing up L2 batcher..."
  L2OO_ADDRESS="$L2OO_ADDRESS" \
    docker-compose up -d --no-deps --build batcher
)

# Bring up the challenger agent
# TODO
# (
#   echo "Bringing up challenger agent..."
#   docker-compose up -d --no-deps --build challenger-agent
# )

# Bring up stateviz
(
  echo "Bringing up stateviz webserver..."
  docker-compose up -d --no-deps --build stateviz
)

echo "Devnet ready to go!"
