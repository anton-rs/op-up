#!/bin/sh

set -e

exec op-challenger \
    --l1-eth-rpc http://l1:8545 \
    --rollup-rpc http://rollup-client:8545 \
    --l2oo-address $L2OO_ADDRESS \
    --dgf-address $DGF_ADDRESS \
    "$@"
