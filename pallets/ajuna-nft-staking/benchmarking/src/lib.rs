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
	BenchmarkHelper as NftStakingBenchmarkHelper, Config as NftStakingConfig, *,
};
use pallet_nfts::{BenchmarkHelper, ItemConfig};
use sp_runtime::{
	traits::{One, UniqueSaturatedFrom, UniqueSaturatedInto},
	DispatchError,
};

// Creator's collections.
const CONTRACT_COLLECTION: u16 = 0;
const REWARD_COLLECTION: u16 = 1;
// Staker's collections.
const STAKE_COLLECTION: u16 = 2;
const FEE_COLLECTION: u16 = 3;
// Sniper's collections.
const SNIPER_STAKE_COLLECTION: u16 = 4;
const SNIPER_FEE_COLLECTION: u16 = 5;
// Unified attribute value for all contracts.
const ATTRIBUTE_VALUE: u64 = 10;

enum Mode {
	Staker,
	Sniper,
}
impl Mode {
	fn collections(self) -> (u16, u16) {
		match self {
			Self::Staker => (STAKE_COLLECTION, FEE_COLLECTION),
			Self::Sniper => (SNIPER_STAKE_COLLECTION, SNIPER_FEE_COLLECTION),
		}
	}
}

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
	<T as NftStakingConfig>::AttributeKey,
	<T as NftStakingConfig>::AttributeValue,
>;

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

fn create_creator<T: Config>(reward_item: Option<Vec<u16>>) -> Result<T::AccountId, DispatchError> {
	let creator = account::<T>("creator");
	create_contract_collection::<T>(&creator)?; // reserve CONTRACT_COLLECTION
	create_collections::<T>(&creator, 1)?; // reserve REWARD_COLLECTION
	if let Some(item_ids) = reward_item {
		item_ids
			.into_iter()
			.try_for_each(|item_id| mint_item::<T>(&creator, REWARD_COLLECTION, item_id))?;
	}
	Creator::<T>::put(&creator);
	Ok(creator)
}

fn create_contract_collection<T: Config>(creator: &T::AccountId) -> DispatchResult {
	create_collection::<T>(creator)?;
	ContractCollectionId::<T>::put(CollectionIdOf::<T>::from(CONTRACT_COLLECTION));
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
		RawOrigin::Signed(creator).into(),
		contract_id,
		contract,
	)
}

