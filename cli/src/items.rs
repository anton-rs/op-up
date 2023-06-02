macro_rules! make_item {
    ($name:ident, $value: expr) => {
        pub const $name: &str = $value;
    };
}

// L1 clients
make_item!(GETH, "geth (go)");
make_item!(ERIGON, "erigon (go)");

// L2 clients
make_item!(OP_GETH, "op-geth (go)");
make_item!(OP_ERIGON, "op-erigon (go)");

// Rollup clients
make_item!(OP_NODE, "op-node (go)");
make_item!(MAGI, "magi (rust)");

// Challenger agents
make_item!(OP_CHALLENGER_GO, "op-challenger (go)");
make_item!(OP_CHALLENGER_RUST, "op-challenger (rust)");
