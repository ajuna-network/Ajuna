# Ajuna Network Awesome Avatars Pallet

Ajuna Network Awesome Avatars logic.

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
