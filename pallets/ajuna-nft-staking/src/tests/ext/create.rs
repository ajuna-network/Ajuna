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
			let mut contract = Contract::default().reward(Reward::Tokens(reward_amount));
			let base_reserves = CurrencyOf::<Test>::free_balance(NftStake::account_id());

			let contract_id = CONTRACT_ID;
			let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

			assert_ok!(NftStake::create(
				RuntimeOrigin::signed(ALICE),
				contract_id,
				contract.clone()
			));
			contract.activation = Some(System::block_number());
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

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Created {
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
			let nft_addr = mint_item(&ALICE, &collection_id, &123);
			let mut contract = Contract::default().reward(Reward::Nft(nft_addr.clone()));

			let contract_id = CONTRACT_ID;
			let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

			assert_ok!(NftStake::create(
				RuntimeOrigin::signed(ALICE),
				contract_id,
				contract.clone()
			));
			contract.activation = Some(System::block_number());
			assert_eq!(Contracts::<Test>::get(contract_id), Some(contract));
			assert_eq!(Nft::owner(collection_id, nft_addr.1), Some(NftStake::account_id()));
			assert_eq!(
				Nft::owner(contract_collection_id, contract_id),
				Some(NftStake::account_id())
			);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Created {
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
			assert_noop!(
				NftStake::create(RuntimeOrigin::signed(BOB), CONTRACT_ID, Contract::default()),
				DispatchError::BadOrigin
			);
		});
}

#[test]
fn rejects_when_pallet_is_locked() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
		assert_noop!(
			NftStake::create(RuntimeOrigin::signed(ALICE), CONTRACT_ID, Contract::default()),
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
				CONTRACT_ID,
				Contract::default().reward(Reward::Tokens(1)).stake_clauses(staking_clauses)
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
				CONTRACT_ID,
				Contract::default().reward(Reward::Tokens(1)).fee_clauses(fee_clauses)
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
					CONTRACT_ID,
					Contract::default().reward(Reward::Tokens(2_000_000))
				),
				pallet_balances::Error::<Test>::InsufficientBalance,
			);
		});
}

#[test]
fn rejects_unowned_nfts() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		let collection_id = create_collection(BOB);
		let nft_addr = mint_item(&BOB, &collection_id, &333);
		let contract = Contract::default().reward(Reward::Nft(nft_addr));
		assert_noop!(
			NftStake::create(RuntimeOrigin::signed(ALICE), CONTRACT_ID, contract),
			Error::<Test>::Ownership
		);
	});
}

#[test]
fn rejects_when_contract_collection_id_is_not_set() {
	ExtBuilder::default().set_creator(ALICE).build().execute_with(|| {
		assert_noop!(
			NftStake::create(RuntimeOrigin::signed(ALICE), CONTRACT_ID, Contract::default()),
			Error::<Test>::UnknownContractCollection
		);
	});
}

#[test]
fn rejects_incorrect_activation() {
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.balances(vec![(ALICE, ItemDeposit::get())])
		.build()
		.execute_with(|| {
			let (now, activation) = (5, 2);
			assert!(activation < now);
			run_to_block(now);
			assert_noop!(
				NftStake::create(
					RuntimeOrigin::signed(ALICE),
					CONTRACT_ID,
					Contract::default().activation(activation)
				),
				Error::<Test>::IncorrectActivation
			);
		});
}

#[test]
fn rejects_incorrect_active_duration() {
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.balances(vec![(ALICE, ItemDeposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::create(
					RuntimeOrigin::signed(ALICE),
					CONTRACT_ID,
					Contract::default().active_duration(0)
				),
				Error::<Test>::ZeroActiveDuration
			);
		});
}

#[test]
fn rejects_incorrect_claim_duration() {
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.balances(vec![(ALICE, ItemDeposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::create(
					RuntimeOrigin::signed(ALICE),
					CONTRACT_ID,
					Contract::default().claim_duration(0)
				),
				Error::<Test>::ZeroClaimDuration
			);
		});
}
