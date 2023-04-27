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
use frame_support::traits::{tokens::nonfungibles_v2::Inspect, ConstU32};
use scale_info::TypeInfo;
use sp_runtime::BoundedVec;
use sp_std::fmt::Debug;

/// Type to represent the collection and item IDs of an NFT.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct NftAddress<CollectionId, ItemId>(pub CollectionId, pub ItemId);

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Reward<Balance, CollectionId, ItemId> {
	Tokens(Balance),
	Nft(NftAddress<CollectionId, ItemId>),
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Clause<CollectionId, AttributeKey, AttributeValue> {
	HasAttribute(CollectionId, AttributeKey),
	HasAttributeWithValue(CollectionId, AttributeKey, AttributeValue),
}

impl<CollectionId, AttributeKey, AttributeValue>
	Clause<CollectionId, AttributeKey, AttributeValue>
{
	pub fn evaluate_for<AccountId, NftInspector, ItemId>(
		&self,
		address: &NftAddress<CollectionId, ItemId>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		CollectionId: PartialEq,
		ItemId: PartialEq,
		AttributeKey: Encode,
		AttributeValue: Encode + Decode + PartialEq,
	{
		let clause_collection_id = match self {
			Clause::HasAttribute(collection_id, _) => collection_id,
			Clause::HasAttributeWithValue(collection_id, _, _) => collection_id,
		};
		let NftAddress(collection_id, item_id) = address;
		clause_collection_id == collection_id &&
			(match self {
				Clause::HasAttribute(_, key) =>
					NftInspector::system_attribute(collection_id, item_id, &key.encode()).is_some(),
				Clause::HasAttributeWithValue(_, key, expected_value) =>
					if let Some(value) =
						NftInspector::system_attribute(collection_id, item_id, &key.encode())
					{
						expected_value.eq(&AttributeValue::decode(&mut value.as_slice()).unwrap())
					} else {
						false
					},
			})
	}
}

type BoundedClauses<CollectionId, AttributeKey, AttributeValue> =
	BoundedVec<Clause<CollectionId, AttributeKey, AttributeValue>, ConstU32<100>>;

/// Specification for a staking contract, in short it's a list of criteria to be fulfilled,
/// with a given reward after the duration is complete.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Contract<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue> {
	pub reward: Reward<Balance, CollectionId, ItemId>,
	pub duration: BlockNumber,
	pub expire_after: BlockNumber,
	pub stake_clauses: BoundedClauses<CollectionId, AttributeKey, AttributeValue>,
	pub fee_clauses: BoundedClauses<CollectionId, AttributeKey, AttributeValue>,
}

impl<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue>
	Contract<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue>
where
	CollectionId: PartialEq,
	ItemId: PartialEq,
	AttributeKey: Encode,
	AttributeValue: Encode + Decode + PartialEq,
{
	pub fn evaluate_stakes<AccountId, NftInspector>(
		&self,
		stakes: &[NftAddress<CollectionId, ItemId>],
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
	{
		(self.stake_clauses.len() == stakes.len())
			.then(|| {
				self.stake_clauses.iter().zip(stakes.iter()).all(|(stake_clause, stake)| {
					stake_clause.evaluate_for::<AccountId, NftInspector, ItemId>(stake)
				})
			})
			.unwrap_or(false)
	}

	pub fn evaluate_fees<AccountId, NftInspector>(
		&self,
		fees: &[NftAddress<CollectionId, ItemId>],
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
	{
		(self.fee_clauses.len() == fees.len())
			.then(|| {
				self.fee_clauses.iter().zip(fees.iter()).all(|(fee_clause, fee)| {
					fee_clause.evaluate_for::<AccountId, NftInspector, ItemId>(fee)
				})
			})
			.unwrap_or(false)
	}
}
