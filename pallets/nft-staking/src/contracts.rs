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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{
	tokens::{nonfungibles_v2::Inspect, AttributeNamespace, Balance as RewardBalance},
	ConstU32,
};
use scale_info::TypeInfo;
use sp_runtime::BoundedVec;
use sp_std::fmt::Debug;

/// Struct that represents a combination of an Nft collection id and item id.
/// Used in combination of an [`Inspect`] capable provider.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct NftAddress<CollectionId, ItemId>(pub CollectionId, pub ItemId)
where
	CollectionId: Debug,
	ItemId: Debug;

/// List of Nft assets to be used for contract validation and staking.
/// See also: [`NftAddress`], [`StakingContract`]
pub type StakedAssetsVec<CollectionId, ItemId, const N: u32> =
	BoundedVec<NftAddress<CollectionId, ItemId>, ConstU32<N>>;

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum StakingReward<Balance, CollectionId, ItemId>
where
	Balance: RewardBalance,
	CollectionId: Debug + Copy,
	ItemId: Debug + Copy,
{
	Tokens(Balance),
	Nft(NftAddress<CollectionId, ItemId>),
}

/// Specification for a staking contract, in short it's a list of criteria to be fulfilled,
/// with a given reward after the duration is complete.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct StakingContract<
	Balance,
	CollectionId,
	ItemId,
	AccountId,
	BlockNumber,
	AttributeKey,
	AttributeValue,
	const N: u32,
> where
	Balance: RewardBalance,
	CollectionId: Debug + Copy,
	ItemId: Debug + Copy,
	BlockNumber: Debug + Copy,
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	staking_reward: StakingReward<Balance, CollectionId, ItemId>,
	contract_clauses:
		BoundedVec<ContractClause<AccountId, AttributeKey, AttributeValue>, ConstU32<N>>,
	contract_block_duration: BlockNumber,
}

impl<
		Balance,
		CollectionId,
		ItemId,
		AccountId,
		BlockNumber,
		AttributeKey,
		AttributeValue,
		const N: u32,
	>
	StakingContract<
		Balance,
		CollectionId,
		ItemId,
		AccountId,
		BlockNumber,
		AttributeKey,
		AttributeValue,
		N,
	> where
	Balance: RewardBalance,
	CollectionId: Debug + Copy,
	ItemId: Debug + Copy,
	BlockNumber: Debug + Copy,
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	pub fn new(
		reward: StakingReward<Balance, CollectionId, ItemId>,
		duration: BlockNumber,
	) -> Self {
		Self {
			staking_reward: reward,
			contract_block_duration: duration,
			contract_clauses: BoundedVec::default(),
		}
	}

	pub fn with_clause(
		mut self,
		clause: ContractClause<AccountId, AttributeKey, AttributeValue>,
	) -> Self {
		let _ = self.contract_clauses.try_push(clause);

		self
	}

	pub fn evaluate_for<NftInspector>(
		&self,
		staked_assets: &StakedAssetsVec<CollectionId, ItemId, N>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
	{
		(self.contract_clauses.len() == staked_assets.len())
			.then(|| {
				self.contract_clauses.iter().zip(staked_assets.iter()).all(|(clause, asset)| {
					clause.evaluate_for::<NftInspector, CollectionId, ItemId>(asset)
				})
			})
			.unwrap_or(false)
	}

	pub fn get_reward(&self) -> StakingReward<Balance, CollectionId, ItemId> {
		self.staking_reward.clone()
	}

	pub fn get_duration(&self) -> BlockNumber {
		self.contract_block_duration
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ContractClause<AccountId, AttributeKey, AttributeValue>
where
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	HasAttribute(AttributeNamespace<AccountId>, AttributeKey),
	HasAttributeWithValue(AttributeNamespace<AccountId>, AttributeKey, AttributeValue),
}

impl<AccountId, AttributeKey, AttributeValue>
	ContractClause<AccountId, AttributeKey, AttributeValue>
where
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	pub fn evaluate_for<NftInspector, CollectionId, ItemId>(
		&self,
		asset: &NftAddress<CollectionId, ItemId>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		CollectionId: Debug + Copy,
		ItemId: Debug + Copy,
	{
		let NftAddress(collection_id, item_id) = asset;

		match self {
			ContractClause::HasAttribute(ns, key) => NftInspector::typed_attribute::<
				AttributeKey,
				AttributeValue,
			>(collection_id, item_id, ns, key)
			.is_some(),
			ContractClause::HasAttributeWithValue(ns, key, expected_value) => {
				if let Some(value) = NftInspector::typed_attribute::<AttributeKey, AttributeValue>(
					collection_id,
					item_id,
					ns,
					key,
				) {
					expected_value.eq(&value)
				} else {
					false
				}
			},
		}
	}
}
