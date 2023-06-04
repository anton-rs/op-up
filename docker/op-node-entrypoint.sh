#!/bin/sh

set -exu

exec op-node \
      --l1=ws://l1:8546 \
      --l2=http://l2:8551 \
      --l2.jwt-secret=/config/test-jwt-secret.txt \
      --sequencer.enabled \
      --sequencer.l1-confs=0 \
      --verifier.l1-confs=0 \
      --p2p.sequencer.key=8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba \
      --rollup.config=/rollup.json \
      --rpc.addr=0.0.0.0 \
      --rpc.port=8545 \
      --p2p.listen.ip=0.0.0.0 \
      --p2p.listen.tcp=9003 \
      --p2p.listen.udp=9003 \
      --p2p.scoring.peers=light \
      --p2p.ban.peers=true \
      --snapshotlog.file=/op_log/snapshot.log \
      --p2p.priv.path=/config/p2p-node-key.txt \
      --metrics.enabled \
      --metrics.addr=0.0.0.0 \
      --metrics.port=7300 \
      --pprof.enabled \
      --rpc.enable-admin \
      "$@"
