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
	pallet_prelude::DispatchResult,
	traits::{
		tokens::nonfungibles_v2::{Create, Mutate},
		Currency, Get,
	},
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use pallet_ajuna_nft_staking::{
	BenchmarkHelper as NftStakingBenchHelper, Clause, Config as NftStakingConfig,
	ContractCollectionId, Reward, *,
};
use pallet_nfts::{BenchmarkHelper as NftBenchHelper, ItemConfig};
use sp_runtime::traits::{UniqueSaturatedFrom, UniqueSaturatedInto, Zero};

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
	CurrencyOf::<T>::deposit_creating(&account, u64::MAX.unique_saturated_into());
	account
}

fn assert_last_event<T: Config>(avatars_event: Event<T>) {
	let event = <T as NftStakingConfig>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

fn create_creator<T: Config>() -> T::AccountId {
	let creator = account::<T>("creator");
	Creator::<T>::put(&creator);
	creator
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

fn contract<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	reward: RewardOf<T>,
) -> ContractOf<T> {
	ContractOf::<T> {
		activation: Some(BlockNumberFor::<T>::unique_saturated_from(u64::MAX)),
		active_duration: BlockNumberFor::<T>::unique_saturated_from(u64::MAX),
		claim_duration: BlockNumberFor::<T>::unique_saturated_from(u64::MAX),
		stake_duration: BlockNumberFor::<T>::unique_saturated_from(u64::MAX),
		stake_clauses: (0..num_stake_clauses)
			.map(|i| {
				Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(1_u32),
					T::BenchmarkHelper::contract_key(i),
					T::BenchmarkHelper::contract_value(u64::MAX),
				)
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		fee_clauses: (num_stake_clauses..num_stake_clauses + num_fee_clauses)
			.map(|i| {
				Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(2_u32),
					T::BenchmarkHelper::contract_key(i),
					T::BenchmarkHelper::contract_value(u64::MAX),
				)
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		reward,
		cancel_fee: BalanceOf::<T>::unique_saturated_from(u64::MAX),
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
		let creator = create_creator::<T>();
		let collection_id = CollectionIdOf::<T>::unique_saturated_from(0_u32);
		create_collection::<T>(&creator)?;
		ContractCollectionId::<T>::kill();
	}: _(RawOrigin::Signed(creator), collection_id.clone())
	verify {
		assert_last_event::<T>(Event::ContractCollectionSet { collection_id }.into())
	}

	set_global_config {
		let creator = create_creator::<T>();
		let new_config = GlobalConfig::default();
	}: _(RawOrigin::Signed(creator), new_config)
	verify {
		assert_last_event::<T>(Event::SetGlobalConfig { new_config }.into())
	}

	create_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();

		let amount = u64::MAX.checked_div((m * n) as u64).unwrap_or(Zero::zero());
		let reward = Reward::Tokens(amount.unique_saturated_into());
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>();
		create_contract_collection::<T>(&creator)?;
	}: create(RawOrigin::Signed(creator), contract_id, contract)
	verify {
		assert_last_event::<T>(Event::Created { contract_id }.into())
	}

	create_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();

		let reward_nft_collection = 1_u16;
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			(reward_nft_collection as u32).unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>();
		create_contract_collection::<T>(&creator)?;
		create_collections::<T>(&creator, 1)?;
		mint_item::<T>(&creator, reward_nft_collection, reward_nft_item)?;
	}: create(RawOrigin::Signed(creator), contract_id, contract)
	verify {
		assert_last_event::<T>(Event::Created { contract_id }.into())
	}

	remove_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();

		let amount = u64::MAX.checked_div((m * n) as u64).unwrap_or(Zero::zero());
		let reward = Reward::Tokens(amount.unique_saturated_into());
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>();
		create_contract_collection::<T>(&creator)?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;
	}: remove(RawOrigin::Signed(creator), contract_id)
	verify {
		assert_last_event::<T>(Event::Removed { contract_id }.into())
	}

	remove_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();

		let reward_nft_collection = 1_u16;
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			(reward_nft_collection as u32).unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract::<T>(m, n, reward);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>();
		create_contract_collection::<T>(&creator)?;
		create_collections::<T>(&creator, 1)?;
		mint_item::<T>(&creator, reward_nft_collection, reward_nft_item)?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;
	}: remove(RawOrigin::Signed(creator), contract_id)
	verify {
		assert_last_event::<T>(Event::Removed { contract_id }.into())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
