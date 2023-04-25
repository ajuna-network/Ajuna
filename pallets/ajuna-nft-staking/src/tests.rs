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
use frame_support::{assert_noop, assert_ok};
use sp_runtime::testing::H256;

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
				let contract = Contract::new(reward, 10, Default::default(), Default::default());
				let base_reserves = CurrencyOf::<Test>::free_balance(NftStake::account_id());

				let contract_id = H256::random();
				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

				assert_ok!(NftStake::create(
					RuntimeOrigin::signed(ALICE),
					contract_id,
					contract.clone()
				));
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
				let nft_addr = mint_item(&ALICE, &collection_id, &H256::default());
				let reward = Reward::Nft(nft_addr.clone());
				let contract = Contract::new(reward, 10, Default::default(), Default::default());

				let contract_id = H256::random();
				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

				assert_ok!(NftStake::create(
					RuntimeOrigin::signed(ALICE),
					contract_id,
					contract.clone()
				));
				assert_eq!(Contracts::<Test>::get(contract_id), Some(contract));
				assert_eq!(Nft::owner(collection_id, nft_addr.1), Some(NftStake::account_id()));
				assert_eq!(
					Nft::owner(contract_collection_id, contract_id),
					Some(NftStake::account_id())
				);

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
				let nft_addr = mint_item(&ALICE, &collection_id, &H256::default());
				let reward = Reward::Nft(nft_addr);
				let contract = Contract::new(reward, 10, Default::default(), Default::default());
				assert_noop!(
					NftStake::create(RuntimeOrigin::signed(BOB), H256::random(), contract),
					DispatchError::BadOrigin
				);
			});
	}

	#[test]
	fn rejects_when_pallet_is_locked() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
			assert_noop!(
				NftStake::create(
					RuntimeOrigin::signed(ALICE),
					H256::random(),
					Contract::new(Reward::Tokens(333), 10, Default::default(), Default::default())
				),
				Error::<Test>::PalletLocked
			);
		});
	}

	#[test]
	fn rejects_out_of_bound_staking_clauses() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let staking_clauses = (0..MaxStakingClauses::get() + 1)
				.map(|i| Clause::HasAttribute(RESERVED_COLLECTION_0, i))
				.collect::<Vec<_>>();
			assert!(staking_clauses.len() as u32 > MaxStakingClauses::get());
			assert_noop!(
				NftStake::create(
					RuntimeOrigin::signed(ALICE),
					H256::random(),
					Contract::new(
						Reward::Tokens(1),
						10,
						staking_clauses.try_into().unwrap(),
						Default::default()
					)
				),
				Error::<Test>::MaxStakingClauses
			);
		});
	}

	#[test]
	fn rejects_out_of_bound_fee_clauses() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let fee_clauses = (0..MaxFeeClauses::get() + 1)
				.map(|i| Clause::HasAttribute(RESERVED_COLLECTION_0, i))
				.collect::<Vec<_>>();
			assert!(fee_clauses.len() as u32 > MaxFeeClauses::get());
			assert_noop!(
				NftStake::create(
					RuntimeOrigin::signed(ALICE),
					H256::random(),
					Contract::new(
						Reward::Tokens(1),
						10,
						Default::default(),
						fee_clauses.try_into().unwrap()
					)
				),
				Error::<Test>::MaxFeeClauses
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
				assert_noop!(
					NftStake::create(
						RuntimeOrigin::signed(ALICE),
						H256::random(),
						Contract::new(
							Reward::Tokens(2_000_000),
							10,
							Default::default(),
							Default::default()
						)
					),
					pallet_balances::Error::<Test>::InsufficientBalance,
				);
			});
	}

	#[test]
	fn rejects_unowned_nfts() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let collection_id = create_collection(BOB);
			let nft_addr = mint_item(&BOB, &collection_id, &H256::random());
			let reward = Reward::Nft(nft_addr);
			let contract = Contract::new(reward, 10, Default::default(), Default::default());
			assert_noop!(
				NftStake::create(RuntimeOrigin::signed(ALICE), H256::random(), contract),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn rejects_when_contract_collection_id_is_not_set() {
		ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let nft_addr = mint_item(&ALICE, &collection_id, &H256::random());
			let reward = Reward::Nft(nft_addr);
			let contract = Contract::new(reward, 10, Default::default(), Default::default());
			assert_noop!(
				NftStake::create(RuntimeOrigin::signed(ALICE), H256::random(), contract),
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
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 123)];
		let duration = 4;
		let contract = Contract::new(
			Reward::Tokens(123),
			duration,
			stake_clauses.clone().try_into().unwrap(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();

		let stakes = MockMints::from(MockClauses(stake_clauses));
		let stake_addresses =
			stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		let fees = MockMints::from(MockClauses(fee_clauses));
		let fee_addresses = fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.mint_stakes(vec![(BOB, stakes)])
			.mint_fees(vec![(BOB, fees)])
			.build()
			.execute_with(|| {
				assert_ok!(NftStake::accept(
					RuntimeOrigin::signed(BOB),
					contract_id,
					stake_addresses.clone(),
					fee_addresses.clone()
				));

				// Contract ownership transferred to staker
				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
				assert_eq!(Nft::owner(contract_collection_id, contract_id), Some(BOB));

				// Stake ownership transferred to pallet account
				for NftAddress(collection_id, item_id) in stake_addresses.clone() {
					assert_eq!(Nft::owner(collection_id, item_id), Some(NftStake::account_id()));
				}
				assert_eq!(
					ContractStakedItems::<Test>::get(contract_id),
					Some(stake_addresses.try_into().unwrap())
				);

				// Fee ownership transferred to creator
				for NftAddress(collection_id, item_id) in fee_addresses.clone() {
					assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
				}

				// Check contract duration
				let current_block = <frame_system::Pallet<Test>>::block_number();
				assert_eq!(ContractEnds::<Test>::get(contract_id), Some(current_block + duration));

				System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Accepted {
					accepted_by: BOB,
					contract_id,
				}));
			});
	}

	#[test]
	fn rejects_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				NftStake::accept(
					RuntimeOrigin::none(),
					Default::default(),
					Default::default(),
					Default::default()
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn rejects_when_pallet_is_locked() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
			assert_noop!(
				NftStake::accept(
					RuntimeOrigin::none(),
					Default::default(),
					Default::default(),
					Default::default()
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn rejects_out_of_bound_stakes() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.build()
			.execute_with(|| {
				let stake_addresses = (0..MaxStakingClauses::get() + 1)
					.map(|_| NftAddress(RESERVED_COLLECTION_0, H256::random()))
					.collect::<Vec<_>>();
				assert!(stake_addresses.len() as u32 > MaxStakingClauses::get());
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(ALICE),
						Default::default(),
						stake_addresses,
						Default::default()
					),
					Error::<Test>::MaxStakingClauses
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
					NftStake::accept(
						RuntimeOrigin::signed(ALICE),
						Default::default(),
						Default::default(),
						Default::default()
					),
					Error::<Test>::UnknownContractCollection
				);
			});
	}

	#[test]
	fn rejects_when_contract_is_already_accepted() {
		let stake_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 2)];
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 123)];
		let contract = Contract::new(
			Reward::Tokens(123),
			123,
			stake_clauses.clone().try_into().unwrap(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();

		let alice_stakes = MockMints::from(MockClauses(stake_clauses));
		let bob_stakes = alice_stakes
			.clone()
			.into_iter()
			.map(|(NftAddress(collection_id, _item_id), key, value)| {
				(NftAddress(collection_id, H256::random()), key, value)
			})
			.collect::<Vec<_>>();
		let charlie_stakes = bob_stakes
			.clone()
			.into_iter()
			.map(|(NftAddress(collection_id, _item_id), key, value)| {
				(NftAddress(collection_id, H256::random()), key, value)
			})
			.collect::<Vec<_>>();

		let alice_stake_addresses =
			alice_stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();
		let bob_stake_addresses =
			bob_stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();
		let charlie_stake_addresses =
			charlie_stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		let alice_fees = MockMints::from(MockClauses(fee_clauses));
		let bob_fees = alice_fees
			.clone()
			.into_iter()
			.map(|(NftAddress(collection_id, _item_id), key, value)| {
				(NftAddress(collection_id, H256::random()), key, value)
			})
			.collect::<Vec<_>>();
		let charlie_fees = bob_fees
			.clone()
			.into_iter()
			.map(|(NftAddress(collection_id, _item_id), key, value)| {
				(NftAddress(collection_id, H256::random()), key, value)
			})
			.collect::<Vec<_>>();

		let alice_fee_addresses =
			alice_fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();
		let bob_fee_addresses =
			bob_fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();
		let charlie_fee_addresses =
			charlie_fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		ExtBuilder::default()
			.set_creator(BOB)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.mint_stakes(vec![(ALICE, alice_stakes), (BOB, bob_stakes), (CHARLIE, charlie_stakes)])
			.mint_fees(vec![(ALICE, alice_fees), (BOB, bob_fees), (CHARLIE, charlie_fees)])
			.build()
			.execute_with(|| {
				assert_ok!(NftStake::accept(
					RuntimeOrigin::signed(ALICE),
					contract_id,
					alice_stake_addresses.clone(),
					alice_fee_addresses.clone()
				));

				for (staker, stake_addr, fee_addr, err) in [
					// alice has already staked
					(ALICE, alice_stake_addresses, alice_fee_addresses, Error::<Test>::Ownership),
					(BOB, bob_stake_addresses, bob_fee_addresses, Error::<Test>::ContractOwnership),
					(
						CHARLIE,
						charlie_stake_addresses,
						charlie_fee_addresses,
						Error::<Test>::ContractOwnership,
					),
				] {
					assert_noop!(
						NftStake::accept(
							RuntimeOrigin::signed(staker),
							contract_id,
							stake_addr,
							fee_addr
						),
						err
					);
				}
			});
	}

	#[test]
	fn rejects_unowned_stakes() {
		let stake_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 2)];
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 2)];
		let contract = Contract::new(
			Reward::Tokens(123),
			123,
			stake_clauses.clone().try_into().unwrap(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();

		let stakes = MockMints::from(MockClauses(stake_clauses));
		let stake_addresses =
			stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		let fees = MockMints::from(MockClauses(fee_clauses));
		let fee_addresses = fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.mint_stakes(vec![(BOB, stakes)])
			.mint_fees(vec![(BOB, fees)])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(CHARLIE),
						contract_id,
						stake_addresses,
						fee_addresses
					),
					Error::<Test>::Ownership
				);
			});
	}

	#[test]
	fn rejects_when_contract_is_not_created() {
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(BOB),
						H256::random(),
						Default::default(),
						Default::default()
					),
					Error::<Test>::UnknownContract
				);
			});
	}

	#[test]
	fn rejects_unfulfilling_stakes() {
		let stake_clauses = vec![
			Clause::HasAttribute(RESERVED_COLLECTION_0, 123),
			Clause::HasAttribute(RESERVED_COLLECTION_0, 456),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 12, 34),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 56, 78),
		];
		let contract = Contract::new(
			Reward::Tokens(123),
			123,
			stake_clauses.clone().try_into().unwrap(),
			Default::default(),
		);
		let contract_id = H256::random();

		let mut stakes = MockMints::from(MockClauses(stake_clauses));
		stakes.iter_mut().for_each(|(_, key, value)| {
			*key += 1;
			*value += 1;
		});
		let stake_addresses =
			stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.mint_stakes(vec![(BOB, stakes)])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(BOB),
						contract_id,
						stake_addresses,
						Default::default()
					),
					Error::<Test>::UnfulfilledStakingClause
				);
			});
	}

	#[test]
	fn rejects_unfulfilling_fees() {
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 123)];
		let contract = Contract::new(
			Reward::Tokens(123),
			123,
			Default::default(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();

		let mut fees = MockMints::from(MockClauses(fee_clauses));
		fees.iter_mut().for_each(|(_, key, value)| {
			*key += 1;
			*value += 1;
		});
		let fee_addresses = fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.mint_fees(vec![(BOB, fees)])
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(BOB),
						contract_id,
						Default::default(),
						fee_addresses
					),
					Error::<Test>::UnfulfilledFeeClause
				);
			});
	}
}

