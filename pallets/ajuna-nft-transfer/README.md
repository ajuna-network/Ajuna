# Ajuna NFT-Transfer Pallet

This pallet provides utilities for transforming any given asset into a serialized form which can later be uploaded to an IPFS server provider.

## Integration

## Runtime `Cargo.toml`

To add this pallet to your runtime, include the following to your runtime's `Cargo.toml` file:

```toml
pallet-ajuna-nft-transfer = { default-features = false, path = "../../pallets/ajuna-nft-transfer" }

std = [
    "pallet-ajuna-nft-transfer/std",
]
```

### Runtime `lib.rs`

You should implement its trait like so:

```rust
parameter_types! {
    pub const NftTransferPalletId: PalletId = PalletId(*b"aj/nfttr");
}

impl pallet_ajuna_nft_transfer::Config for Runtime {
    type PalletId = NftTransferPalletId;
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = CollectionId;
	type ItemId = Hash;
	type ItemConfig = pallet_nfts::ItemConfig;
	type NftHelper = Nft;
}

impl pallet_nfts::Config for Runtime {
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
        NftTransfer: pallet_ajuna_nft_transfer,
        Nft: pallet_nfts,
    }
);
```

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
