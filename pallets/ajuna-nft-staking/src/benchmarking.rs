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

use super::*;

#[allow(unused)]
use crate::Pallet as NftStake;
use frame_benchmarking::{benchmarks, whitelist_account};
use frame_support::traits::tokens::nonfungibles_v2::{Create, Mutate};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

pub fn prepare_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = get_account::<T>(name);
	whitelist_account!(account);

	// Give the account enough tokens
	T::Currency::make_free_balance_be(
		&account,
		BalanceOf::<T>::max_value().saturating_sub(1_000_000_u32.into()),
	);
	account
}

fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	frame_benchmarking::account(name, 0, 0)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn create_staking_contract<T: Config>(
	reward: StakingRewardOf<T>,
	duration: BlockNumberOf<T>,
	clause: ContractClause<ContractAttributeKeyOf<T>, ContractAttributeValueOf<T>>,
) -> StakingContractOf<T> {
	StakingContractOf::<T>::new(reward, duration)
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause.clone())
		.with_clause(clause)
}

fn create_random_nft_collection<T: Config>(account: AccountIdOf<T>) -> CollectionIdOf<T> {
	let collection_config = T::ContractCollectionConfig::get();
	T::NftHelper::create_collection(&account, &account, &collection_config)
		.expect("Should have create contract collection")
}

fn create_random_nft_batch<T: Config>(
	owner: &AccountIdOf<T>,
	collection_id: CollectionIdOf<T>,
	amount: u32,
) -> Vec<NftAddressOf<T>> {
	let mut nft_vec = Vec::with_capacity(amount as usize);

	for item_id in 0..amount {
		nft_vec.push(create_random_nft::<T>(owner, collection_id, item_id.into()));
	}

	nft_vec
}

fn create_random_nft<T: Config>(
	owner: &AccountIdOf<T>,
	collection_id: CollectionIdOf<T>,
	item_id: ItemIdOf<T>,
) -> NftAddressOf<T> {
	let item_config = T::ContractCollectionItemConfig::get();
	T::NftHelper::mint_into(&collection_id, &item_id, owner, &item_config, true)
		.expect("Should create Nft");

	NftAddress(collection_id, item_id)
}

fn create_staking_vector_from<T: Config>(
	mut nft_vec: Vec<NftAddressOf<T>>,
) -> StakedAssetsVecOf<T> {
	let mut vec = StakedAssetsVecOf::<T>::with_max_capacity();

	for _ in 0..vec.capacity() {
		if let Some(item) = nft_vec.pop() {
			vec.force_push(item);
		} else {
			break
		}
	}

	vec
}

fn set_attribute_for_nft_batch<T: Config>(
	nft_batch: &[NftAddressOf<T>],
	nft_attr_key: u32,
	nft_attr_value: u64,
) {
	for item in nft_batch.iter() {
		set_attribute_for_nft::<T>(item, nft_attr_key, nft_attr_value);
	}
}

fn set_attribute_for_nft<T: Config>(
	nft_addr: &NftAddressOf<T>,
	nft_attr_key: u32,
	nft_attr_value: u64,
) {
	T::NftHelper::set_typed_attribute::<u32, u64>(
		&nft_addr.0,
		&nft_addr.1,
		&nft_attr_key,
		&nft_attr_value,
	)
	.expect("Should add attribute Nft");
}

fn create_contract_clause<T: Config>(attr_key: u32, attr_value: u64) -> ContractClauseOf<T> {
	ContractClauseOf::<T>::HasAttributeWithValue(attr_key.into(), attr_value.into())
}

fn create_staking_contract_collection<T: Config>(account: &T::AccountId) -> T::CollectionId {
	let collection_config = <T as crate::pallet::Config>::ContractCollectionConfig::get();
	<T as crate::pallet::Config>::NftHelper::create_collection(account, account, &collection_config)
		.expect("Should have create contract collection")
}

type ContractClauseOf<T> =
	ContractClause<<T as Config>::ContractAttributeKey, <T as Config>::ContractAttributeValue>;

