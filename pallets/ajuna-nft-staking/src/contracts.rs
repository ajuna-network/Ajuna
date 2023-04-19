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
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{nonfungibles_v2::Inspect, Balance as BalanceT},
		ConstU32,
	},
};
use scale_info::TypeInfo;
use sp_runtime::BoundedVec;
use sp_std::fmt::Debug;

/// Struct that represents a combination of an Nft collection id and item id.
/// Used in combination of an [`Inspect`] capable provider.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct NftAddress<CollectionId, ItemId>(pub CollectionId, pub ItemId);

/// List of Nft assets to be used for contract validation and staking.
/// See also: [`NftAddress`], [`Contract`]
pub type StakedAssetsVec<CollectionId, ItemId, const N: u32> =
	BoundedVec<NftAddress<CollectionId, ItemId>, ConstU32<N>>;

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Reward<Balance, CollectionId, ItemId> {
	Tokens(Balance),
	Nft(NftAddress<CollectionId, ItemId>),
}

/// Specification for a staking contract, in short it's a list of criteria to be fulfilled,
/// with a given reward after the duration is complete.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Contract<
	Balance,
	CollectionId,
	ItemId,
	BlockNumber,
	AttributeKey,
	AttributeValue,
	const N: u32,
> {
	pub reward: Reward<Balance, CollectionId, ItemId>,
	contract_clauses: BoundedVec<Clause<AttributeKey, AttributeValue>, ConstU32<N>>,
	pub duration: BlockNumber,
}

impl<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue, const N: u32>
	Contract<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue, N>
where
	Balance: BalanceT,
	CollectionId: Debug + Copy,
	ItemId: Debug + Copy,
	BlockNumber: Debug + Copy,
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	pub fn new(reward: Reward<Balance, CollectionId, ItemId>, duration: BlockNumber) -> Self {
		Self { reward, duration, contract_clauses: BoundedVec::default() }
	}

	pub fn with_clause(mut self, clause: Clause<AttributeKey, AttributeValue>) -> Self {
		let _ = self.contract_clauses.try_push(clause);

		self
	}

	pub fn evaluate_for<NftInspector, AccountId>(
		&self,
		staked_assets: &StakedAssetsVec<CollectionId, ItemId, N>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		AccountId: Parameter + Member,
	{
		(self.contract_clauses.len() == staked_assets.len())
			.then(|| {
				self.contract_clauses.iter().zip(staked_assets.iter()).all(|(clause, asset)| {
					clause.evaluate_for::<NftInspector, AccountId, CollectionId, ItemId>(asset)
				})
			})
			.unwrap_or(false)
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Clause<AttributeKey, AttributeValue> {
	HasAttribute(AttributeKey),
	HasAttributeWithValue(AttributeKey, AttributeValue),
}

impl<AttributeKey, AttributeValue> Clause<AttributeKey, AttributeValue>
where
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	pub fn evaluate_for<NftInspector, AccountId, CollectionId, ItemId>(
		&self,
		asset: &NftAddress<CollectionId, ItemId>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		AccountId: Parameter + Member,
		CollectionId: Debug + Copy,
		ItemId: Debug + Copy,
	{
		let NftAddress(collection_id, item_id) = asset;

		match self {
			Clause::HasAttribute(key) =>
				NftInspector::system_attribute(collection_id, item_id, &key.encode()).is_some(),
			Clause::HasAttributeWithValue(key, expected_value) => {
				if let Some(value) =
					NftInspector::system_attribute(collection_id, item_id, &key.encode())
				{
					expected_value.eq(&AttributeValue::decode(&mut value.as_slice()).unwrap())
				} else {
					false
				}
			},
		}
	}
}
