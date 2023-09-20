// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::traits::{tokens::nonfungibles_v2::Inspect, ConstU32};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_runtime::BoundedVec;
use sp_std::{fmt::Debug, vec::Vec};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ContractStats {
	pub contracts_staked: u32,
	pub contracts_claimed: u32,
	pub contracts_sniped: u32,
	pub contracts_cancelled: u32,
	pub contracts_lost: u32,
}

/// Attribute namespaces for non-fungible tokens.
/// Based on the logic for
/// https://github.com/paritytech/substrate/blob/polkadot-v0.9.42/frame/nfts/src/types.rs#L326
#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AttributeNamespace {
	Pallet,
	CollectionOwner,
}

/// Type to represent the collection and item IDs of an NFT.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct NftId<CollectionId, ItemId>(pub CollectionId, pub ItemId);

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Reward<Balance, CollectionId, ItemId> {
	Tokens(Balance),
	Nft(NftId<CollectionId, ItemId>),
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ContractClause<CollectionId, KL, VL>
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	pub namespace: AttributeNamespace,
	pub target_index: u8,
	pub clause: Clause<CollectionId, KL, VL>,
}

impl<CollectionId, KL, VL> ContractClause<CollectionId, KL, VL>
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	pub fn evaluate_for<AccountId, NftInspector, ItemId>(
		&self,
		address: &NftId<CollectionId, ItemId>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		CollectionId: PartialEq,
		ItemId: PartialEq,
	{
		let evaluate_fn = match self.namespace {
			AttributeNamespace::Pallet => NftInspector::system_attribute,
			AttributeNamespace::CollectionOwner => NftInspector::attribute,
		};

		self.clause
			.evaluate_for::<AccountId, NftInspector, ItemId, _>(address, evaluate_fn)
	}
}
pub type Attribute<N> = BoundedVec<u8, N>;

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Clause<CollectionId, KL, VL>
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	HasAttribute(CollectionId, Attribute<KL>),
	HasAllAttributes(CollectionId, BoundedVec<Attribute<KL>, ConstU32<10>>),
	HasAnyAttributes(CollectionId, BoundedVec<Attribute<KL>, ConstU32<10>>),
	HasAttributeWithValue(CollectionId, Attribute<KL>, Attribute<VL>),
	HasAllAttributesWithValues(
		CollectionId,
		BoundedVec<(Attribute<KL>, Attribute<VL>), ConstU32<10>>,
	),
	HasAnyAttributesWithValues(
		CollectionId,
		BoundedVec<(Attribute<KL>, Attribute<VL>), ConstU32<10>>,
	),
}

impl<CollectionId, KL, VL> Clause<CollectionId, KL, VL>
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	pub fn evaluate_for<AccountId, NftInspector, ItemId, Fn>(
		&self,
		address: &NftId<CollectionId, ItemId>,
		mut evaluate_fn: Fn,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		CollectionId: PartialEq,
		ItemId: PartialEq,
		Fn: FnMut(&CollectionId, &ItemId, &[u8]) -> Option<Vec<u8>>,
	{
		let clause_collection_id = match self {
			Clause::HasAttribute(collection_id, _) => collection_id,
			Clause::HasAttributeWithValue(collection_id, _, _) => collection_id,
			Clause::HasAllAttributes(collection_id, _) => collection_id,
			Clause::HasAnyAttributes(collection_id, _) => collection_id,
			Clause::HasAllAttributesWithValues(collection_id, _) => collection_id,
			Clause::HasAnyAttributesWithValues(collection_id, _) => collection_id,
		};
		let NftId(collection_id, item_id) = address;
		clause_collection_id == collection_id &&
			(match self {
				Clause::HasAttribute(_, key) =>
					evaluate_fn(collection_id, item_id, key.as_slice()).is_some(),
				Clause::HasAllAttributes(_, attributes) => attributes
					.iter()
					.all(|key| evaluate_fn(collection_id, item_id, key.as_slice()).is_some()),
				Clause::HasAnyAttributes(_, attributes) => attributes
					.iter()
					.any(|key| evaluate_fn(collection_id, item_id, key.as_slice()).is_some()),
				Clause::HasAttributeWithValue(_, key, expected_value) =>
					if let Some(value) = evaluate_fn(collection_id, item_id, key.as_slice()) {
						expected_value.as_slice().eq(value.as_slice())
					} else {
						false
					},
				Clause::HasAllAttributesWithValues(_, attributes) =>
					attributes.iter().all(|(key, expected_value)| {
						if let Some(value) = evaluate_fn(collection_id, item_id, key.as_slice()) {
							expected_value.as_slice().eq(value.as_slice())
						} else {
							false
						}
					}),
				Clause::HasAnyAttributesWithValues(_, attributes) =>
					attributes.iter().any(|(key, expected_value)| {
						if let Some(value) = evaluate_fn(collection_id, item_id, key.as_slice()) {
							expected_value.as_slice().eq(value.as_slice())
						} else {
							false
						}
					}),
			})
	}
}

