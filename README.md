<p align="center" width="100%">
  <a href="https://ajuna.io" target="_blank">
    <img src="docs/ajuna-banner.jpeg" alt="Ajuna Network">
  </a>
</p>

[![Build](https://github.com/ajuna-network/Ajuna/actions/workflows/check-pull-request.yml/badge.svg?branch=main)](https://github.com/ajuna-network/Ajuna/actions/workflows/check-pull-request.yml)
[![codecov](https://codecov.io/gh/ajuna-network/Ajuna/branch/main/graph/badge.svg?token=V2Y88ZUD6C)](https://codecov.io/gh/ajuna-network/Ajuna)
[![Docker Image Version (latest semver)](https://img.shields.io/docker/v/ajuna/parachain-bajun?label=bajun%20network&logo=docker&sort=semver&style=plastic)](https://hub.docker.com/repository/docker/ajuna/parachain-bajun/tags?page=1&ordering=last_updated)
[![Docker Image Version (latest semver)](https://img.shields.io/docker/v/ajuna/parachain-ajuna?label=ajuna%20network&logo=docker&sort=semver&style=plastic)](https://hub.docker.com/repository/docker/ajuna/parachain-ajuna/tags?page=1&ordering=last_updated)

A game platform [parachain](https://wiki.polkadot.network/docs/learn-parachains) built with [Substrate](https://docs.substrate.io/).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Substrate environment](https://docs.substrate.io/install/)

## Build

- Using `cargo`:

  ```bash
  # solochain
  cargo build-ajuna-solo

  # parachain with Bajun runtime
  cargo build-bajun-rococo
  cargo build-bajun-kusama

  # parachain with Bajun runtime
  cargo build-ajuna-rococo
  cargo build-ajuna-polkadot
  ```

- Using `Docker`:

  ```bash
  # solochain
  docker build -f docker/Dockerfile -t ajuna/solochain:latest . --build-arg features=solo  --build-arg bin=ajuna-solo

  # parachain with Bajun runtime
  docker build -f docker/Dockerfile -t ajuna/parachain-bajun:latest . --build-arg features=bajun --build-arg bin=bajun-para

  # parachain with Ajuna runtime
  docker build -f docker/Dockerfile -t ajuna/parachain-ajuna:latest . --build-arg features=ajuna --build-arg bin=ajuna-para
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
