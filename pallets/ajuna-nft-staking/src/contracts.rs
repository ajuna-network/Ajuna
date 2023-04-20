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
	tokens::{nonfungibles_v2::Inspect, Balance as BalanceT},
	ConstU32,
};
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
pub enum Clause<CollectionId, ItemId, AttributeKey, AttributeValue> {
	HasAttribute(NftAddress<CollectionId, ItemId>, AttributeKey),
	HasAttributeWithValue(NftAddress<CollectionId, ItemId>, AttributeKey, AttributeValue),
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
	pub staking_clauses:
		BoundedVec<Clause<CollectionId, ItemId, AttributeKey, AttributeValue>, ConstU32<N>>,
	pub duration: BlockNumber,
}

impl<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue, const N: u32>
	Contract<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue, N>
where
	Balance: BalanceT,
	CollectionId: Debug + Copy + PartialEq,
	ItemId: Debug + Copy + PartialEq,
	BlockNumber: Debug + Copy,
	AttributeKey: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
	AttributeValue: Debug + Clone + Encode + Decode + Eq + PartialEq + Ord + PartialOrd,
{
	pub fn new(
		reward: Reward<Balance, CollectionId, ItemId>,
		duration: BlockNumber,
		staking_clauses: BoundedVec<
			Clause<CollectionId, ItemId, AttributeKey, AttributeValue>,
			ConstU32<N>,
		>,
	) -> Self {
		Self { reward, duration, staking_clauses }
	}

	pub fn evaluate_for<AccountId, NftInspector>(
		&self,
		stakes: &[NftAddress<CollectionId, ItemId>],
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
	{
		(self.staking_clauses.len() == stakes.len())
			.then(|| {
				self.staking_clauses.iter().zip(stakes.iter()).all(|(staking_clause, stake)| {
					let staking = match staking_clause {
						Clause::HasAttribute(staking, _) => staking,
						Clause::HasAttributeWithValue(staking, _, _) => staking,
					};
					staking == stake &&
						match staking_clause {
							Clause::HasAttribute(_, key) =>
								NftInspector::system_attribute(&stake.0, &stake.1, &key.encode())
									.is_some(),
							Clause::HasAttributeWithValue(_, key, expected_value) =>
								if let Some(value) = NftInspector::system_attribute(
									&stake.0,
									&stake.1,
									&key.encode(),
								) {
									expected_value
										.eq(&AttributeValue::decode(&mut value.as_slice()).unwrap())
								} else {
									false
								},
						}
				})
			})
			.unwrap_or(false)
	}
}
