use codec::{Decode, Encode, Error as CodecError};
use frame_support::dispatch::DispatchResult;

/// Type used to differentiate attribute codes for each Asset.
pub type AssetCode = u16;

/// Marker trait for Assets that can be converted back and forth into an NFT representation.
pub trait NftConvertible: Encode + Decode {
	/// Returns the numeric key used to store this specific asset's attributes in the NFT.
	fn get_asset_code() -> AssetCode;

	/// Encodes the asset into a byte representation for storage.
	fn encode_into(self) -> Vec<u8> {
		self.encode()
	}

	/// Decodes a given byte representation back into it's asset form.
	/// Returns None if decoding fails or if input is empty.
	fn decode_from(input: Vec<u8>) -> Result<Self, CodecError> {
		Self::decode(&mut input.as_slice())
	}
}

/// Trait to define the transformation and bridging of assets as NFT.
pub trait NftHandler<Account, CollectionId, AssetId, Asset: NftConvertible, ItemConfig> {
	/// Consumes the given **asset** and stores it as an NFT owned by **owner**,
	/// returns the NFT index for tracking and recovering the asset.
	fn store_as_nft(
		owner: Account,
		collection_id: CollectionId,
		asset: Asset,
		asset_config: Option<ItemConfig>,
	) -> Result<AssetId, sp_runtime::DispatchError>;
	/// Attempts to recover the NFT indexed by **nft_id** and transform it back into an
	/// asset, returns an appropriate error if the process fails.
	fn recover_from_nft(
		owner: Account,
		collection_id: CollectionId,
		nft_id: AssetId,
	) -> Result<Asset, sp_runtime::DispatchError>;
	/// Schedules a previously stored NFT asset to be transferred outside of the chain,
	/// once this process completes the NFT won't be recoverable until the asset is transferred back
	/// from the outside of the chain.
	fn schedule_nft_upload(
		owner: Account,
		collection_id: CollectionId,
		nft_id: AssetId,
	) -> DispatchResult;
}