benchmarks! {
	set_creator {
		let creator = prepare_account::<T>("ALICE");
	}: _(RawOrigin::Root, creator.clone())
	verify {
		assert_last_event::<T>(Event::CreatorSet { creator }.into())
	}

	set_contract_collection_id {
		let account = NftStake::<T>::treasury_account_id();
		let collection_id = create_staking_contract_collection::<T>(&account);
		Creator::<T>::put(&account);
	}: _(RawOrigin::Signed(account), collection_id)
	verify {
		assert_last_event::<T>(Event::ContractCollectionSet { collection_id }.into())
	}

	set_locked_state {
		let creator = prepare_account::<T>("ALICE");
		Creator::<T>::put(&creator);
	}: _(RawOrigin::Signed(creator), PalletLockedState::Locked)
	verify {
		assert_last_event::<T>(Event::LockedStateSet { locked_state: PalletLockedState::Locked }.into())
	}

	create_token_reward {
		let account = NftStake::<T>::treasury_account_id();
		let collection_id = create_staking_contract_collection::<T>(&account);
		ContractCollectionId::<T>::put(collection_id);

		let caller = prepare_account::<T>("ALICE");
		let reward_amt: BalanceOf<T> = 1_000_u32.into();
		let reward = StakingRewardOf::<T>::Tokens(reward_amt);
		let clause = create_contract_clause::<T>(10, 10);
		let contract = create_staking_contract::<T>(reward, 10_u32.into(), clause);
		let expected_id = NextContractId::<T>::get();
	}: create(RawOrigin::Signed(caller.clone()), contract)
	verify {
		assert_last_event::<T>(Event::Created { creator: caller, contract_id: expected_id }.into())
	}

	create_nft_reward {
		let account = NftStake::<T>::treasury_account_id();
		let collection_id = create_staking_contract_collection::<T>(&account);
		ContractCollectionId::<T>::put(collection_id);

		let caller = prepare_account::<T>("ALICE");
		let collection_id = create_random_nft_collection::<T>(caller.clone());
		let nft_addr = create_random_nft::<T>(&caller, collection_id, 0_u32.into());
		let reward = StakingRewardOf::<T>::Nft(nft_addr);
		let clause = create_contract_clause::<T>(10, 10);
		let contract = create_staking_contract::<T>(reward, 10_u32.into(), clause);
		let expected_id = NextContractId::<T>::get();
	}: create(RawOrigin::Signed(caller.clone()), contract)
	verify {
		assert_last_event::<T>(Event::Created { creator: caller, contract_id: expected_id }.into())
	}

	accept {
		let account = NftStake::<T>::treasury_account_id();
		let collection_id = create_staking_contract_collection::<T>(&account);
		ContractCollectionId::<T>::put(collection_id);

		let creator = prepare_account::<T>("ALICE");
		let reward_amt: BalanceOf<T> = 1_000_u32.into();
		let reward = StakingRewardOf::<T>::Tokens(reward_amt);
		let clause = create_contract_clause::<T>(10, 10);
		let contract = create_staking_contract::<T>(reward, 10_u32.into(), clause);
		let contract_id = NextContractId::<T>::get();

		NftStake::<T>::create(RawOrigin::Signed(creator).into(), contract)?;

		let caller = prepare_account::<T>("BOB");
		let collection_id = create_random_nft_collection::<T>(caller.clone());
		let nft_batch = create_random_nft_batch::<T>(&caller, collection_id, MAXIMUM_CLAUSES_PER_CONTRACT);
		set_attribute_for_nft_batch::<T>(&nft_batch, 10_u32, 10_u64);
		let staking_vec = create_staking_vector_from::<T>(nft_batch);
	}: _(RawOrigin::Signed(caller.clone()), contract_id, staking_vec)
	verify {
		assert_last_event::<T>(Event::Accepted { accepted_by: caller, contract_id }.into())
	}

	claim_token_reward {
		let account = NftStake::<T>::treasury_account_id();
		let collection_id = create_staking_contract_collection::<T>(&account);
		ContractCollectionId::<T>::put(collection_id);

		let creator = prepare_account::<T>("ALICE");
		let reward_amt: BalanceOf<T> = 1_000_u32.into();
		let reward = StakingRewardOf::<T>::Tokens(reward_amt);
		let clause = create_contract_clause::<T>(10, 10);
		let contract = create_staking_contract::<T>(reward.clone(), 0_u32.into(), clause);
		let contract_id = NextContractId::<T>::get();

		NftStake::<T>::create(RawOrigin::Signed(creator).into(), contract)?;

		let caller = prepare_account::<T>("BOB");
		let collection_id = create_random_nft_collection::<T>(caller.clone());
		let nft_batch = create_random_nft_batch::<T>(&caller, collection_id, MAXIMUM_CLAUSES_PER_CONTRACT);
		set_attribute_for_nft_batch::<T>(&nft_batch, 10_u32, 10_u64);
		let staking_vec = create_staking_vector_from::<T>(nft_batch);

		NftStake::<T>::accept(RawOrigin::Signed(caller.clone()).into(), contract_id, staking_vec)?;

	}: claim(RawOrigin::Signed(caller.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Claimed { claimed_by: caller, contract_id, reward }.into())
	}

	claim_nft_reward {
		let account = NftStake::<T>::treasury_account_id();
		let collection_id = create_staking_contract_collection::<T>(&account);
		ContractCollectionId::<T>::put(collection_id);

		let creator = prepare_account::<T>("ALICE");
		let collection_id = create_random_nft_collection::<T>(creator.clone());
		let reward_nft_addr = create_random_nft::<T>(&creator, collection_id, 0_u32.into());
		let reward = StakingRewardOf::<T>::Nft(reward_nft_addr);
		let clause = create_contract_clause::<T>(10, 10);
		let contract = create_staking_contract::<T>(reward.clone(), 0_u32.into(), clause);
		let contract_id = NextContractId::<T>::get();

		NftStake::<T>::create(RawOrigin::Signed(creator).into(), contract)?;

		let caller = prepare_account::<T>("BOB");
		let collection_id = create_random_nft_collection::<T>(caller.clone());
		let nft_batch = create_random_nft_batch::<T>(&caller, collection_id, MAXIMUM_CLAUSES_PER_CONTRACT);
		set_attribute_for_nft_batch::<T>(&nft_batch, 10_u32, 10_u64);
		let staking_vec = create_staking_vector_from::<T>(nft_batch);

		NftStake::<T>::accept(RawOrigin::Signed(caller.clone()).into(), contract_id, staking_vec)?;

	}: claim(RawOrigin::Signed(caller.clone()), contract_id)
	verify {
		assert_last_event::<T>(Event::Claimed { claimed_by: caller, contract_id, reward }.into())
	}

	impl_benchmark_test_suite!(
		NftStake, crate::mock::ExtBuilder::default().create_collection(true).build(), crate::mock::Test
	);
}
