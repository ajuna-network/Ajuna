# Ajuna Network Awesome Avatars Pallet

Ajuna Network Awesome Avatars logic.

## Purpose

TODO

## Dependencies

TODO

### Traits

TODO

### Pallets

TODO

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-ajuna-awesome-avatars = { default-features = false, path = "../../pallets/ajuna-awesome-avatars" }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    'pallet-ajuna-awesome-avatars/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
parameter_types! {

}

impl pallet_ajuna_awesome_avatars::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}
```

and include it in your `construct_runtime!` macro:

```rust

construct_runtime!(
  pub enum Runtime where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    AAA: pallet_ajuna_awesome_avatars,
  }
);
```

### Genesis Configuration

TODO

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
