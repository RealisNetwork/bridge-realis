# Realis Bridge
<p align="center">
  <img src="https://github.com/Daelon02/realis/blob/main/Group%202000x500.png" width="460">
</p>

<div align="center">

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/AcalaNetwork/Acala/Test?label=Actions&logo=github)](https://github.com/cryptosoulgame/Realis.Network/actions?query=workflow%3ATest)
[![GitHub tag (latest by date)](https://img.shields.io/badge/tag-v1.0.2-blue)](https://github.com/cryptosoulgame/Realis.Network/tags)
[![Substrate version](https://img.shields.io/badge/Substrate-3.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![License](https://img.shields.io/github/license/AcalaNetwork/Acala?color=green)](https://github.com/cryptosoulgame/Realis.Network/blob/main/LICENSE)
<br />
[![Twitter URL](https://img.shields.io/twitter/url?style=social&url=https%3A%2F%2Ftwitter.com%2FAcalaNetwork)](https://twitter.com/realisnetwork)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/RealisNetwork)
[![Medium](https://img.shields.io/badge/Medium-gray?logo=medium)](https://realisnetwork.medium.com/)

</div>

## Getting Started

This is the bridge from BSC to Realis Network

### Rust Setup

Follow the [Rust setup instructions](./doc/rust-setup.md) before using the included Makefile to
build the Bridge.

### Makefile

This project uses a [Makefile](Makefile) to document helpful commands and make it easier to execute
them. Get started by running these [`make`](https://www.gnu.org/software/make/manual/make.html)
targets:

1. `make run` - Build and launch this project.
2. `make build` - Build all project.

This project work at nightly and stable versions Rust.

### Build

The `make run` command will perform an initial build. Use the following command to build the bridge
without launching it:

```sh
make build
```

Also, for launching bridge need launch command:
```
make run
```
