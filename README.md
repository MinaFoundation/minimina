# MiniMina

[![Build](https://github.com/MinaFoundation/minimina/actions/workflows/build.yaml/badge.svg)](https://github.com/MinaFoundation/minimina/actions/workflows/build.yaml)

## Overview

MiniMina is a command line tool aimed at providing the capability to spin up Mina networks locally on a user's computer. For more more information see [Minimina RFC](https://www.notion.so/minafoundation/MiniMina-v2-19775eec3c604476894633f8fe84a2d0).

### Prerequisites

MiniMina requires `docker` to be present on user's machine. See [docker install](https://docs.docker.com/engine/install/).

For building and testing MiniMina requires Rust compiler and its package manager `cargo`. See [Rust official website](https://www.rust-lang.org/tools/install) for installation details.

## Getting Started

To set up and use MiniMina, you have a couple of options: building from source or using the provided deb package.

### Building from Source

```bash
git clone https://github.com/MinaFoundation/minimina.git
cd minimina
cargo build --release
cp target/release/minimina ~/.local/bin
```
Assuming `~/.local/bin` is on your `$PATH` you will have `minimina` available for execution directly from the command line.

### Installing from the Deb Package

If you'd prefer a simpler installation or are not interested in building from source, MiniMina is available as deb package.

```bash
echo "deb [trusted=yes] http://packages.o1test.net ubuntu stable" | sudo tee /etc/apt/sources.list.d/mina.list
sudo apt-get update
sudo apt-get install -y minimina
```
**Note 1:**  The `stable` repository contains the release version of MiniMina, while `unstable` mirrors the current state of the `main` branch in the repository. Choose accordingly based on your needs.

**Note 2:** MiniMina deb package is known to work on Ubuntu 18.04, Ubuntu 20.04, Debian 10, Debian 11. Installing it on other systems may necessitate adjusting or installing additional dependent libraries or packages.

## Usage

MiniMina provides functionalities to manage both the entire network and individual nodes within it. To explore the available commands, use:

```bash
minimina --help
minimina network --help
minimina node --help
```
Below are a few fundamental examples to get you started:

### Default network

Create a network with default settings called `default`

```bash
minimina network create
```

Investigate the network directory `~/.minimina/default/`

```bash
$ tree -p ~/.minimina/default/

~/.minimina/default/
├── [-rw-rw-r--]  create_schema.sql
├── [-rw-rw-r--]  docker-compose.yaml
├── [-rw-rw-r--]  genesis_ledger.json
├── [drwx------]  libp2p-keypairs
│   ├── [-rw-------]  mina-archive
│   ├── [-rw-r--r--]  mina-archive.peerid
│   ├── [-rw-------]  mina-bp-1
│   ├── [-rw-r--r--]  mina-bp-1.peerid
│   ├── [-rw-------]  mina-bp-2
│   ├── [-rw-r--r--]  mina-bp-2.peerid
│   ├── [-rw-------]  mina-seed-1
│   ├── [-rw-r--r--]  mina-seed-1.peerid
│   ├── [-rw-------]  mina-snark-coordinator
│   ├── [-rw-r--r--]  mina-snark-coordinator.peerid
│   ├── [-rw-------]  mina-snark-worker-1
│   └── [-rw-r--r--]  mina-snark-worker-1.peerid
├── [-rw-rw-r--]  network.json
├── [drwx------]  network-keypairs
│   ├── [-rw-------]  mina-archive
│   ├── [-rw-r--r--]  mina-archive.pub
│   ├── [-rw-------]  mina-bp-1
│   ├── [-rw-r--r--]  mina-bp-1.pub
│   ├── [-rw-------]  mina-bp-2
│   ├── [-rw-r--r--]  mina-bp-2.pub
│   ├── [-rw-------]  mina-seed-1
│   ├── [-rw-r--r--]  mina-seed-1.pub
│   ├── [-rw-------]  mina-snark-coordinator
│   ├── [-rw-r--r--]  mina-snark-coordinator.pub
│   ├── [-rw-------]  mina-snark-worker-1
│   └── [-rw-r--r--]  mina-snark-worker-1.pub
├── [-rw-rw-r--]  replayer_input.json
├── [-rw-rw-r--]  services.json
└── [-rw-rw-r--]  zkapp_tables.sql
```
**Note:** By default, minimina saves files in `$HOME\.minimina`. To customize this location, set the `$MINIMINA_HOME` environment variable, and files will be stored in `$MINIMINA_HOME\.minimina`.

The default network can be started, stopped, and deleted

```bash
minimina network start
minimina network stop
minimina network delete
```

Default network info and status can be queried

```bash
minimina network info
minimina network status
```

### Networks based on Lucy-generated genesis and topology

We also provide two example networks in `./tests/data/`, `small_network` and `large_network`, with all required keys and files. To create and deploy these networks

```bash
# create large-network's docker compose file
minimina network create \
  -g ./tests/data/large_network/genesis_ledger.json \
  -t ./tests/data/large_network/topology.json \
  -n large-network

# start large-network
minimina network start -n large-network

# stop large-network
minimina network stop -n large-network

# delete large-network
minimina network delete -n large-network
```

```bash
# create small-network's docker compose file
minimina network create \
  -g ./tests/data/small_network/genesis_ledger.json \
  -t ./tests/data/small_network/topology.json \
  -n small-network

# start small-network
minimina network start -n small-network

# stop small-network
minimina network stop -n small-network

# delete small-network
minimina network delete -n small-network
```

To monitor the action, you will want to use a resource monitor, like [`bpytop`](https://github.com/aristocratos/bpytop).

### Testing

You can run unit tests using the `cargo test` command:

```bash
cargo test
```

This will run all tests in the project and output the results.

## Contributing

As MiniMina is a work in progress, contributions are always welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the [MIT License](LICENSE).
