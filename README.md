# MiniMina

[![Build](https://github.com/MinaFoundation/minimina/actions/workflows/build.yaml/badge.svg)](https://github.com/MinaFoundation/minimina/actions/workflows/build.yaml)

## Overview

MiniMina is a command line tool aimed at providing the capability to spin up Mina networks locally on a user's computer. For more more information see [Minimina RFC](https://www.notion.so/minafoundation/MiniMina-v2-19775eec3c604476894633f8fe84a2d0).

:warning: **It is still a work in progress and things may not work as expected.** :warning:

## Getting Started

To start using MiniMina, clone the GitHub repository:

```bash
git clone https://github.com/MinaFoundation/minimina.git
cd minimina
```

### Default network

Create a network with default settings called `default`

```bash
cargo run -- network create
```

Investigate the network directory `~/.minimina/default/`

```bash
$ tree -p ~/.minimina/default/

~/.minimina/default/
├── [-rw-r--r--]  docker-compose.yaml
├── [-rw-r--r--]  genesis_ledger.json
├── [drwx------]  libp2p-keypairs
│   ├── [-rw-------]  mina-bp-1
│   ├── [-rw-r--r--]  mina-bp-1.peerid
│   ├── [-rw-------]  mina-bp-2
│   ├── [-rw-r--r--]  mina-bp-2.peerid
│   ├── [-rw-------]  mina-seed-1
│   ├── [-rw-r--r--]  mina-seed-1.peerid
│   ├── [-rw-------]  mina-snark-coordinator
│   ├── [-rw-r--r--]  mina-snark-coordinator.peerid
│   ├── [-rw-------]  mina-snark-worker-1
│   └── [-rw-r--r--]  mina-snark-worker-1.peerid
├── [-rw-r--r--]  network.json
└── [drwx------]  network-keypairs
    ├── [-rw-------]  mina-bp-1
    ├── [-rw-r--r--]  mina-bp-1.pub
    ├── [-rw-------]  mina-bp-2
    ├── [-rw-r--r--]  mina-bp-2.pub
    ├── [-rw-------]  mina-seed-1
    ├── [-rw-r--r--]  mina-seed-1.pub
    ├── [-rw-------]  mina-snark-coordinator
    ├── [-rw-r--r--]  mina-snark-coordinator.pub
    ├── [-rw-------]  mina-snark-worker-1
    └── [-rw-r--r--]  mina-snark-worker-1.pub
```

The default network can be started, stopped, and deleted

```bash
cargo run -- network start
cargo run -- network stop
cargo run -- network delete
```

Default network info and status can be queried

```bash
cargo run -- network info
cargo run -- network status
```

For debug log info, run any of these commands with the prefix `RUST_LOG=debug`

### Networks based on Lucy-generated genesis and topology

We also provide two example networks in `./tests/data/`, `small_network` and `large_network`, with all required keys and files. To create and deploy these networks

```bash
# create large-network's docker compose file
cargo run -- network create \
  -g ./tests/data/large_network/genesis_ledger.json \
  -t ./tests/data/large_network/topology.json \
  -n large-network

# start large-network
cargo run -- network start -n large-network

# stop large-network
cargo run -- network stop -n large-network

# delete large-network
cargo run -- network delete -n large-network
```

```bash
# create small-network's docker compose file
cargo run -- network create \
  -g ./tests/data/small_network/genesis_ledger.json \
  -t ./tests/data/small_network/topology.json \
  -n small-network

# start small-network
cargo run -- network start -n small-network

# stop small-network
cargo run -- network stop -n small-network

# delete small-network
cargo run -- network delete -n small-network
```

To monitor the action, you will want to use a resource monitor, like [`bpytop`](https://github.com/aristocratos/bpytop).

### Prerequisites

MiniMina is written in Rust and requires the Rust compiler and its package manager, `cargo`. If you don't have Rust installed, you can install it from the [official website](https://www.rust-lang.org/tools/install).

MiniMina uses `docker` and requires it to be installed on user's machine. See [docker install](https://docs.docker.com/engine/install/).

### Building

To build MiniMina, use the `cargo build --release` command in the root directory of the repository:

```bash
cargo build --release
```

This will produce an executable file in the `target/release/` directory.

### Testing

You can run unit tests using the `cargo test` command:

```bash
cargo test
```

This will run all tests in the project and output the results.

## Usage

As MiniMina is a work in progress, for information on usage, please refer to the built-in help:

```bash
minimina --help
```

## Contributing

As MiniMina is a work in progress, contributions are always welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the [MIT License](LICENSE).
