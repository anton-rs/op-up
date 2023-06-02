# op-up

> **Warning**
>
> This is a work in progress.

**OP-Up is a hive tool for testing OP-Stack-compatible software modules.**

This project was born out of the need to test out [Magi](https://github.com/a16z/magi), a rollup client built for the OP stack. Having an easy-to-use environment to spin up a local devnet is crucial for quick testing and experimentation, especially when there exist different implementations of each component in the stack.

For instance, you can use OP-Up to spin up a devnet with a [Geth](https://github.com/ethereum/go-ethereum) L1 node, an [OP-Erigon](https://github.com/testinprod-io/op-erigon) L2 node, and a [Magi](https://github.com/a16z/magi) rollup node, and test out the interoperability between them in an end-to-end fashion.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://www.docker.com/) & [Docker Compose](https://docs.docker.com/compose/)
- [jq](https://jqlang.github.io/jq/) (for parsing JSON)

## Usage

Clone this repository and run the following command. <br />
If all goes well, you should be able to continue with the setup in the terminal prompt.

```
cd cli && cargo run
```

## License

This project is licensed under the [MIT License](LICENSE). <br />
Free and open-source, forever.
