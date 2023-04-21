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

use crate::{mock::*, *};
use frame_support::{
	assert_noop, assert_ok,
	traits::tokens::nonfungibles_v2::{Create, Mutate},
};
use sp_runtime::bounded_vec;

mod set_creator {
	use super::*;

	#[test]
	fn works() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Creator::<Test>::get(), None);
			assert_ok!(NftStake::set_creator(RuntimeOrigin::root(), ALICE));
			assert_eq!(Creator::<Test>::get(), Some(ALICE));
			System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::CreatorSet {
				creator: ALICE,
			}));
		});
	}

	#[test]
	fn rejects_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				NftStake::set_creator(RuntimeOrigin::signed(BOB), ALICE),
				DispatchError::BadOrigin
			);
		});
	}
}

mod set_contract_collection_id {
	use super::*;

	#[test]
	fn works() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			assert_eq!(ContractCollectionId::<Test>::get(), None);
			assert_ok!(NftStake::set_contract_collection_id(
				RuntimeOrigin::signed(ALICE),
				collection_id
			));
			assert_eq!(ContractCollectionId::<Test>::get(), Some(collection_id));
			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::ContractCollectionSet { collection_id },
			));
		});
	}

	#[test]
	fn rejects_non_existing_collection() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			assert_noop!(
				NftStake::set_contract_collection_id(RuntimeOrigin::signed(ALICE), 17),
				Error::<Test>::UnknownCollection
			);
		});
	}

	#[test]
	fn rejects_non_creator_owned_collection() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let collection_id = create_collection(BOB);
			assert_noop!(
				NftStake::set_contract_collection_id(RuntimeOrigin::signed(ALICE), collection_id),
				Error::<Test>::Ownership
			);
		});
	}
}

mod set_global_config {
	use super::*;

	#[test]
	fn works() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let new_config = GlobalConfig { pallet_locked: true };
			assert_ok!(NftStake::set_global_config(RuntimeOrigin::signed(ALICE), new_config));
			assert_eq!(GlobalConfigs::<Test>::get(), new_config);
			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::SetGlobalConfig { new_config },
			));
		});
	}

	#[test]
	fn rejects_non_creator_calls() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			assert_noop!(
				NftStake::set_global_config(RuntimeOrigin::signed(BOB), GlobalConfig::default()),
				DispatchError::BadOrigin
			);
		});
	}
}

mod create {
	use super::*;

