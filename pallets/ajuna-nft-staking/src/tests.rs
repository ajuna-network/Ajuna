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
	traits::tokens::{
		nonfungibles_v2::{Create, Mutate},
		AttributeNamespace,
	},
};
use sp_runtime::bounded_vec;

mod organizer {
	use super::*;

	#[test]
	fn set_organizer_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(NftStake::organizer(), None);
			assert_ok!(NftStake::set_organizer(RuntimeOrigin::root(), ALICE));
			assert_eq!(NftStake::organizer(), Some(ALICE), "Organizer should be Alice");
			System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::OrganizerSet {
				organizer: ALICE,
			}));
		});
	}

	#[test]
	fn set_organizer_should_reject_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				NftStake::set_organizer(RuntimeOrigin::signed(BOB), ALICE),
				DispatchError::BadOrigin
			);
		});
	}
}

mod set_lock_state {
	use super::*;

	#[test]
	fn set_lock_state_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(NftStake::set_organizer(RuntimeOrigin::root(), ALICE));

			assert_ok!(NftStake::set_locked_state(
				RuntimeOrigin::signed(ALICE),
				PalletLockedState::Locked
			));
			assert_eq!(
				NftStake::lock_status(),
				PalletLockedState::Locked,
				"Pallet should be locked"
			);
			System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::LockedStateSet {
				locked_state: PalletLockedState::Locked,
			}));

			let contract_reward = StakingRewardOf::<Test>::Tokens(1_000);
			let contract = StakingContractOf::<Test>::new(contract_reward, 10);
			assert_noop!(
				NftStake::submit_staking_contract(RuntimeOrigin::signed(BOB), contract),
				Error::<Test>::PalletLocked
			);
		});
	}

	#[test]
	fn set_lock_state_should_fail_with_non_organizer_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(NftStake::set_organizer(RuntimeOrigin::root(), ALICE));

			assert_noop!(
				NftStake::set_locked_state(RuntimeOrigin::signed(BOB), PalletLockedState::Locked),
				DispatchError::BadOrigin
			);
		});
	}
}

mod fund_treasury {
	use super::*;

	#[test]
	fn fund_treasury_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = RuntimeOrigin::signed(ALICE);
			let fund_amount: MockBalance = 1_000_000;
			let treasury_account = NftStake::treasury_account().unwrap();
			let base_reserves = Balances::reserved_balance(treasury_account);

			assert_ok!(NftStake::fund_treasury(account, fund_amount));

			System::assert_last_event(mock::RuntimeEvent::NftStake(crate::Event::TreasuryFunded {
				funding_account: ALICE,
				funds: fund_amount,
			}));

			assert_eq!(Balances::reserved_balance(treasury_account), base_reserves + fund_amount);
		});
	}

	#[test]
	fn cannot_fund_treasury_without_funds() {
		let account_balance = 1_000;
		let fund_amount = account_balance * 10;
		ExtBuilder::default()
			.balances(vec![(ALICE, account_balance)])
			.build()
			.execute_with(|| {
				let account = RuntimeOrigin::signed(ALICE);

				assert_noop!(
					NftStake::fund_treasury(account, fund_amount),
					crate::pallet::Error::<Test>::AccountLacksFunds
				);
			});
	}

	#[test]
	fn cannot_fund_treasury_if_the_funding_account_would_be_killed() {
		let account_balance = 1_000;
		let fund_amount = account_balance - 10;
		ExtBuilder::default()
			.balances(vec![(ALICE, account_balance)])
			.build()
			.execute_with(|| {
				let account = RuntimeOrigin::signed(ALICE);

				assert_noop!(
					NftStake::fund_treasury(account, fund_amount),
					pallet_balances::Error::<Test>::KeepAlive
				);
			});
	}
}

mod submit_staking_contract {
	use super::*;