fn accept_contract<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	staker: T::AccountId,
	contract_id: ItemIdOf<T>,
	mode: Mode,
) -> DispatchResult {
	let (stakes, fees) = stakes_and_fees::<T>(num_stake_clauses, num_fee_clauses, &staker, mode)?;
	pallet_ajuna_nft_staking::Pallet::<T>::accept(
		RawOrigin::Signed(staker).into(),
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
		owner,
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
	mode: Mode,
) -> Result<(Vec<NftIdOf<T>>, Vec<NftIdOf<T>>), DispatchError> {
	let (stake_collection, fee_collection) = mode.collections();
	let mut stakes = Vec::new();
	let mut fees = Vec::new();
	for i in 0..num_stake_clauses {
		let item_id = i as u16;
		let attr_key = i;
		mint_item::<T>(who, stake_collection, item_id)?;
		set_attribute::<T>(stake_collection, item_id, attr_key, ATTRIBUTE_VALUE)?;
		stakes.push(NftId(
			CollectionIdOf::<T>::unique_saturated_from(stake_collection),
			T::BenchmarkHelper::item_id(item_id),
		));
	}
	for i in num_stake_clauses..num_stake_clauses + num_fee_clauses {
		let item_id = i as u16;
		let attr_key = i;
		mint_item::<T>(who, fee_collection, item_id)?;
		set_attribute::<T>(fee_collection, item_id, attr_key, ATTRIBUTE_VALUE)?;
		fees.push(NftId(
			CollectionIdOf::<T>::unique_saturated_from(fee_collection),
			T::BenchmarkHelper::item_id(item_id),
		));
	}
	Ok((stakes, fees))
}

fn contract_with<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	reward: RewardOf<T>,
	mode: Mode,
) -> ContractOf<T> {
	let (stake_collection, fee_collection) = mode.collections();
	ContractOf::<T> {
		activation: None,
		active_duration: 1_u32.unique_saturated_into(),
		claim_duration: 1_u32.unique_saturated_into(),
		stake_duration: 1_u32.unique_saturated_into(),
		stake_clauses: (0..num_stake_clauses)
			.map(|i| {
				Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(stake_collection),
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
					CollectionIdOf::<T>::unique_saturated_from(fee_collection),
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
		assert_last_event::<T>(Event::CreatorSet { creator })
	}

	set_contract_collection_id {
		let creator = create_creator::<T>(None)?;
		let collection_id = CollectionIdOf::<T>::unique_saturated_from(0_u32);
		ContractCollectionId::<T>::kill();
	}: _(RawOrigin::Signed(creator), collection_id)
	verify {
		assert_last_event::<T>(Event::ContractCollectionSet { collection_id })
	}

	set_global_config {
		let creator = create_creator::<T>(None)?;
		let new_config = GlobalConfig::default();
	}: _(RawOrigin::Signed(creator), new_config)
	verify {
		assert_last_event::<T>(Event::SetGlobalConfig { new_config })
	}

	create_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let creator = create_creator::<T>(None)?;
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract_with::<T>(m, n, reward, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
	}: create(RawOrigin::Signed(creator), contract_id, contract)
	verify {
		assert_last_event::<T>(Event::Created { contract_id })
	}

	create_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 123_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract_with::<T>(m, n, reward, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(Some(vec![reward_nft_item]))?;
	}: create(RawOrigin::Signed(creator), contract_id, contract)
	verify {
		assert_last_event::<T>(Event::Created { contract_id })
	}

	remove_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract_with::<T>(m, n, reward, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;
	}: remove(RawOrigin::Signed(creator), contract_id)
	verify {
		assert_last_event::<T>(Event::Removed { contract_id })
	}

	remove_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract_with::<T>(m, n, reward, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(Some(vec![reward_nft_item]))?;
		create_contract::<T>(creator.clone(), contract_id, contract)?;
	}: remove(RawOrigin::Signed(creator), contract_id)
	verify {
		assert_last_event::<T>(Event::Removed { contract_id })
	}

	accept_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract_with::<T>(m, n, reward, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator, contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		let (stakes, fees) = stakes_and_fees::<T>(m, n, &by, Mode::Staker)?;
	}: accept(RawOrigin::Signed(by.clone()), contract_id, stakes, fees)
	verify {
		assert_last_event::<T>(Event::Accepted { by, contract_id })
	}

	accept_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(
			NftId(REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract_with::<T>(m, n, reward, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(Some(vec![reward_nft_item]))?;
		create_contract::<T>(creator, contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		let (stakes, fees) = stakes_and_fees::<T>(m, n, &by, Mode::Staker)?;
	}: accept(RawOrigin::Signed(by.clone()), contract_id, stakes, fees)
	verify {
		assert_last_event::<T>(Event::Accepted { by, contract_id })
	}

	cancel_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let mut contract = contract_with::<T>(m, n, reward, Mode::Staker);
		contract.stake_duration = 100_u32.unique_saturated_into();
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator, contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)?;
	}: cancel(RawOrigin::Signed(by.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Cancelled { by, contract_id })
	}

	cancel_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let mut contract = contract_with::<T>(m, n, reward, Mode::Staker);
		contract.stake_duration = 100_u32.unique_saturated_into();
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(Some(vec![reward_nft_item]))?;
		create_contract::<T>(creator, contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)?;
	}: cancel(RawOrigin::Signed(by.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Cancelled { by, contract_id })
	}

	claim_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract_with::<T>(m, n, reward.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator, contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)?;
	}: claim(RawOrigin::Signed(by.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Claimed { by, contract_id, reward })
	}

	claim_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract_with::<T>(m, n, reward.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(Some(vec![reward_nft_item]))?;
		create_contract::<T>(creator, contract_id, contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)?;
	}: claim(RawOrigin::Signed(by.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Claimed { by, contract_id, reward })
	}

	snipe_token_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward = Reward::Tokens(123_u64.unique_saturated_into());
		let contract = contract_with::<T>(m, n, reward.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let mut sniper_contract = contract_with::<T>(m, n, reward.clone(), Mode::Sniper);
		sniper_contract.stake_duration = 100_u32.unique_saturated_into();
		let sniper_contract_id = T::BenchmarkHelper::item_id(1_u16);

		let creator = create_creator::<T>(None)?;
		create_contract::<T>(creator.clone(), contract_id, contract.clone())?;
		create_contract::<T>(creator, sniper_contract_id, sniper_contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by, contract_id, Mode::Staker)?;

		let sniper = account::<T>("sniper");
		create_collections::<T>(&sniper, 2)?;
		accept_contract::<T>(m, n, sniper.clone(), sniper_contract_id, Mode::Sniper)?;

		// Advance block past contract expiry.
		frame_system::Pallet::<T>::set_block_number(
			contract.stake_duration + contract.claim_duration + One::one()
		);
	}: snipe(RawOrigin::Signed(sniper.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Sniped { by: sniper, contract_id, reward })
	}

	snipe_nft_reward {
		let m in 0..T::MaxStakingClauses::get();
		let n in 0..T::MaxFeeClauses::get();
		let reward_nft_item = 2_u16;
		let reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		));
		let contract = contract_with::<T>(m, n, reward.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let sniper_reward_nft_item = 123_u16;
		let sniper_reward = Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(sniper_reward_nft_item),
		));
		let mut sniper_contract = contract_with::<T>(m, n, sniper_reward, Mode::Sniper);
		sniper_contract.stake_duration = 100_u32.unique_saturated_into();
		let sniper_contract_id = T::BenchmarkHelper::item_id(1_u16);

		let creator = create_creator::<T>(Some(vec![reward_nft_item, sniper_reward_nft_item]))?;
		create_contract::<T>(creator.clone(), contract_id, contract.clone())?;
		create_contract::<T>(creator, sniper_contract_id, sniper_contract)?;

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2)?;
		accept_contract::<T>(m, n, by, contract_id, Mode::Staker)?;

		let sniper = account::<T>("sniper");
		create_collections::<T>(&sniper, 2)?;
		accept_contract::<T>(m, n, sniper.clone(), sniper_contract_id, Mode::Sniper)?;

		// Advance block past contract expiry.
		frame_system::Pallet::<T>::set_block_number(
			contract.stake_duration + contract.claim_duration + One::one()
		);
	}: snipe(RawOrigin::Signed(sniper.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Sniped { by: sniper, contract_id, reward })
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
