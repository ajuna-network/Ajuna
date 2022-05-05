<p align="center" width="100%">
  <a href="https://ajuna.io" target="_blank">
    <img src="docs/ajuna-banner.jpeg" alt="Ajuna Network">
  </a>
</p>

[![Build](https://github.com/ajuna-network/ajuna-node/actions/workflows/check-pull-request.yml/badge.svg)](https://github.com/ajuna-network/ajuna-node/actions/workflows/check-pull-request.yml)

A [Substrate](https://www.substrate.io/)-based blockchain implementation, ready for hacking :rocket:

## Prerequisites

- [Build dependencies](https://docs.substrate.io/v3/getting-started/installation/#1-build-dependencies)
- [Rust](https://www.rust-lang.org/tools/install)
- [OnFinality CLI](https://github.com/OnFinality-io/onf-cli#installation)

## Build

- Using `cargo`:

  ```bash
  cargo build --release --features solo   # solochain
  cargo build --release --features bajun  # parachain with Bajun runtime
  ```

- Using `Docker`:

  ```bash
  docker build -f docker/Dockerfile.solochain -t ajuna/solochain:latest .  # solochain
  docker build -f docker/Dockerfile.parachain -t ajuna/parachain:latest .  # parachain
  ```

## Run

- Using compiled binaries:

  ```bash
  ./target/release/ajuna-solo --dev --tmp  # solochain
  ```

- Using `Docker`:

  ```bash
  docker-compose -f docker/solochain.yml up  # solochain
  docker-compose -f docker/parachain.yml up  # parachain
  ```

## Deploy

- Using OnFinality:

  ```bash
  onf network bootstrap -f .onf/bajun-rococo-testnet.yml
  ```