	#[test]
	fn can_submit_staking_contract_with_tokens_as_reward() {
		let account = ALICE;
		let account_balance = 1_000_000;
		ExtBuilder::default()
			.balances(vec![(account, account_balance)])
			.build()
			.execute_with(|| {
				let reward = 1_000;
				let contract_reward = StakingRewardOf::<Test>::Tokens(reward);
				let contract = StakingContractOf::<Test>::new(contract_reward, 10)
					.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, 10_u32));
				let treasury_account = NftStake::treasury_account().unwrap();
				let base_reserves = Balances::reserved_balance(treasury_account);

				let expected_contract_id = NftStake::next_contract_id();

				assert_ok!(NftStake::submit_staking_contract(
					RuntimeOrigin::signed(account),
					contract.clone()
				));

				System::assert_last_event(mock::RuntimeEvent::NftStake(
					crate::Event::StakingContractCreated {
						creator: account,
						contract: expected_contract_id,
					},
				));

				assert_eq!(NftStake::active_contracts(expected_contract_id), Some(contract));

				assert_eq!(
					Nft::owner(contract_collection_id(), expected_contract_id),
					Some(NftStake::treasury_account_id())
				);

				let new_reserve = Balances::reserved_balance(treasury_account);

				assert_eq!(Balances::free_balance(account), account_balance - reward);
				assert_eq!(new_reserve, base_reserves + reward);
			});
	}

	#[test]
	fn can_submit_staking_contract_with_nft_as_reward() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let collection_id = create_random_mock_nft_collection(account);
			let nft_addr = create_random_mock_nft(account, collection_id, 1);

			let contract_reward = StakingRewardOf::<Test>::Nft(nft_addr.clone());
			let contract = StakingContractOf::<Test>::new(contract_reward, 10)
				.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, 10_u32));

			let expected_contract_id = NftStake::next_contract_id();

			assert_ok!(NftStake::submit_staking_contract(
				RuntimeOrigin::signed(account),
				contract.clone()
			));

			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::StakingContractCreated {
					creator: account,
					contract: expected_contract_id,
				},
			));

			assert_eq!(NftStake::active_contracts(expected_contract_id), Some(contract));

			assert_eq!(
				Nft::owner(contract_collection_id(), expected_contract_id),
				Some(NftStake::treasury_account_id())
			);

			assert_eq!(Nft::owner(collection_id, nft_addr.1), Some(NftStake::treasury_account_id()))
		});
	}

	#[test]
	fn cannot_submit_staking_contract_without_enough_tokens_for_reward_in_account() {
		let account = ALICE;
		let account_balance = 1_000_000;
		ExtBuilder::default()
			.balances(vec![(account, account_balance)])
			.build()
			.execute_with(|| {
				let reward = 2_000_000;
				let contract_reward = StakingRewardOf::<Test>::Tokens(reward);
				let contract = StakingContractOf::<Test>::new(contract_reward, 10)
					.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, 10_u32));

				assert_noop!(
					NftStake::submit_staking_contract(RuntimeOrigin::signed(account), contract),
					Error::<Test>::AccountLacksFunds
				);
			});
	}

	#[test]
	fn cannot_submit_staking_contract_without_owning_the_nft_reward() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let other_account = BOB;
			let nft_addr = create_random_mock_nft_for(other_account);

			let contract_reward = StakingRewardOf::<Test>::Nft(nft_addr);
			let contract = StakingContractOf::<Test>::new(contract_reward, 10)
				.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, 10_u32));

			assert_noop!(
				NftStake::submit_staking_contract(RuntimeOrigin::signed(account), contract),
				Error::<Test>::ContractRewardNotOwned
			);
		});
	}

	#[test]
	fn cannot_submit_staking_contract_with_another_contract_as_reward() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let collection_id = contract_collection_id();
			let item_id = NftStake::next_contract_id() + 100;
			let nft_addr = create_random_mock_nft(account, collection_id, item_id);

			let contract_reward = StakingRewardOf::<Test>::Nft(nft_addr);
			let contract = StakingContractOf::<Test>::new(contract_reward, 10)
				.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, 10_u32));

			assert_noop!(
				NftStake::submit_staking_contract(RuntimeOrigin::signed(account), contract),
				Error::<Test>::InvalidContractReward
			);
		});
	}
}

mod take_staking_contract {
	use super::*;

