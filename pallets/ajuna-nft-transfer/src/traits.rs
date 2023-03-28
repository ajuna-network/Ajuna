use codec::{Codec, Error as CodecError, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	Parameter,
};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::vec::Vec;

/// Type used to differentiate attribute codes for each Asset.
pub type AssetCode = u16;
pub type AttributeCode = u16;

/// Marker trait for Assets that can be converted back and forth into an NFT representation.
pub trait NftConvertible: Codec {
	/// Numeric key used to store this specific asset's attributes in the NFT.
	const ASSET_CODE: AssetCode;

	/// Encodes the asset into a byte representation for storage.
	fn encode_into(self) -> Vec<u8> {
		self.encode()
	}

	/// Decodes a given byte representation back into it's asset form.
	/// Returns None if decoding fails or if input is empty.
	fn decode_from(input: Vec<u8>) -> Result<Self, CodecError> {
		Self::decode(&mut input.as_slice())
	}

	/// Returns the list of attribute keys associated with the specific type.
	fn get_attribute_table() -> Vec<AttributeCode>;

	/// Returns a list of key-value pairs with the attributes to attach to the encoded asset.
	fn get_encoded_attributes(&self) -> Vec<(AttributeCode, Vec<u8>)>;
}

/// Trait to define the transformation and bridging of assets as NFT.
pub trait NftHandler<Account, Asset: NftConvertible, AssetConfig> {
	type CollectionId: AtLeast32BitUnsigned + Codec + Parameter + MaxEncodedLen;
	type AssetId: Codec + Parameter + MaxEncodedLen;

	/// Consumes the given `asset` and its associated `collection_id`, and stores it as an NFT
	/// owned by `owner`. Returns the index of the NFT for tracking and recovering the asset.
	fn store_as_nft(
		owner: Account,
		collection_id: Self::CollectionId,
		asset: Asset,
		asset_config: AssetConfig,
	) -> Result<Self::AssetId, DispatchError>;

	/// Recovers the NFT indexed by `collection_id` and `asset_id`, and transforms it back into an
	/// asset. Returns an appropriate error if the process fails.
	fn recover_from_nft(
		owner: Account,
		collection_id: Self::CollectionId,
		asset_id: Self::AssetId,
	) -> Result<Asset, DispatchError>;

	/// Schedules a previously stored NFT asset to be transferred outside of the chain,
	/// once this process completes the NFT won't be recoverable until the asset is transferred back
	/// from the outside of the chain.
	fn schedule_nft_upload(
		owner: Account,
		collection_id: Self::CollectionId,
		asset_id: Self::AssetId,
	) -> DispatchResult;
}
