# op-up

[![Build Status]][actions]
[![License]][mit-license]
[![Docs]][Docs-rs]
[![Latest Version]][crates.io]
[![rustc 1.31+]][Rust 1.31]

[Build Status]: https://img.shields.io/github/actions/workflow/status/anton-rs/op-up/ci.yml?branch=main
[actions]: https://github.com/anton-rs/op-up/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/opup.svg
[crates.io]: https://crates.io/crates/opup
[rustc 1.31+]: https://img.shields.io/badge/rustc_1.31+-lightgray.svg
[Rust 1.31]: https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html
[License]: https://img.shields.io/badge/license-MIT-7795AF.svg
[mit-license]: https://github.com/anton-rs/op-up/blob/main/LICENSE.md
[Docs-rs]: https://docs.rs/opup/
[Docs]: https://img.shields.io/docsrs/morc.svg?color=319e8c&label=docs.rs

**Composable OP Stack Orchestration System.** op-up is in https://github.com/anton-rs/op-up/labels/alpha.

![](https://raw.githubusercontent.com/anton-rs/op-up/main/etc/op-up-banner.png)

**[Install](#usage)**
| [User Docs](#what-is-op-up)
| [Crate Docs][crates.io]
| [Reference][Docs-rs]
| [Contributing](#contributing)
| [License](#license)

## What is op-up?

`op-up` is the infrastructure for building composable OP Stack configurations.
Given the growing number of OP Stack component implementations, having a simple
service to spin up a composable devnet in a programmatical way is crucial for
verifying superchain compatibility, general testing and experimentation.

The project was born out of the need to test out [Magi](https://github.com/a16z/magi),
a rollup client built for the OP stack.

Have a new rollup derivation pipeline implementation for the OP Stack and want to test it?

> Use `op-up` to configure an OP Stack with the new rollup derivation pipeline.
> Then, you can spin up a local devnet and run test suites against it!

## What's the OP Stack?

The [OP Stack](https://stack.optimism.io/) is what powers the superchain!

It is a stack of various software components that, together, can be used
to fully run a chain in the superchain. The [Optimism Collective](https://app.optimism.io/announcement) has
already spent an enormous amount of effort and time building out the
[specifications](https://github.com/ethereum-optimism/optimism/blob/develop/specs/README.md)
for how [OP Stack](https://stack.optimism.io/) components work together
in an interoperable way.

For example, want to run a pure rust op-stack?

You can use

- [reth](https://github.com/paradigmxyz/reth) as an L1 execution node.
- [op-reth](https://github.com/anton-rs/op-reth/) as an L2 node (interchangeable with [op-geth](https://github.com/ethereum-optimism/op-geth)).
- [magi](https://github.com/a16z/magi) as the rollup node.

_Notice, this does not include the proposer or batcher,
as well as fault proof components._

## Usage

### Pre-requisites

First, install the following dependencies on your machine:

- Rust: [Install guide](https://www.rust-lang.org/tools/install)
- Docker: [Install guide](https://docs.docker.com/get-docker/)

### Installation

To get started with the interactive prompt, run the following commands:

```sh
git clone git@github.com:anton-rs/op-up.git && cd op-up
cargo run
```

This will bring up a local devnet using the default components.

Once the devnet is up and running, L1 is accessible at `http://localhost:8545`, and L2 is accessible at `http://localhost:9545`.
Any Ethereum tool - Metamask, `seth`, etc. - can use these endpoints.
Note that you will need to specify the L2 chain ID manually if you use Metamask. The devnet's L2 chain ID is 901.

The devnet comes with a pre-funded account you can use as a faucet:

- Address: `0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266`
- Private key: `ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`

The faucet account exists on both L1 and L2.

---

To stop the devnet, run:

```sh
cargo run -- stop
```

To reset the devnet state, run:

```sh
cargo run -- nuke
```

## Using `op-up` as a library

By building with Rust's [crate system](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html),
`op-up` can easily be used as a library in an extensible way.

## Contributing

Please report any bugs or issues you encounter by [opening a github issue](https://github.com/anton-rs/op-up/issues/new).

All contributions are welcome, but if you are at all unsure, visit the [developer docs](./docs/developers/contributing.md).

## License

This project is licensed under the [MIT License](LICENSE.md). Free and open-source, forever.

_All our rust are belong to you._