	#[test]
	fn take_staking_contract_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key = 10_u32;
			let contract_duration = 10;
			let contract = StakingContractOf::<Test>::new(
				StakingRewardOf::<Test>::Tokens(1_000),
				contract_duration,
			)
			.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, attr_key));
			let expected_contract_id = NftStake::next_contract_id();
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			let contract_taker = BOB;
			let staked_nft = create_random_mock_nft_for(contract_taker);
			set_attribute_for_nft(&staked_nft, attr_key, 42_u64);

			assert_ok!(NftStake::take_staking_contract(
				RuntimeOrigin::signed(contract_taker),
				contract_addr.1,
				bounded_vec![staked_nft.clone()],
			));

			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::StakingContractTaken {
					taken_by: contract_taker,
					contract: expected_contract_id,
				},
			));

			assert_eq!(
				Nft::owner(staked_nft.0, staked_nft.1),
				Some(NftStake::treasury_account_id())
			);

			assert_eq!(
				Nft::owner(contract_collection_id(), expected_contract_id),
				Some(contract_taker)
			);

			assert_eq!(NftStake::contract_owners(expected_contract_id), Some(contract_taker));

			let current_block = <frame_system::Pallet<Test>>::block_number();
			assert_eq!(
				NftStake::contract_durations(expected_contract_id),
				Some(current_block + contract_duration)
			);

			assert_eq!(
				NftStake::contract_staked_assets(expected_contract_id),
				Some(bounded_vec![staked_nft])
			);
		});
	}

	#[test]
	fn take_a_complex_staking_contract_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key_set = vec![10_u32, 15_u32, 57_u32];
			let attr_value_set = vec![0_u64, 11_u64, 2812_u64];
			let contract_duration = 10;
			let contract = StakingContractOf::<Test>::new(
				StakingRewardOf::<Test>::Tokens(1_000),
				contract_duration,
			)
			.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, attr_key_set[0]))
			.with_clause(ContractClause::HasAttributeWithValue(
				AttributeNamespace::Pallet,
				attr_key_set[1],
				attr_value_set[1],
			))
			.with_clause(ContractClause::HasAttributeWithValue(
				AttributeNamespace::Pallet,
				attr_key_set[2],
				attr_value_set[2],
			));
			let expected_contract_id = NftStake::next_contract_id();
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			let contract_taker = BOB;
			let staked_nft_vec: StakedAssetsVecOf<Test> = {
				let staked_nft_1 = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft_1, attr_key_set[0], attr_value_set[0]);

				let staked_nft_2 = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft_2, attr_key_set[1], attr_value_set[1]);

				let staked_nft_3 = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft_3, attr_key_set[2], attr_value_set[2]);

				bounded_vec![staked_nft_1, staked_nft_2, staked_nft_3]
			};

			assert_ok!(NftStake::take_staking_contract(
				RuntimeOrigin::signed(contract_taker),
				contract_addr.1,
				staked_nft_vec,
			));

			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::StakingContractTaken {
					taken_by: contract_taker,
					contract: expected_contract_id,
				},
			));
		});
	}

	#[test]
	fn fail_to_take_an_already_taken_by_other_staking_contract() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key = 10_u32;
			let contract =
				StakingContractOf::<Test>::new(StakingRewardOf::<Test>::Tokens(1_000), 10)
					.with_clause(ContractClause::HasAttribute(
						AttributeNamespace::Pallet,
						attr_key,
					));
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			// Contract taken by another
			{
				let contract_taker = BOB;
				let staked_nft_vec = {
					let staked_nft = create_random_mock_nft_for(contract_taker);
					set_attribute_for_nft(&staked_nft, attr_key, 42_u64);

					bounded_vec![staked_nft]
				};

				assert_ok!(NftStake::take_staking_contract(
					RuntimeOrigin::signed(contract_taker),
					contract_addr.1,
					staked_nft_vec,
				));
			};

			// Trying to take already taken contract
			{
				let contract_taker = CHARLIE;
				let staked_nft_vec = {
					let staked_nft = create_random_mock_nft_for(contract_taker);
					set_attribute_for_nft(&staked_nft, attr_key, 42_u64);

					bounded_vec![staked_nft]
				};

				assert_noop!(
					NftStake::take_staking_contract(
						RuntimeOrigin::signed(contract_taker),
						contract_addr.1,
						staked_nft_vec,
					),
					Error::<Test>::ContractTakenByOther
				);
			};
		});
	}

	#[test]
	fn fail_to_take_an_already_taken_by_self_staking_contract() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key = 10_u32;
			let contract =
				StakingContractOf::<Test>::new(StakingRewardOf::<Test>::Tokens(1_000), 10)
					.with_clause(ContractClause::HasAttribute(
						AttributeNamespace::Pallet,
						attr_key,
					));
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			// Trying to take contract again
			{
				let contract_taker = BOB;
				let staked_nft_vec: StakedAssetsVecOf<Test> = {
					let staked_nft = create_random_mock_nft_for(contract_taker);
					set_attribute_for_nft(&staked_nft, attr_key, 42_u64);

					bounded_vec![staked_nft]
				};

				assert_ok!(NftStake::take_staking_contract(
					RuntimeOrigin::signed(contract_taker),
					contract_addr.1,
					staked_nft_vec.clone(),
				));

				assert_noop!(
					NftStake::take_staking_contract(
						RuntimeOrigin::signed(contract_taker),
						contract_addr.1,
						staked_nft_vec,
					),
					Error::<Test>::ContractAlreadyTaken
				);
			};
		});
	}

	#[test]
	fn fail_to_take_staking_contract_on_unfulfilled_conditions() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key = 10_u32;
			let contract =
				StakingContractOf::<Test>::new(StakingRewardOf::<Test>::Tokens(1_000), 10)
					.with_clause(ContractClause::HasAttribute(
						AttributeNamespace::Pallet,
						attr_key,
					));
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			let contract_taker = BOB;
			let nft_attr_key = 13_u32;
			let staked_nft_vec = {
				let staked_nft = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft, nft_attr_key, 42_u64);

				bounded_vec![staked_nft]
			};

			assert_noop!(
				NftStake::take_staking_contract(
					RuntimeOrigin::signed(contract_taker),
					contract_addr.1,
					staked_nft_vec,
				),
				Error::<Test>::ContractConditionsNotFulfilled
			);
		});
	}

	#[test]
	fn fail_to_take_a_complex_staking_contract_on_unfulfilled_conditions() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key_set = vec![10_u32, 15_u32, 57_u32];
			let attr_value_set = vec![0_u64, 11_u64, 2812_u64];
			let contract_duration = 10;
			let contract = StakingContractOf::<Test>::new(
				StakingRewardOf::<Test>::Tokens(5_000),
				contract_duration,
			)
			.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, attr_key_set[0]))
			.with_clause(ContractClause::HasAttributeWithValue(
				AttributeNamespace::Pallet,
				attr_key_set[1],
				attr_value_set[1],
			))
			.with_clause(ContractClause::HasAttributeWithValue(
				AttributeNamespace::Pallet,
				attr_key_set[2],
				attr_value_set[2],
			));
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			let contract_taker = BOB;
			let staked_nft_vec: StakedAssetsVecOf<Test> = {
				let staked_nft_1 = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft_1, attr_key_set[0], attr_value_set[0]);

				let staked_nft_2 = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft_2, attr_key_set[1], attr_value_set[1]);

				let staked_nft_3 = create_random_mock_nft_for(contract_taker);
				set_attribute_for_nft(&staked_nft_3, attr_key_set[2], attr_value_set[1]);

				bounded_vec![staked_nft_1, staked_nft_2, staked_nft_3]
			};

			assert_noop!(
				NftStake::take_staking_contract(
					RuntimeOrigin::signed(contract_taker),
					contract_addr.1,
					staked_nft_vec,
				),
				Error::<Test>::ContractConditionsNotFulfilled
			);
		});
	}

	#[test]
	fn fail_to_take_a_staking_contract_with_non_owned_assets() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let attr_key = 10_u32;
			let contract =
				StakingContractOf::<Test>::new(StakingRewardOf::<Test>::Tokens(1_000), 10)
					.with_clause(ContractClause::HasAttribute(
						AttributeNamespace::Pallet,
						attr_key,
					));
			let contract_addr = create_and_submit_random_staking_contract_nft(account, contract);

			// Trying to take contract
			{
				let contract_taker = BOB;
				let nft_owner = CHARLIE;
				let staked_nft_vec: StakedAssetsVecOf<Test> = {
					let staked_nft = create_random_mock_nft_for(nft_owner);
					set_attribute_for_nft(&staked_nft, attr_key, 42_u64);

					bounded_vec![staked_nft]
				};

				assert_noop!(
					NftStake::take_staking_contract(
						RuntimeOrigin::signed(contract_taker),
						contract_addr.1,
						staked_nft_vec,
					),
					Error::<Test>::StakedAssetNotOwned
				);
			};
		});
	}
}

