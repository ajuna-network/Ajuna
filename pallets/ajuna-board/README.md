# Ajuna Network Pallet Board

The runtime module, `pallet-ajuna-board`

## Purpose


## Dependencies

### Traits


### Pallets


## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-ajuna-board = { default-features = false, git = "https://github.com/ajuna-network/Ajuna" }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    'pallet-ajuna-board/std',
]
```

### Runtime `lib.rs`

You should implement its trait like so:

```rust
impl pallet_board::Config for Test {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
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
