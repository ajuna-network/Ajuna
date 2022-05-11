# Ajuna Network Pallet Game Registry

The Game Registry controls Layer 2 computation; this is not restricted to games. It can also be some fast-paced interactions that need to be on the side chain.

## Purpose

This pallet acts as a game registry for games between L1 and L2, with Ajuna TEE. It has a synergy with `pallet-ajuna-matchmaker`.

![GameRegistry](https://user-images.githubusercontent.com/17710198/142016775-9f8b5845-da6e-47ed-afb9-e86b0f6fe18f.png)

## Dependencies

### Traits

This pallet depends on `pallet-ajuna-matchmaker`.

### Pallets

This pallet utilizes `pallet-ajuna-matchmaker` for its underlying queue implementation, as per the design above.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-ajuna-matchmaker = { default-features = false, git = "https://github.com/ajuna-network/Ajuna" }
pallet-ajuna-gameregistry = { default-features = false, git = "https://github.com/ajuna-network/Ajuna" }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    'pallet-ajuna-gameregistry/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
parameter_types! {

}

impl pallet_ajuna_gameregistry::Config for Runtime {
    type Event = Event;
    type Proposal = Call;
    type Randomness = Randomness;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MatchMaker = Matchmaker;
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
    Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
    Matchmaker: pallet_ajuna_matchmaker::{Pallet, Storage, Event<T>},
    GameRegistry: pallet_ajuna_gameregistry::{Pallet, Call, Config<T>, Storage, Event<T>},
  }
);
```

### Genesis Configuration

```rust
#[pallet::genesis_config]
pub struct GenesisConfig<T: Config> {
  /// The founder key is used for administration, much like the sudo key
  pub founder_key: T::AccountId,
  // The maximum number of games that can be acknowledged in a single batch
  max_acknowledge_batch: u32,
  // The maximum queue size for a game engine
  max_queue_size: u8,
  // The maximum number of games that can be handled in a block
  max_games_per_block: u8,
}
```

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
