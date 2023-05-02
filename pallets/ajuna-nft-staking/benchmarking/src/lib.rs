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

#![cfg(feature = "runtime-benchmarks")]
#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::benchmarks;
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::nonfungibles_v2::{Create, Mutate},
		Currency, Get,
	},
};
use frame_system::RawOrigin;
use pallet_ajuna_nft_staking::{
	BenchmarkHelper as NftStakingBenchHelper, Clause, Config as NftStakingConfig,
	ContractCollectionId, Reward, *,
};
use pallet_nfts::{BenchmarkHelper as NftBenchHelper, ItemConfig};
use sp_runtime::{
	traits::{UniqueSaturatedFrom, UniqueSaturatedInto},
	DispatchError,
};

const CONTRACT_COLLECTION: u16 = 0;
const REWARD_COLLECTION: u16 = 1;
const STAKE_COLLECTION: u16 = 2;
const FEE_COLLECTION: u16 = 3;
const ATTRIBUTE_VALUE: u64 = 4;

pub struct Pallet<T: Config>(pallet_ajuna_nft_staking::Pallet<T>);
pub trait Config: NftStakingConfig + pallet_nfts::Config + pallet_balances::Config {}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

type CurrencyOf<T> = <T as NftStakingConfig>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;
type CollectionIdOf<T> = <T as NftStakingConfig>::CollectionId;
type ItemIdOf<T> = <T as NftStakingConfig>::ItemId;
type ContractOf<T> = Contract<
	BalanceOf<T>,
	CollectionIdOf<T>,
	<T as NftStakingConfig>::ItemId,
	<T as frame_system::Config>::BlockNumber,
	<T as NftStakingConfig>::ContractAttributeKey,
	<T as NftStakingConfig>::ContractAttributeValue,
>;
type RewardOf<T> = Reward<BalanceOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;

type NftCurrencyOf<T> = <T as pallet_nfts::Config>::Currency;
type NftBalanceOf<T> = <NftCurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;
type NftCollectionIdOf<T> = <T as pallet_nfts::Config>::CollectionId;
type CollectionDeposit<T> = <T as pallet_nfts::Config>::CollectionDeposit;
type ItemDeposit<T> = <T as pallet_nfts::Config>::ItemDeposit;
type CollectionConfigOf<T> =
	pallet_nfts::CollectionConfig<NftBalanceOf<T>, BlockNumberOf<T>, NftCollectionIdOf<T>>;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let account = frame_benchmarking::account(name, Default::default(), Default::default());
	CurrencyOf::<T>::make_free_balance_be(&account, 999_999_999_u64.unique_saturated_into());
	account
}

fn assert_last_event<T: Config>(avatars_event: Event<T>) {
	let event = <T as NftStakingConfig>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

fn create_creator<T: Config>(reward_item: Option<u16>) -> Result<T::AccountId, DispatchError> {
	let creator = account::<T>("creator");
	create_contract_collection::<T>(&creator)?; // reserve CONTRACT_COLLECTION
	create_collections::<T>(&creator, 1)?; // reserve REWARD_COLLECTION
	if let Some(item_id) = reward_item {
		mint_item::<T>(&creator, REWARD_COLLECTION, item_id)?;
	}
	Creator::<T>::put(&creator);
	Ok(creator)
}

fn create_contract_collection<T: Config>(creator: &T::AccountId) -> DispatchResult {
	create_collection::<T>(creator)?;
	ContractCollectionId::<T>::put(CollectionIdOf::<T>::from(0_u32));
	Ok(())
}

fn create_collections<T: Config>(creator: &T::AccountId, n: usize) -> DispatchResult {
	(0..n).try_for_each(|_| create_collection::<T>(creator))
}

fn create_collection<T: Config>(owner: &T::AccountId) -> DispatchResult {
	NftCurrencyOf::<T>::deposit_creating(owner, CollectionDeposit::<T>::get());
	<pallet_nfts::Pallet<T> as Create<T::AccountId, CollectionConfigOf<T>>>::create_collection(
		owner,
		owner,
		&pallet_nfts::CollectionConfig {
			settings: Default::default(),
			max_supply: Default::default(),
			mint_settings: Default::default(),
		},
	)?;
	Ok(())
}

fn create_contract<T: Config>(
	creator: T::AccountId,
	contract_id: ItemIdOf<T>,
	contract: ContractOf<T>,
) -> DispatchResult {
	pallet_ajuna_nft_staking::Pallet::<T>::create(
		RawOrigin::Signed(creator.clone()).into(),
		contract_id,
		contract,
	)
}

fn accept_contract<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	staker: T::AccountId,
	contract_id: ItemIdOf<T>,
) -> DispatchResult {
	let (stakes, fees) = stakes_and_fees::<T>(num_stake_clauses, num_fee_clauses, &staker)?;
	pallet_ajuna_nft_staking::Pallet::<T>::accept(
		RawOrigin::Signed(staker.clone()).into(),
		contract_id,
		stakes,
		fees,
	)
}

fn mint_item<T: Config>(owner: &T::AccountId, collection_id: u16, item_id: u16) -> DispatchResult {
	let collection_id = &T::Helper::collection(collection_id);
	let item_id = &T::Helper::item(item_id);
	NftCurrencyOf::<T>::deposit_creating(owner, ItemDeposit::<T>::get());
	<pallet_nfts::Pallet<T> as Mutate<T::AccountId, ItemConfig>>::mint_into(
		collection_id,
		item_id,
		&owner,
		&ItemConfig::default(),
		false,
	)?;
	Ok(())
}

