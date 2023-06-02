#!/usr/bin/env bash

# WARNING: Do not call this script directly. Use the CLI interface instead.

# check if optimism directory does not exists
if [ ! -d "optimism" ]; then
    echo "getting optimism monorepo ref from github..."
    git clone --no-checkout git@github.com:ethereum-optimism/optimism.git
fi

# check if optimism-rs directory does not exists
if [ ! -d "optimism-rs" ]; then
    echo "getting optimism-rs ref from github..."
    git clone --no-checkout git@github.com:refcell/optimism-rs.git
fi
