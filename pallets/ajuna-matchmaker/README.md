# Ajuna Network Pallet Matchmaker

The runtime module, `pallet-ajuna-matchmaker`, is a generic implementation of bracket-based matchmaking.
It is intended to be used with other game engine pallets to support matchmaking, or queuing for a match, via extrinsic calls.

## Purpose

This pallet acts by matching an arbitrary number of players to play each other. Players are matched by their bracket only.

## Dependencies

### Traits

This pallet implements the `Matchmaker` trait

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-ajuna-matchmaker = { default-features = false, git = "https://github.com/ajuna-network/Ajuna" }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    'pallet-ajuna-matchmaker/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
impl pallet_matchmaker::Config for Test {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
Matchmaker: pallet_ajuna_matchmaker::{Pallet, Storage, Event<T>},
```

### Genesis Configuration

This matchmaker pallet does not have any genesis configuration.

### Types

No additional types

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
