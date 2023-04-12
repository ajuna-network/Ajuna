# Ajuna Network Awesome Avatars Pallet

The Ajuna Awesome Avatars Pallet provides necessary extrinsics and storage for the collectible game
[AAA](https://aaa.ajuna.io). It allows:

- an organizer to manage the game and its seasons with various configurations
- players to obtain new avatars via minting, forging and trading
- players to trade avatars via setting / removing price for their avatars and buying others
- players to upgrade storage to hold more avatars

## Overview

The pallet must be initialized with a root call to set an account to act as an organizer.
The organizer can then set seasons with parameters to control various aspects of the game such as
the name, description and duration of a season as well as probabilities that affect forging
algorithm. When the network's block number reaches that of a season start, the season becomes active
and season-specific avatars can be obtained, which will no longer be available once the season
finishes. Avatars from previous seasons are available for trade if their owners are willing to sell.

An optional requirement for the pallet is an account to act as a season treasurer. Each season can
optionally have an associated treasurer who can claim the season's treasury once the season
finishes. It can be used as rewards for accounts who have contributed to a particular season.

## Integration

### Runtime `Cargo.toml`

To add this pallet to your runtime, include the following to your runtime's `Cargo.toml` file:

```toml
pallet-ajuna-awesome-avatars = { default-features = false, path = "../../pallets/ajuna-awesome-avatars" }

std = [
    "pallet-ajuna-awesome-avatars/std",
]
```

### Runtime `lib.rs`

You should implement its trait like so:

```rust
parameter_types! {
    pub const AwesomeAvatarsPalletId: PalletId = PalletId(*b"aj/aaatr");
}

impl pallet_ajuna_awesome_avatars::Config for Runtime {
    type PalletId = AwesomeAvatarsPalletId;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Randomness = Randomness;
    type NftHandler = NftTransfer;
    type WeightInfo = pallet_ajuna_awesome_avatars::weights::AjunaWeight<Runtime>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_balances::Config for Runtime {
    // -- snip --
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
        AwesomeAvatars: pallet_ajuna_awesome_avatars,
        Balances: pallet_balances,
        Randomness: pallet_randomness_collective_flip,
    }
);
```

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
