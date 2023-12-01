# Ajuna NFT-Transfer Pallet

The Ajuna NFT Transfer Pallet provides necessary functionalities to tokenize an arbitrary piece of
data that supports the SCALE codec into an appropriate NFT representation. It interfaces with the
non-fungible traits to support their arbitrary NFT standards and underlying storage solutions.

## Overview

The pallet must be initialized with a collection ID, created externally via `pallet-nfts`, to group
similar NFTs under the same collection. In order to store and recover NFTs, the `NftConvertible`
trait must be implemented by the objects of interest. When storing NFTs, the owners pay for the
associated deposit amount, which is fully refunded when the NFTs are recovered back into their
original form.

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
construct_runtime! {
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
