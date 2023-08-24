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
pub struct ContractClause<CollectionId, AttributeKey, AttributeValue> {
	pub namespace: AttributeNamespace,
	pub clause: Clause<CollectionId, AttributeKey, AttributeValue>,
}

impl<CollectionId, AttributeKey, AttributeValue>
	ContractClause<CollectionId, AttributeKey, AttributeValue>
{
	pub fn evaluate_for<AccountId, NftInspector, ItemId>(
		&self,
		address: &NftId<CollectionId, ItemId>,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		CollectionId: PartialEq,
		ItemId: PartialEq,
		AttributeKey: Encode,
		AttributeValue: Encode + Decode + PartialEq,
	{
		let evaluate_fn = match self.namespace {
			AttributeNamespace::Pallet => NftInspector::system_attribute,
			AttributeNamespace::CollectionOwner => NftInspector::attribute,
		};

		self.clause
			.evaluate_for::<AccountId, NftInspector, ItemId, _>(address, evaluate_fn)
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Clause<CollectionId, AttributeKey, AttributeValue> {
	HasAttribute(CollectionId, AttributeKey),
	HasAttributeWithValue(CollectionId, AttributeKey, AttributeValue),
}

impl<CollectionId, AttributeKey, AttributeValue>
	Clause<CollectionId, AttributeKey, AttributeValue>
{
	pub fn evaluate_for<AccountId, NftInspector, ItemId, Fn>(
		&self,
		address: &NftId<CollectionId, ItemId>,
		evaluate_fn: Fn,
	) -> bool
	where
		NftInspector: Inspect<AccountId, CollectionId = CollectionId, ItemId = ItemId>,
		CollectionId: PartialEq,
		ItemId: PartialEq,
		AttributeKey: Encode,
		AttributeValue: Encode + Decode + PartialEq,
		Fn: FnOnce(&CollectionId, &ItemId, &[u8]) -> Option<Vec<u8>>,
	{
		let clause_collection_id = match self {
			Clause::HasAttribute(collection_id, _) => collection_id,
			Clause::HasAttributeWithValue(collection_id, _, _) => collection_id,
		};
		let NftId(collection_id, item_id) = address;
		clause_collection_id == collection_id &&
			(match self {
				Clause::HasAttribute(_, key) =>
					evaluate_fn(collection_id, item_id, &key.encode()).is_some(),
				Clause::HasAttributeWithValue(_, key, expected_value) =>
					if let Some(value) = evaluate_fn(collection_id, item_id, &key.encode()) {
						expected_value.eq(&AttributeValue::decode(&mut value.as_slice()).unwrap())
					} else {
						false
					},
			})
	}
}

type BoundedClauses<CollectionId, AttributeKey, AttributeValue> =
	BoundedVec<ContractClause<CollectionId, AttributeKey, AttributeValue>, ConstU32<100>>;

/// Specification for a staking contract, in short it's a list of criteria to be fulfilled,
/// with a given reward after the duration is complete.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Contract<Balance, CollectionId, ItemId, BlockNumber, AttributeKey, AttributeValue> {
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
	pub stake_clauses: BoundedClauses<CollectionId, AttributeKey, AttributeValue>,
	/// The list of conditions to satisfy as fee NFTs. A staker must provide NFTs that meet these
	/// requirements to accept the contract, which is transferred to the contract creator.
	pub fee_clauses: BoundedClauses<CollectionId, AttributeKey, AttributeValue>,

	/// The reward of fulfilling the given contract in the form of either tokens or NFTs.
	pub reward: Reward<Balance, CollectionId, ItemId>,
	/// The fee required to cancel the given contract. Any staked NFTs for the contract will be
	/// immediately returned to the staker upon cancellation.
	pub cancel_fee: Balance,
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
		stakes: &[NftId<CollectionId, ItemId>],
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
		fees: &[NftId<CollectionId, ItemId>],
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
