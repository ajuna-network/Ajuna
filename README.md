<p align="center" width="100%">
  <a href="https://ajuna.io" target="_blank">
    <img src="docs/ajuna-banner.jpeg" alt="Ajuna Network">
  </a>
</p>

[![Build](https://github.com/ajuna-network/Ajuna/actions/workflows/check-pull-request.yml/badge.svg)](https://github.com/ajuna-network/Ajuna/actions/workflows/check-pull-request.yml)

A [Substrate](https://www.substrate.io/)-based blockchain implementation, ready for hacking :rocket:

## Prerequisites

- [Build dependencies](https://docs.substrate.io/v3/getting-started/installation/#1-build-dependencies)
- [Rust](https://www.rust-lang.org/tools/install)
- [OnFinality CLI](https://github.com/OnFinality-io/onf-cli#installation)

## Build

- Using `cargo`:

  ```bash
  # solochain
  cargo build-ajuna-solo

  # parachain with Bajun runtime
  cargo build-bajun-rococo
  cargo build-bajun-kusama
  ```

- Using `Docker`:

  ```bash
  # solochain
  docker build -f docker/Dockerfile -t ajuna/solochain:latest . --build-arg features=solo  --build-arg bin=ajuna-solo

  # parachain with Bajun runtime
  docker build -f docker/Dockerfile -t ajuna/parachain:latest . --build-arg features=bajun --build-arg bin=ajuna-para
  ```

## Run

- Using compiled binaries:

  ```bash
  # solochain
  ./target/release/ajuna-solo --dev --tmp
  ```

- Using `Docker`:

  ```bash
   # solochain
  docker-compose -f docker/solochain.yml up

  # parachain with rococo-local relay chain
  docker-compose -f docker/parachain.yml up
  ```

## Deploy

- Using OnFinality:

  ```bash
  onf network bootstrap -f .onf/bajun-rococo-testnet.yml
  ```
