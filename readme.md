# Rusty_Torrent BitTorrent Client

![GitHub](https://img.shields.io/github/license/ArloFilley/rusty_torrent)
![GitHub last commit](https://img.shields.io/github/last-commit/ArloFilley/rusty_torrent)
![GitHub stars](https://img.shields.io/github/stars/ArloFilley/rusty_torrent?style=social)

A BitTorrent client implemented in Rust that allows you to interact with the BitTorrent protocol and download torrents.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
- [How It Works](#how-it-works)
- [Contributing](#contributing)
- [License](#license)

## Introduction

This BitTorrent client is designed to provide a simple and functional implementation of the BitTorrent protocol. It supports downloading torrents and interacting with peers to exchange pieces of files.

## Features

- Handshake and communication with peers using the BitTorrent protocol.
- Support for downloading torrents in both single-file and multi-file mode.
- Ability to request and download individual pieces from peers.
- Piece verification using SHA-1 hashes to ensure data integrity.
- Logging using the `log` crate for better debugging and tracing.

## Getting Started

### Prerequisites

- Rust programming language: [Install Rust](https://www.rust-lang.org/tools/install)
- Cargo: The Rust package manager, usually installed with Rust.

### Installation

1. Clone the repository:

```bash
git clone https://github.com/ArloFilley/rusty_torrent.git
```

2. Navigate to the project directory:

```bash
cd rusty_torrent
```

3. Build the project

```bash
cargo build --release
```

### Usage

To use the BitTorrent client, follow these steps:

1. Run the compiled binary:

```bash
cargo run --release
```

2. Provide the path to a .torrent file:

```bash
cargo run --release /path/to/your.torrent
```

3. Provide the path to download 
```bash
cargo run --release /path/to/your.torrent /path/to/downloads
```

The client will start downloading the torrent files and interacting with peers.

## How It Works

This BitTorrent client uses Rust's asynchronous programming features to manage connections with peers and perform file downloads. It employs the BitTorrent protocol's handshake and communication mechanisms to exchange pieces of data with other peers in the network. The client also verifies downloaded pieces using SHA-1 hashes provided by the torrent file.

## Contributing

Contributions are welcome! If you find any bugs or want to add new features, please feel free to open issues and pull requests on the GitHub repository.

## License

This project is licensed under the MIT License.