use frame_support::pallet_prelude::*;
use sp_core::sp_std::vec::Vec;

pub trait OnMappingRequest<AssetId, CollectionId, ItemId> {
	fn on_fungible_asset_mapping(id: WideId) -> AssetId;
	fn on_non_fungible_collection_mapping(id: WideId) -> CollectionId;
	fn on_non_fungible_item_mapping(id: WideId) -> ItemId;
}

pub type WideId = sp_core::H256;

pub(crate) const NATIVE_FUNGIBLE_PAD: [u8; 30] = [
	0xff, 0x42, 0x41, 0x4a, 0x55, 0x4e, 0x5f, 0x4e, 0x45, 0x54, 0x57, 0x4f, 0x52, 0x4b, 0x5f, 0x4f,
	0x4e, 0x5f, 0x4b, 0x55, 0x53, 0x41, 0x4d, 0x41, 0x5f, 0x31, 0x33, 0x33, 0x37, 0xda,
];

pub(crate) const NATIVE_NON_FUNGIBLE_PAD: [u8; 30] = [
	0x77, 0x42, 0x41, 0x4a, 0x55, 0x4e, 0x5f, 0x4e, 0x45, 0x54, 0x57, 0x4f, 0x52, 0x4b, 0x5f, 0x4f,
	0x4e, 0x5f, 0x4b, 0x55, 0x53, 0x41, 0x4d, 0x41, 0x5f, 0x31, 0x33, 0x33, 0x37, 0xda,
];

pub(crate) const BALANCE_PROOF_PREFIX: &[u8] = b"Erdstall balance proof";
pub(crate) const FREEZE_PROOF_PREFIX: &[u8] = b"Erdstall freeze proof";
pub(crate) const ZERO_BALANCE_PROOF_PREFIX: &[u8] = b"Erdstall zero balance";
pub(crate) const LIGHT_CLIENT_PROOF_PREFIX: &[u8] = b"Is a certified light client";

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum AssetType {
	Fungible = 0,
	NonFungible = 1,
}

