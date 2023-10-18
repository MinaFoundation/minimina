# MiniMina

[![Build](https://github.com/MinaFoundation/minimina/actions/workflows/build.yaml/badge.svg)](https://github.com/MinaFoundation/minimina/actions/workflows/build.yaml)

## Overview

MiniMina is a command line tool aimed at providing the capability to spin up Mina networks locally on a user's computer. For more more information see [Minimina RFC](/docs/rfc/README.md).

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

 - [Default network](https://github.com/MinaFoundation/minimina/wiki/Default-network)
 - [Networks based on Lucy-generated genesis and topology](https://github.com/MinaFoundation/minimina/wiki/Networks-based-on-Lucy%E2%80%90generated-genesis-and-topology)
 - [Network with uptime-service-backend](https://github.com/MinaFoundation/minimina/wiki/Network-with-uptime%E2%80%90service%E2%80%90backend)

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