pub(crate) type BoundedClauses<CollectionId, KL, VL> =
	BoundedVec<ContractClause<CollectionId, KL, VL>, ConstU32<100>>;

pub(crate) type BoundedRewards<Balance, CollectionId, ItemId> =
	BoundedVec<Reward<Balance, CollectionId, ItemId>, ConstU32<5>>;

/// Specification for a staking contract, in short it's a list of criteria to be fulfilled,
/// with a given reward after the duration is complete.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Contract<Balance, CollectionId, ItemId, BlockNumber, KL, VL>
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	/// The block number at which the given contract becomes active for a staker to accept. If it
	/// is not set, the contract activates immediately upon creation.
	pub activation: Option<BlockNumber>,
	/// The duration for which the given contract is active. When the block number advances beyond
	/// this, the contract becomes unavailable to be accepted and can be removed by its creator.
	pub active_duration: BlockNumber,
	/// The duration for which the given contract must be claimed. When the block number advances
	/// beyond it, the contract becomes available to be claimed by other stakers via snipe.
	pub claim_duration: BlockNumber,
	/// The duration for which the given contract must remain staked. When the block number
	/// advances beyond this, the contract becomes claimable by the staker for rewards.
	pub stake_duration: BlockNumber,

	/// The list of conditions to satisfy as staking NFTs. A staker must provide NFTs that meet
	/// these requirements, which will be locked for the staking duration of the contract.
	pub stake_clauses: BoundedClauses<CollectionId, KL, VL>,
	/// The list of conditions to satisfy as fee NFTs. A staker must provide NFTs that meet these
	/// requirements to accept the contract, which is transferred to the contract creator.
	pub fee_clauses: BoundedClauses<CollectionId, KL, VL>,

	/// The rewards of fulfilling the given contract in the form of either tokens or NFTs.
	pub rewards: BoundedRewards<Balance, CollectionId, ItemId>,
	/// The fee required to cancel the given contract. Any staked NFTs for the contract will be
	/// immediately returned to the staker upon cancellation.
	pub cancel_fee: Balance,

	/// Amount of NFT to stake for the contract.
	pub nft_stake_amount: u8,
	/// Amount of NFT to fee for the contract.
	pub nft_fee_amount: u8,

	/// Can the contract be sniped by a third party?
	pub is_snipeable: bool,
}

impl<Balance, CollectionId, ItemId, BlockNumber, KL, VL>
	Contract<Balance, CollectionId, ItemId, BlockNumber, KL, VL>
where
	CollectionId: PartialEq,
	ItemId: PartialEq,
	KL: Get<u32>,
	VL: Get<u32>,
{
	pub fn evaluate_stakes<AccountId, NftInspector>(
		&self,
		stakes: &[NftId<CollectionId, ItemId>],
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
	{
		(self.nft_stake_amount == stakes.len() as u8)
			.then(|| {
				self.stake_clauses.iter().all(|stake_clause| {
					stake_clause.evaluate_for::<AccountId, NftInspector, ItemId>(
						&stakes[stake_clause.target_index as usize],
					)
				})
			})
			.unwrap_or(false)
	}

	pub fn evaluate_fees<AccountId, NftInspector>(
		&self,
		fees: &[NftId<CollectionId, ItemId>],
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
	{
		(self.nft_fee_amount == fees.len() as u8)
			.then(|| {
				self.fee_clauses.iter().all(|fee_clause| {
					fee_clause.evaluate_for::<AccountId, NftInspector, ItemId>(
						&fees[fee_clause.target_index as usize],
					)
				})
			})
			.unwrap_or(false)
	}
}