impl From<u8> for AssetType {
	fn from(value: u8) -> Self {
		match value {
			0 => Self::Fungible,
			1 => Self::NonFungible,
			_ => Self::Fungible,
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct AssetDeposit {
	pub origin: AssetOrigin,
	pub asset_type: AssetType,
	pub primary_id: WideId,
	pub secondary_id: WideId,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// Represents the address in which the non-fungible token can be located.
pub struct NftAddress<CollectionId, ItemId>(pub CollectionId, pub ItemId)
where
	CollectionId: Encode,
	ItemId: Encode;

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// Describes which kind of asset are we managing
pub enum AssetKind<AssetId, CollectionId, ItemId>
where
	AssetId: Encode,
	CollectionId: Encode,
	ItemId: Encode,
{
	Fungible(AssetId),
	NonFungible(NftAddress<CollectionId, ItemId>),
}

/// Indicates the chain/ecosystem from which a given asset is native to.
pub type AssetOrigin = u16;

pub type ChainId = u16;

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Asset<AssetId, CollectionId, ItemId>
where
	AssetId: Encode,
	CollectionId: Encode,
	ItemId: Encode,
{
	pub origin: AssetOrigin,
	pub kind: AssetKind<AssetId, CollectionId, ItemId>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// Indicates which king of value are we depositing for a fungible asset
pub enum DepositValue<TokenBalance, AssetBalance> {
	Token(TokenBalance),
	Asset(AssetBalance),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
/// Indicates if we are depositing a fungible or non-fungible asset
pub enum DepositValueKind<TokenBalance, AssetBalance> {
	Fungible(DepositValue<TokenBalance, AssetBalance>),
	NonFungible,
}

pub type ChunkIndex = u32;
pub type EpochNumber = u64;

pub trait Proof {
	fn extract_msg(&self) -> Vec<u8>;
}

#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BalanceProof {
	pub epoch: EpochNumber,
	pub origin: ChainId,
	pub account: WideId,
	pub exit_flag: bool,
	pub chunk_index: ChunkIndex,
	pub chunk_last: ChunkIndex,
	pub asset_origin: AssetOrigin,
	pub asset_type: u8,
	pub primary_id: WideId,
	pub secondary_id: WideId,
}

impl Proof for BalanceProof {
	fn extract_msg(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(0);

		bytes.extend(BALANCE_PROOF_PREFIX.to_vec());
		bytes.extend(self.epoch.encode());
		bytes.extend(self.origin.encode());
		bytes.extend(self.account.encode());
		bytes.extend(self.exit_flag.encode());
		bytes.extend(self.chunk_index.encode());
		bytes.extend(self.chunk_last.encode());
		bytes.extend(self.asset_origin.encode());
		bytes.extend(self.asset_type.encode());
		bytes.extend(self.primary_id.encode());
		bytes.extend(self.secondary_id.encode());

		bytes
	}
}

impl BalanceProof {
	#[cfg(test)]
	pub(crate) fn using_deposit(
		deposit: &AssetDeposit,
		epoch: EpochNumber,
		account: crate::tests::mock::MockAccountId,
		exit_flag: bool,
		chunk_index: ChunkIndex,
	) -> Self {
		let account_encoded = {
			let account_bytes = {
				let mut buff = account.encode();
				buff.extend(&[0x00; 24]);
				buff
			};

			WideId::from_slice(account_bytes.as_slice())
		};

		Self {
			epoch,
			origin: deposit.origin,
			account: account_encoded,
			exit_flag,
			chunk_index,
			chunk_last: 0,
			asset_origin: deposit.origin,
			asset_type: deposit.asset_type as u8,
			primary_id: deposit.primary_id,
			secondary_id: deposit.secondary_id,
		}
	}

	#[cfg(test)]
	pub(crate) fn using_challenge(
		epoch: EpochNumber,
		account: crate::tests::mock::MockAccountId,
		chunk_index: ChunkIndex,
		chunk_last: ChunkIndex,
	) -> Self {
		let account_encoded = {
			let account_bytes = {
				let mut buff = account.encode();
				buff.extend(&[0x00; 24]);
				buff
			};

			WideId::from_slice(account_bytes.as_slice())
		};

		Self {
			epoch,
			origin: crate::tests::mock::CHAIN_ID,
			account: account_encoded,
			exit_flag: true,
			chunk_index,
			chunk_last,
			asset_origin: crate::tests::mock::CHAIN_ID,
			asset_type: AssetType::Fungible as u8,
			primary_id: WideId::default(),
			secondary_id: WideId::default(),
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct FreezeProof {
	pub epoch: EpochNumber,
	pub origin: ChainId,
	pub identifier: WideId,
}

impl Proof for FreezeProof {
	fn extract_msg(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(0);

		bytes.extend(FREEZE_PROOF_PREFIX.to_vec());
		bytes.extend(self.epoch.encode());
		bytes.extend(self.origin.encode());
		bytes.extend(self.identifier.encode());

		bytes
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ZeroBalanceProof {
	pub epoch: EpochNumber,
	pub origin: ChainId,
	pub account: WideId,
}

impl Proof for ZeroBalanceProof {
	fn extract_msg(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(0);

		bytes.extend(ZERO_BALANCE_PROOF_PREFIX.to_vec());
		bytes.extend(self.epoch.encode());
		bytes.extend(self.origin.encode());
		bytes.extend(self.account.encode());

		bytes
	}
}

impl ZeroBalanceProof {
	#[cfg(test)]
	pub(crate) fn using_challenge(
		epoch: EpochNumber,
		account: crate::tests::mock::MockAccountId,
	) -> Self {
		let account_encoded = {
			let account_bytes = {
				let mut buff = account.encode();
				buff.extend(&[0x00; 24]);
				buff
			};

			WideId::from_slice(account_bytes.as_slice())
		};

		Self { epoch, origin: crate::tests::mock::CHAIN_ID, account: account_encoded }
	}
}
