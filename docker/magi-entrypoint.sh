#!/bin/sh

set -e

NETWORK=optimism-goerli # TODO: Add optimism-devnet support 
JWT_SECRET="688f5d737bad920bdfb2fc2f488d6b6209eebda1dae949a8de91398d932c517a"

exec magi \
    --network $NETWORK \
    --jwt-secret $JWT_SECRET \
    --l1-rpc-url http://l1:8545 \
    --l2-rpc-url http://l2:8545 \
    --l2-engine-url http://l2:8551 \
    --rpc-port 8545 \
    --sync-mode full \
    "$@"
