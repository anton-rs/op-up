# op-up

[![CI](https://github.com/anton-rs/op-up/actions/workflows/ci.yml/badge.svg)][gh-ci]
[![License](https://img.shields.io/badge/License-MIT-orange.svg)][mit-license]
[![Chat][tg-badge]][tg-url]

[mit-license]: https://github.com/anton-rs/op-up/blob/main/LICENSE.md
[gh-ci]: https://github.com/anton-rs/op-up/actions/workflows/ci.yml
[tg-url]: https://t.me/+XR8_p3qjzoFiMjEx
[tg-badge]: https://img.shields.io/badge/chat-telegram-blue

**Composable OP Stack Orchestration System**

![](./etc/op-up-banner.png)

**[Install](./docs/install/installation.md)**
| [User Book](https://opup.anton.systems)
| [Developer Docs](./docs/developers/developers.md)
| [Crate Docs](https://crates.io/crates/opup)

_The project is still work in progress, see the [disclaimer below](#status-httpsgithubcomanton-rsop-uplabelsalpha)._

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

For example, want to run a pure rust op-stack (almost)?

You can use

- [reth](https://github.com/paradigmxyz/reth) as an L1 execution node.
- [op-reth](https://github.com/anton-rs/op-reth/) as an L2 node (interchangeable with [op-geth](https://github.com/ethereum-optimism/op-geth)).
- [magi](https://github.com/a16z/magi) as the rollup node.

_Notice, this does not include the proposer or batcher,
as well as fault proof components._

## Usage

First, make sure you have a few things installed.

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://www.docker.com/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Make](https://www.gnu.org/software/make/)
- [jq](https://jqlang.github.io/jq/)

To get started with the interactive prompt, run the following commands:

```sh
git clone git@github.com:anton-rs/op-up.git && cd op-up
make devnet
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
make devnet-stop
```

To reset the devnet state, run:

```sh
make nuke
```

## Using `op-up` as a library

By building with Rust's [crate system](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html),
`op-up` can easily be used as a library in an extensible way.

## Status https://github.com/anton-rs/op-up/labels/alpha

`op-up` is **not ready for production use**.

Local and devnet experimentation is highly encouraged.
New issues are also welcome.

We appreciate your patience until we release the first version of `op-up`.

In the meantime, contribute, chat with us [on telegram][tg-url], and most
importantly, have fun!

## Troubleshooting

Please check if your issue is covered in the [troubleshooting docs](./docs/developers/troubleshooting.md).

If not, [open an issue](https://github.com/anton-rs/op-up/issues/new) with all possible relevant details.

## Contributions & Bug Reports

Please report any bugs or issues you encounter by [opening a github issue](https://github.com/anton-rs/op-up/issues/new).

All contributions are welcome, but if you are at all unsure, visit the [developer docs](./docs/developers/contributing.md).

## License

This project is licensed under the [MIT License](LICENSE.md). Free and open-source, forever.

_All our rust are belong to you._