	#[test]
	fn works_with_token_reward() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.balances(vec![(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let reward_amount = 1_000;
				let reward = Reward::Tokens(reward_amount);
				let contract = Contract::new(reward, 10, Default::default());
				let base_reserves = CurrencyOf::<Test>::free_balance(NftStake::account_id());

				let contract_id = NextContractId::<Test>::get();
				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

				assert_ok!(NftStake::create(RuntimeOrigin::signed(ALICE), contract.clone()));
				assert_eq!(Contracts::<Test>::get(contract_id), Some(contract));
				assert_eq!(
					Nft::owner(contract_collection_id, contract_id),
					Some(NftStake::account_id())
				);
				assert_eq!(
					Balances::free_balance(ALICE),
					initial_balance - reward_amount - ItemDeposit::get()
				);
				assert_eq!(NftStake::account_balance(), base_reserves + reward_amount);
				assert_eq!(NextContractId::<Test>::get(), contract_id + 1);

				System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Created {
					creator: ALICE,
					contract_id,
				}));
			});
	}

	#[test]
	fn works_with_nft_reward() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.balances(vec![(ALICE, ItemDeposit::get())])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let nft_addr = mint_item(ALICE, collection_id, 1);
				let reward = Reward::Nft(nft_addr.clone());
				let contract = Contract::new(reward, 10, Default::default());

				let contract_id = NextContractId::<Test>::get();
				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

				assert_ok!(NftStake::create(RuntimeOrigin::signed(ALICE), contract.clone()));
				assert_eq!(Contracts::<Test>::get(contract_id), Some(contract));
				assert_eq!(Nft::owner(collection_id, nft_addr.1), Some(NftStake::account_id()));
				assert_eq!(
					Nft::owner(contract_collection_id, contract_id),
					Some(NftStake::account_id())
				);
				assert_eq!(NextContractId::<Test>::get(), contract_id + 1);

				System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Created {
					creator: ALICE,
					contract_id,
				}));
			});
	}

	#[test]
	fn rejects_non_creator_calls() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.balances(vec![(ALICE, ItemDeposit::get())])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let nft_addr = mint_item(ALICE, collection_id, 1);
				let reward = Reward::Nft(nft_addr);
				let contract = Contract::new(reward, 10, Default::default());
				assert_noop!(
					NftStake::create(RuntimeOrigin::signed(BOB), contract),
					DispatchError::BadOrigin
				);
			});
	}

	#[test]
	fn rejects_when_pallet_is_locked() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.build()
			.execute_with(|| {
				let reward = Reward::Tokens(333);
				let contract = Contract::new(reward, 10, Default::default());
				GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
				assert_noop!(
					NftStake::create(RuntimeOrigin::signed(ALICE), contract),
					Error::<Test>::PalletLocked
				);
			});
	}

	#[test]
	fn rejects_insufficient_balance() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.balances(vec![(ALICE, 333)])
			.build()
			.execute_with(|| {
				let reward = Reward::Tokens(2_000_000);
				let contract = Contract::new(reward, 10, Default::default());
				assert_noop!(
					NftStake::create(RuntimeOrigin::signed(ALICE), contract),
					pallet_balances::Error::<Test>::InsufficientBalance,
				);
			});
	}

	#[test]
	fn rejects_unowned_nfts() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let collection_id = create_collection(BOB);
			let nft_addr = mint_item(BOB, collection_id, 1);
			let reward = Reward::Nft(nft_addr);
			let contract = Contract::new(reward, 10, Default::default());
			assert_noop!(
				NftStake::create(RuntimeOrigin::signed(ALICE), contract),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn rejects_when_contract_collection_id_is_not_set() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let nft_addr = mint_item(ALICE, collection_id, 1);
			let reward = Reward::Nft(nft_addr);
			let contract = Contract::new(reward, 10, Default::default());
			assert_noop!(
				NftStake::create(RuntimeOrigin::signed(ALICE), contract),
				Error::<Test>::UnknownContractCollection
			);
		});
	}
}

mod accept {
	use super::*;

	#[test]
	fn works() {
		let stake_clauses = vec![
			Clause::HasAttribute(RESERVED_COLLECTION_0, 4),
			Clause::HasAttribute(RESERVED_COLLECTION_1, 5),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_2, 6, 7),
		];
		let duration = 4;
		let contract = ContractOf::<Test>::new(
			RewardOf::<Test>::Tokens(123),
			duration,
			stake_clauses.clone().try_into().unwrap(),
		);

