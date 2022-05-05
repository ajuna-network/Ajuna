# Ajuna Network Pallet Runner

The runtime module, `pallet-ajuna-runner`, stores a arbitrary number of `Runners`.  A `Runner` is any off-chain process that
may be running in which the state of this off-chain process is stored on chain.

It is intended to be used by using the `Runner` trait by other pallets internally. 

## Purpose

The pallet stores state of off-chain processes, which are managed via the `Runner` trait.

## Dependencies

### Traits

This pallet implements the `Runner` trait

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-ajuna-runner = { default-features = false, version = "4.0.0-dev", git = "https://github.com/ajuna-network/ajuna-node.git" }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    'pallet-ajuna-runner/std',
]
```

### Runtime `lib.rs`

You should implement its trait like so:

```rust
impl pallet_runner::Config for Test {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
MatchMaker: pallet_ajuna_runner::{Pallet, Storage, Event<T>},
```

### Genesis Configuration

This runner pallet does not have any genesis configuration.

### Types

No additional types

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