mod claim {
	use super::*;

	#[test]
	fn works_with_token_reward() {
		let stake_clauses = vec![
			Clause::HasAttribute(RESERVED_COLLECTION_0, 4),
			Clause::HasAttribute(RESERVED_COLLECTION_1, 5),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_2, 6, 7),
		];
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_2, 33)];
		let duration = 4;
		let reward_amount = 135;
		let contract = Contract::new(
			Reward::Tokens(reward_amount),
			duration,
			stake_clauses.clone().try_into().unwrap(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();

		let stakes = MockMints::from(MockClauses(stake_clauses));
		let stake_addresses =
			stakes.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
		let fees = MockMints::from(MockClauses(fee_clauses));
		let fee_addresses =
			fees.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
		let initial_balance = 333;

		ExtBuilder::default()
			.set_creator(ALICE)
			.balances(vec![(BOB, initial_balance)])
			.create_contract_collection()
			.create_contract(contract_id, contract.clone())
			.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
			.build()
			.execute_with(|| {
				run_to_block(System::block_number() + duration);

				assert_ok!(NftStake::claim(RuntimeOrigin::signed(BOB), contract_id));

				for NftAddress(collection_id, item_id) in stake_addresses {
					assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
				}
				for NftAddress(collection_id, item_id) in fee_addresses {
					assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
				}
				assert_eq!(Balances::free_balance(BOB), initial_balance + reward_amount);

				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
				assert_eq!(Nft::owner(contract_collection_id, contract_id), None);
				assert_eq!(ContractOwners::<Test>::get(contract_id), None);
				assert_eq!(ContractEnds::<Test>::get(contract_id), None);
				assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);

				System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Claimed {
					claimed_by: BOB,
					contract_id,
					reward: contract.reward,
				}));
			});
	}

	#[test]
	fn works_with_nft_reward() {
		let stake_clauses = vec![
			Clause::HasAttribute(RESERVED_COLLECTION_0, 4),
			Clause::HasAttribute(RESERVED_COLLECTION_0, 5),
			Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 6, 7),
		];
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_1, 1)];
		let duration = 8;
		let reward_addr = NftAddress(RESERVED_COLLECTION_2, H256::random());
		let contract = Contract::new(
			Reward::Nft(reward_addr.clone()),
			duration,
			stake_clauses.clone().try_into().unwrap(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();

		let stakes = MockMints::from(MockClauses(stake_clauses));
		let stake_addresses =
			stakes.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
		let fees = MockMints::from(MockClauses(fee_clauses));
		let fee_addresses =
			fees.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract.clone())
			.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
			.build()
			.execute_with(|| {
				run_to_block(System::block_number() + duration);

				assert_ok!(NftStake::claim(RuntimeOrigin::signed(BOB), contract_id));

				for NftAddress(collection_id, item_id) in stake_addresses {
					assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
				}
				for NftAddress(collection_id, item_id) in fee_addresses {
					assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
				}
				assert_eq!(Nft::owner(reward_addr.0, reward_addr.1), Some(BOB));

				let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
				assert_eq!(Nft::owner(contract_collection_id, contract_id), None);
				assert_eq!(ContractOwners::<Test>::get(contract_id), None);
				assert_eq!(ContractEnds::<Test>::get(contract_id), None);
				assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);

				System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::Claimed {
					claimed_by: BOB,
					contract_id,
					reward: contract.reward,
				}));
			});
	}

	#[test]
	fn rejects_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				NftStake::claim(RuntimeOrigin::none(), Default::default()),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn rejects_when_pallet_is_locked() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
			assert_noop!(
				NftStake::claim(RuntimeOrigin::signed(ALICE), Default::default()),
				Error::<Test>::PalletLocked
			);
		});
	}

	#[test]
	fn rejects_when_contract_is_not_owned() {
		let contract =
			Contract::new(Reward::Tokens(321), 123, Default::default(), Default::default());
		let contract_id = H256::random();
		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.build()
			.execute_with(|| {
				assert_noop!(
					NftStake::claim(RuntimeOrigin::signed(BOB), contract_id),
					Error::<Test>::ContractOwnership
				);
			});
	}

	#[test]
	fn rejects_when_contract_is_active() {
		let stake_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_0, 4)];
		let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_2, 2)];
		let duration = 3;
		let contract = Contract::new(
			Reward::Tokens(321),
			duration,
			stake_clauses.clone().try_into().unwrap(),
			fee_clauses.clone().try_into().unwrap(),
		);
		let contract_id = H256::random();
		let stakes = MockMints::from(MockClauses(stake_clauses));
		let fees = MockMints::from(MockClauses(fee_clauses));

		ExtBuilder::default()
			.set_creator(ALICE)
			.create_contract_collection()
			.create_contract(contract_id, contract)
			.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
			.build()
			.execute_with(|| {
				for i in 0..(duration - 1) {
					run_to_block(System::block_number() + i);
					assert_noop!(
						NftStake::claim(RuntimeOrigin::signed(BOB), contract_id),
						Error::<Test>::ContractStillActive
					);
				}
			});
	}
}