		let stakes = MockStakes::from(MockClauses(stake_clauses));
		let staking_addresses = StakedItemsOf::<Test>::truncate_from(
			stakes.0.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>(),
		);

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.contracts(vec![contract])
			.stakes(vec![(BOB, stakes)])
			.build()
			.execute_with(|| {
				let contract_id = 0;
				assert_ok!(NftStake::accept(
					RuntimeOrigin::signed(BOB),
					contract_id,
					staking_addresses.clone()
				));

				// Contract ownership transferred to staker
				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
				assert_eq!(Nft::owner(contract_collection_id, contract_id), Some(BOB));

				// Stake ownership transferred to pallet account
				for addr in staking_addresses.clone() {
					let NftAddress(collection_id, item_id) = addr;
					assert_eq!(Nft::owner(collection_id, item_id), Some(NftStake::account_id()));
				}
				assert_eq!(ContractStakedItems::<Test>::get(contract_id), Some(staking_addresses));

				// Check contract duration
				let current_block = <frame_system::Pallet<Test>>::block_number();
				assert_eq!(
					ContractDurations::<Test>::get(contract_id),
					Some(current_block + duration)
				);

				System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Accepted {
					accepted_by: BOB,
					contract_id,
				}));
			});
	}

	#[test]
	fn rejects_unsigned_calls() {
		let contract = Contract::new(Reward::Tokens(123), 1, Default::default());
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.contracts(vec![contract])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(RuntimeOrigin::none(), Default::default(), Default::default()),
					DispatchError::BadOrigin
				);
			});
	}

	#[test]
	fn rejects_when_pallet_is_locked() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.build()
			.execute_with(|| {
				GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
				assert_noop!(
					NftStake::accept(RuntimeOrigin::none(), 1, Default::default()),
					DispatchError::BadOrigin
				);
			});
	}

	#[test]
	fn rejects_when_contract_collection_id_is_not_set() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.build()
			.execute_with(|| {
				ContractCollectionId::<Test>::kill();
				assert_noop!(
					NftStake::accept(RuntimeOrigin::signed(ALICE), 1, Default::default()),
					Error::<Test>::UnknownContractCollection
				);
			});
	}

	#[test]
	fn rejects_when_contract_is_already_accepted() {
		let clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 2)];
		let contract = Contract::new(Reward::Tokens(123), 123, clauses.clone().try_into().unwrap());

		let alice_stakes = MockStakes::from(MockClauses(clauses));
		let bob_stakes = MockStakes::inc_item_id(alice_stakes.clone());
		let charlie_stakes = MockStakes::inc_item_id(bob_stakes.clone());

		let alice_staking_addresses = StakedItemsOf::<Test>::truncate_from(
			alice_stakes.0.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>(),
		);
		let bob_staking_addresses = StakedItemsOf::<Test>::truncate_from(
			bob_stakes.0.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>(),
		);
		let charlie_staking_addresses = StakedItemsOf::<Test>::truncate_from(
			charlie_stakes
				.0
				.clone()
				.into_iter()
				.map(|(addr, _, _)| addr)
				.collect::<Vec<_>>(),
		);

		ExtBuilder::default()
			.set_creator(BOB)
			.create_contract_collection()
			.contracts(vec![contract])
			.stakes(vec![
				(ALICE, alice_stakes.clone()),
				(BOB, bob_stakes.clone()),
				(CHARLIE, charlie_stakes.clone()),
			])
			.build()
			.execute_with(|| {
				let contract_id = 0;
				assert_ok!(NftStake::accept(
					RuntimeOrigin::signed(ALICE),
					contract_id,
					alice_staking_addresses.clone()
				));

				for (staker, address, err) in [
					// alice has already staked
					(ALICE, alice_staking_addresses, Error::<Test>::Ownership),
					(BOB, bob_staking_addresses, Error::<Test>::ContractOwnership),
					(CHARLIE, charlie_staking_addresses, Error::<Test>::ContractOwnership),
				] {
					assert_noop!(
						NftStake::accept(RuntimeOrigin::signed(staker), contract_id, address),
						err
					);
				}
			});
	}

	#[test]
	fn rejects_unowned_stakes() {
		let clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 2)];
		let contract = Contract::new(Reward::Tokens(123), 123, clauses.clone().try_into().unwrap());

		let stakes = MockStakes::from(MockClauses(clauses));
		let staking_addresses = StakedItemsOf::<Test>::truncate_from(
			stakes.0.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>(),
		);

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.contracts(vec![contract])
			.stakes(vec![(BOB, stakes)])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(RuntimeOrigin::signed(CHARLIE), 0, staking_addresses),
					Error::<Test>::Ownership
				);
			});
	}

	#[test]
	fn rejects_when_contract_is_not_created() {
		let clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 2)];
		let contract = Contract::new(Reward::Tokens(123), 123, clauses.clone().try_into().unwrap());

		let stakes = MockStakes::from(MockClauses(clauses));
		let staking_addresses = StakedItemsOf::<Test>::truncate_from(
			stakes.0.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>(),
		);

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.contracts(vec![contract])
			.stakes(vec![(BOB, stakes)])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(RuntimeOrigin::signed(BOB), 123, staking_addresses),
					Error::<Test>::UnknownContract
				);
			});
	}

	#[test]
	fn rejects_unfulfilling_stakes() {
		let clauses = vec![
			Clause::HasAttribute(RESERVED_COLLECTION_0, 123),
			Clause::HasAttribute(RESERVED_COLLECTION_0, 456),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 12, 34),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 56, 78),
		];
		let contract = Contract::new(Reward::Tokens(123), 123, clauses.clone().try_into().unwrap());

		let mut stakes = MockStakes::from(MockClauses(clauses));
		stakes.0.iter_mut().for_each(|(_, key, value)| {
			*key += 1;
			*value += 1;
		});
		let staking_addresses = StakedItemsOf::<Test>::truncate_from(
			stakes.0.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>(),
		);

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.contracts(vec![contract])
			.stakes(vec![(BOB, stakes)])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(RuntimeOrigin::signed(BOB), 0, staking_addresses),
					Error::<Test>::UnfulfilledClause
				);
			});
	}
}

