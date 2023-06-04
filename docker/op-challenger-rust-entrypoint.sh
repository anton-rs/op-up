#!/bin/sh

set -e

OP_CHALLENGER_SIGNER_KEY=a1742ee5f7898541224d6a91d9f3b34ad442e27bcb43223c01e47e58fc0a0c12

exec op-challenger \
    --l1-ws-endpoint ws://l1:8546 \
    --trusted-op-node-endpoint http://rollup-client:8545 \
    --signer-key $OP_CHALLENGER_SIGNER_KEY \
    --dispute-game-factory $DGF_ADDRESS \
    --l2-output-oracle $L2OO_ADDRESS \
    --mode listen-and-respond \
    -vv