mod redeem_staking_contract {
	use super::*;

	#[test]
	fn redeem_a_staking_contract_successfully_with_token_reward() {
		ExtBuilder::default().build().execute_with(|| {
			let attr_key = 10_u32;
			let contract_duration = 10;
			let reward_amt = 1_000;
			let contract_reward = StakingRewardOf::<Test>::Tokens(reward_amt);

			let contract_addr = {
				let account = ALICE;
				let contract =
					StakingContractOf::<Test>::new(contract_reward.clone(), contract_duration)
						.with_clause(ContractClause::HasAttribute(
							AttributeNamespace::Pallet,
							attr_key,
						));

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

			assert_ok!(NftStake::take_staking_contract(
				RuntimeOrigin::signed(account),
				contract_id,
				bounded_vec![staked_nft.clone()],
			));

			// Run to block
			let current_block = <frame_system::Pallet<Test>>::block_number();
			run_to_block(current_block + contract_duration);

			assert_ok!(NftStake::redeem_staking_contract(
				RuntimeOrigin::signed(account),
				contract_id,
			));

			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::StakingContractRedeemed {
					redeemed_by: account,
					contract: contract_id,
					reward: contract_reward,
				},
			));

			assert_eq!(Balances::free_balance(account), current_balance + reward_amt);

			assert_eq!(Nft::owner(staked_nft.0, staked_nft.1), Some(account));

			assert_eq!(Nft::owner(contract_collection_id(), contract_id), None);

			assert_eq!(NftStake::contract_owners(contract_id), None);

			assert_eq!(NftStake::contract_durations(contract_id), None);

			assert_eq!(NftStake::contract_staked_assets(contract_id), None);
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

			let contract_reward = StakingRewardOf::<Test>::Nft(nft_reward_addr.clone());

			let contract_addr = {
				let contract = StakingContractOf::<Test>::new(contract_reward.clone(), 10)
					.with_clause(ContractClause::HasAttribute(AttributeNamespace::Pallet, 10_u32));

				create_and_submit_random_staking_contract_nft(creator_account, contract)
			};

			let account = BOB;
			let staked_nft = {
				let staked_nft = create_random_mock_nft_for(account);
				set_attribute_for_nft(&staked_nft, attr_key, 42_u64);
				staked_nft
			};

			let contract_id = contract_addr.1;

			assert_ok!(NftStake::take_staking_contract(
				RuntimeOrigin::signed(account),
				contract_id,
				bounded_vec![staked_nft.clone()],
			));

			// Run to block
			let current_block = <frame_system::Pallet<Test>>::block_number();
			run_to_block(current_block + contract_duration);

			assert_ok!(NftStake::redeem_staking_contract(
				RuntimeOrigin::signed(account),
				contract_id,
			));

			System::assert_last_event(mock::RuntimeEvent::NftStake(
				crate::Event::StakingContractRedeemed {
					redeemed_by: account,
					contract: contract_id,
					reward: contract_reward,
				},
			));

			assert_eq!(Nft::owner(nft_reward_addr.0, nft_reward_addr.1), Some(account));

			assert_eq!(Nft::owner(staked_nft.0, staked_nft.1), Some(account));

			assert_eq!(Nft::owner(contract_collection_id(), contract_id), None);

			assert_eq!(NftStake::active_contracts(contract_id), None);

			assert_eq!(NftStake::contract_owners(contract_id), None);

			assert_eq!(NftStake::contract_durations(contract_id), None);

			assert_eq!(NftStake::contract_staked_assets(contract_id), None);
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

				assert_ok!(NftStake::take_staking_contract(
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
				NftStake::redeem_staking_contract(
					RuntimeOrigin::signed(contract_redeemer),
					taken_contract_id,
				),
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

				assert_ok!(NftStake::take_staking_contract(
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
				NftStake::redeem_staking_contract(
					RuntimeOrigin::signed(contract_taker),
					taken_contract_id,
				),
				Error::<Test>::ContractStillActive
			);
		});
	}
}

fn contract_collection_id() -> MockCollectionId {
	ContractCollectionId::<Test>::get().expect("Should get contract collection id")
}

fn create_random_mock_nft_collection(account: MockAccountId) -> MockCollectionId {
	let collection_config = CollectionConfig::default();
	<Test as crate::pallet::Config>::NftHelper::create_collection(
		&account,
		&account,
		&collection_config,
	)
	.expect("Should have create contract collection")
}

fn create_random_mock_nft(
	owner: MockAccountId,
	collection_id: MockCollectionId,
	item_id: MockItemId,
) -> NftAddressOf<Test> {
	let item_config = pallet_nfts::ItemConfig::default();
	<Test as crate::pallet::Config>::NftHelper::mint_into(
		&collection_id,
		&item_id,
		&owner,
		&item_config,
		true,
	)
	.expect("Should create NFT");

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
	let expected_contract_id = NftStake::next_contract_id();

	assert_ok!(NftStake::submit_staking_contract(RuntimeOrigin::signed(creator), contract.clone()));

	assert_eq!(NftStake::active_contracts(expected_contract_id), Some(contract));

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