mod claim {
	use super::*;

	#[test]
	fn redeem_a_staking_contract_successfully_with_token_reward() {
		ExtBuilder::default().build().execute_with(|| {
			let attr_key = 10_u32;
			let contract_duration = 10;
			let reward_amount = 1_000;
			let reward = StakingRewardOf::<Test>::Tokens(reward_amount);

			let contract_addr = {
				let account = ALICE;
				let contract = StakingContractOf::<Test>::new(reward.clone(), contract_duration)
					.with_clause(ContractClause::HasAttribute(attr_key));

				create_and_submit_random_staking_contract_nft(account, contract)
			};

			let account = BOB;
			let current_balance = Balances::free_balance(account);

			let staked_nft = {
				let staked_nft = create_random_mock_nft_for(account);
				set_attribute_for_nft(&staked_nft, attr_key, 42_u64);
				staked_nft
			};

			let contract_id = contract_addr.1;

			assert_ok!(NftStake::accept(
				RuntimeOrigin::signed(account),
				contract_id,
				bounded_vec![staked_nft.clone()],
			));

			// Run to block
			let current_block = <frame_system::Pallet<Test>>::block_number();
			run_to_block(current_block + contract_duration);

			assert_ok!(NftStake::claim(RuntimeOrigin::signed(account), contract_id,));

			System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Claimed {
				claimed_by: account,
				contract_id,
				reward,
			}));

			assert_eq!(Balances::free_balance(account), current_balance + reward_amount);

			assert_eq!(Nft::owner(staked_nft.0, staked_nft.1), Some(account));

			assert_eq!(Nft::owner(contract_collection_id(), contract_id), None);

			assert_eq!(ContractOwners::<Test>::get(contract_id), None);

			assert_eq!(ContractDurations::<Test>::get(contract_id), None);

			assert_eq!(ContractStakedAssets::<Test>::get(contract_id), None);
		});
	}

	#[test]
	fn redeem_a_staking_contract_successfully_with_nft_reward() {
		ExtBuilder::default().build().execute_with(|| {
			let attr_key = 10_u32;
			let contract_duration = 10;

			let creator_account = ALICE;
			let collection_id = create_random_mock_nft_collection(creator_account);
			let nft_reward_addr = create_random_mock_nft(creator_account, collection_id, 1);

			let reward = StakingRewardOf::<Test>::Nft(nft_reward_addr.clone());

			let contract_addr = {
				let contract = StakingContractOf::<Test>::new(reward.clone(), 10)
					.with_clause(ContractClause::HasAttribute(10_u32));

				create_and_submit_random_staking_contract_nft(creator_account, contract)
			};

			let account = BOB;
			let staked_nft = {
				let staked_nft = create_random_mock_nft_for(account);
				set_attribute_for_nft(&staked_nft, attr_key, 42_u64);
				staked_nft
			};

			let contract_id = contract_addr.1;

			assert_ok!(NftStake::accept(
				RuntimeOrigin::signed(account),
				contract_id,
				bounded_vec![staked_nft.clone()],
			));

			// Run to block
			let current_block = <frame_system::Pallet<Test>>::block_number();
			run_to_block(current_block + contract_duration);

			assert_ok!(NftStake::claim(RuntimeOrigin::signed(account), contract_id,));

			System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Claimed {
				claimed_by: account,
				contract_id,
				reward,
			}));

			assert_eq!(Nft::owner(nft_reward_addr.0, nft_reward_addr.1), Some(account));

			assert_eq!(Nft::owner(staked_nft.0, staked_nft.1), Some(account));

			assert_eq!(Nft::owner(contract_collection_id(), contract_id), None);

			assert_eq!(ActiveContracts::<Test>::get(contract_id), None);

			assert_eq!(ContractOwners::<Test>::get(contract_id), None);

			assert_eq!(ContractDurations::<Test>::get(contract_id), None);

			assert_eq!(ContractStakedAssets::<Test>::get(contract_id), None);
		});
	}

	#[test]
	fn cannot_redeem_non_owned_contract() {
		ExtBuilder::default().build().execute_with(|| {
			let (taken_contract_id, expiry_block) = {
				let creator_account = ALICE;
				let contract_reward = StakingRewardOf::<Test>::Tokens(1_000);
				let contract_duration = 10;
				let contract = StakingContractOf::<Test>::new(contract_reward, contract_duration);

				let contract_addr =
					create_and_submit_random_staking_contract_nft(creator_account, contract);

				let taker_account = BOB;

				assert_ok!(NftStake::accept(
					RuntimeOrigin::signed(taker_account),
					contract_addr.1,
					bounded_vec![],
				));

				let contract_expiry =
					ContractDurations::<Test>::get(contract_addr.1).expect("Should contain expiry");

				(contract_addr.1, contract_expiry)
			};

			let contract_redeemer = CHARLIE;

			run_to_block(expiry_block);

			assert_noop!(
				NftStake::claim(RuntimeOrigin::signed(contract_redeemer), taken_contract_id,),
				Error::<Test>::ContractNotOwned
			);
		});
	}

	#[test]
	fn cannot_redeem_active_contract() {
		ExtBuilder::default().build().execute_with(|| {
			let contract_taker = BOB;
			let (taken_contract_id, expiry_block) = {
				let creator_account = ALICE;
				let contract_reward = StakingRewardOf::<Test>::Tokens(1_000);
				let contract_duration = 10;
				let contract = StakingContractOf::<Test>::new(contract_reward, contract_duration);

				let contract_addr =
					create_and_submit_random_staking_contract_nft(creator_account, contract);

				assert_ok!(NftStake::accept(
					RuntimeOrigin::signed(contract_taker),
					contract_addr.1,
					bounded_vec![],
				));

				let contract_expiry =
					ContractDurations::<Test>::get(contract_addr.1).expect("Should contain expiry");

				(contract_addr.1, contract_expiry)
			};

			run_to_block(expiry_block - 1);

			assert_noop!(
				NftStake::claim(RuntimeOrigin::signed(contract_taker), taken_contract_id,),
				Error::<Test>::ContractStillActive
			);
		});
	}
}