fn set_attribute<T: Config>(
	collection_id: u16,
	item_id: u16,
	key: u32,
	value: u64,
) -> DispatchResult {
	let collection_id = &T::Helper::collection(collection_id);
	let item_id = &T::Helper::item(item_id);
	<pallet_nfts::Pallet<T> as Mutate<T::AccountId, ItemConfig>>::set_typed_attribute(
		collection_id,
		item_id,
		&key,
		&value,
	)?;
	Ok(())
}

fn stakes_and_fees<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	who: &T::AccountId,
) -> Result<(Vec<NftIdOf<T>>, Vec<NftIdOf<T>>), DispatchError> {
	let mut stakes = Vec::new();
	let mut fees = Vec::new();

	for i in 0..num_stake_clauses {
		let item_id = i as u16;
		let attr_key = i;
		mint_item::<T>(who, STAKE_COLLECTION, item_id)?;
		set_attribute::<T>(STAKE_COLLECTION, item_id, attr_key, ATTRIBUTE_VALUE)?;
		stakes.push(NftId(
			CollectionIdOf::<T>::unique_saturated_from(STAKE_COLLECTION),
			T::BenchmarkHelper::item_id(item_id),
		));
	}
	for i in num_stake_clauses..num_stake_clauses + num_fee_clauses {
		let item_id = i as u16;
		let attr_key = i;
		mint_item::<T>(who, FEE_COLLECTION, item_id)?;
		set_attribute::<T>(FEE_COLLECTION, item_id, attr_key, ATTRIBUTE_VALUE)?;
		fees.push(NftId(
			CollectionIdOf::<T>::unique_saturated_from(FEE_COLLECTION),
			T::BenchmarkHelper::item_id(item_id),
		));
	}
	Ok((stakes, fees))
}

fn contract<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	reward: RewardOf<T>,
) -> ContractOf<T> {
	ContractOf::<T> {
		activation: None,
		active_duration: 1_u32.unique_saturated_into(),
		claim_duration: 1_u32.unique_saturated_into(),
		stake_duration: 1_u32.unique_saturated_into(),
		stake_clauses: (0..num_stake_clauses)
			.map(|i| {
				Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(STAKE_COLLECTION),
					T::BenchmarkHelper::contract_key(i),
					T::BenchmarkHelper::contract_value(ATTRIBUTE_VALUE),
				)
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		fee_clauses: (num_stake_clauses..num_stake_clauses + num_fee_clauses)
			.map(|i| {
				Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(FEE_COLLECTION),
					T::BenchmarkHelper::contract_key(i),
					T::BenchmarkHelper::contract_value(ATTRIBUTE_VALUE),
				)
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		reward,
		cancel_fee: 333_u64.unique_saturated_into(),
	}
}

benchmarks! {
	set_creator {
		let creator = account::<T>("creator");
	}: _(RawOrigin::Root, creator.clone())
	verify {
		assert_last_event::<T>(Event::CreatorSet { creator }.into())
	}

	set_contract_collection_id {
		let creator = create_creator::<T>(None)?;
		let collection_id = CollectionIdOf::<T>::unique_saturated_from(0_u32);
		ContractCollectionId::<T>::kill();
	}: _(RawOrigin::Signed(creator), collection_id.clone())
	verify {
		assert_last_event::<T>(Event::ContractCollectionSet { collection_id }.into())
	}

	set_global_config {
		let creator = create_creator::<T>(None)?;
		let new_config = GlobalConfig::default();
	}: _(RawOrigin::Signed(creator), new_config)
	verify {
		assert_last_event::<T>(Event::SetGlobalConfig { new_config }.into())
	}

	create_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let creator = create_creator::<T>(None)?;
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
	}: create(RawOrigin::Signed(creator), contract_id, contract)
	verify {
		assert_last_event::<T>(Event::Created { contract_id }.into())
	}

	create_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 123_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(Some(reward_nft_item))?;
	}: create(RawOrigin::Signed(creator), contract_id, contract)
	verify {
		assert_last_event::<T>(Event::Created { contract_id }.into())
	}

	remove_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;
	}: remove(RawOrigin::Signed(creator), contract_id)
	verify {
		assert_last_event::<T>(Event::Removed { contract_id }.into())
	}

	remove_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(Some(reward_nft_item))?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;
	}: remove(RawOrigin::Signed(creator), contract_id)
	verify {
		assert_last_event::<T>(Event::Removed { contract_id }.into())
	}

	accept_token_reward {
		let m = T::MaxStakingClauses::get();
		let n = T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		let (stakes, fees) = stakes_and_fees::<T>(m, n, &by)?;
	}: accept(RawOrigin::Signed(by.clone()), contract_id, stakes, fees)
	verify {
		assert_last_event::<T>(Event::Accepted { by, contract_id }.into())
	}

	accept_nft_reward {
		let m = T::MaxStakingClauses::get();
		let n = T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(
			NftId(REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(Some(reward_nft_item))?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		let (stakes, fees) = stakes_and_fees::<T>(m, n, &by)?;
	}: accept(RawOrigin::Signed(by.clone()), contract_id, stakes, fees)
	verify {
		assert_last_event::<T>(Event::Accepted { by, contract_id }.into())
	}

	cancel_token_reward {
		let m = T::MaxStakingClauses::get();
		let n = T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let mut contract = contract::<T>(m, n, reward);
		contract.stake_duration = 100_u32.unique_saturated_into();
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by.clone(), contract_id.clone())?;
	}: cancel(RawOrigin::Signed(by.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Cancelled { by, contract_id }.into())
	}

	cancel_nft_reward {
		let m = T::MaxStakingClauses::get();
		let n = T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let mut contract = contract::<T>(m, n, reward);
		contract.stake_duration = 100_u32.unique_saturated_into();
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(Some(reward_nft_item))?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by.clone(), contract_id.clone())?;
	}: cancel(RawOrigin::Signed(by.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Cancelled { by, contract_id }.into())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
