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

### Prerequisites

MiniMina is written in Rust and requires the Rust compiler and its package manager, Cargo. If you don't have Rust installed, you can install it from the official website: https://www.rust-lang.org/tools/install.

MiniMina will use `docker` and requires it to be installed on user's machine. See: https://docs.docker.com/engine/install/.

### Building

To build MiniMina, use the `cargo build --release` command in the root directory of the repository:

```bash
cargo build --release
```

This will produce an executable file in the `target/release/` directory.

### Testing

You can run tests using the `cargo test` command:

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