fn contract_collection_id() -> MockCollectionId {
	ContractCollectionId::<Test>::get().expect("Should get contract collection id")
}

fn create_collection(account: MockAccountId) -> MockCollectionId {
	<Test as Config>::NftHelper::create_collection(&account, &account, &CollectionConfig::default())
		.unwrap()
}

fn mint_item(
	owner: MockAccountId,
	collection_id: MockCollectionId,
	item_id: MockItemId,
) -> NftAddressOf<Test> {
	<Test as Config>::NftHelper::mint_into(
		&collection_id,
		&item_id,
		&owner,
		&pallet_nfts::ItemConfig::default(),
		true,
	)
	.unwrap();
	NftAddress(collection_id, item_id)
}

fn create_random_mock_nft_for(owner: MockAccountId) -> NftAddressOf<Test> {
	let collection_id = create_random_mock_nft_collection(owner);
	let item_id = 1;
	create_random_mock_nft(owner, collection_id, item_id)
}

fn create_and_submit_random_staking_contract_nft(
	creator: MockAccountId,
	contract: StakingContractOf<Test>,
) -> NftAddressOf<Test> {
	let collection_id = contract_collection_id();
	let expected_contract_id = NextContractId::<Test>::get();

	assert_ok!(NftStake::create(RuntimeOrigin::signed(creator), contract.clone()));

	assert_eq!(ActiveContracts::<Test>::get(expected_contract_id), Some(contract));

	NftAddress(collection_id, expected_contract_id)
}

fn set_attribute_for_nft(nft_addr: &NftAddressOf<Test>, nft_attr_key: u32, nft_attr_value: u64) {
	<Test as crate::pallet::Config>::NftHelper::set_typed_attribute::<u32, u64>(
		&nft_addr.0,
		&nft_addr.1,
		&nft_attr_key,
		&nft_attr_value,
	)
	.expect("Should add attribute NFT");
}
