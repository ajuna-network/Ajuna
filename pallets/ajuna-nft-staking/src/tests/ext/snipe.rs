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
	let stake_clauses = vec![
		(0, Clause::HasAttribute(RESERVED_COLLECTION_0, bounded_vec![4])),
		(1, Clause::HasAttribute(RESERVED_COLLECTION_1, bounded_vec![5])),
		(2, Clause::HasAttributeWithValue(RESERVED_COLLECTION_2, bounded_vec![6], bounded_vec![7])),
	];
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_2, bounded_vec![33]))];
	let (stake_duration, claim_duration) = (4, 3);
	let reward = 135;
	let contract = Contract::default()
		.rewards(bounded_vec![Reward::Tokens(reward)])
		.stake_duration(stake_duration)
		.claim_duration(claim_duration)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	let stake_addresses =
		stakes.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
	let fees = MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));
	let fee_addresses = fees.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract.clone())
		.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
		.create_sniper(CHARLIE, Contract::default().stake_duration(10).claim_duration(20))
		.build()
		.execute_with(|| {
			let initial_balance_bob = Balances::free_balance(BOB);
			let initial_balance_charlie = Balances::free_balance(CHARLIE);

			let accepted_at = ContractAccepted::<Test>::get(contract_id).unwrap();
			run_to_block(accepted_at + stake_duration + claim_duration + 1);

			assert_ok!(NftStake::snipe(RuntimeOrigin::signed(CHARLIE), contract_id));

			for NftId(collection_id, item_id) in stake_addresses {
				assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
			}
			for NftId(collection_id, item_id) in fee_addresses {
				assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
			}
			assert_eq!(Balances::free_balance(BOB), initial_balance_bob);
			assert_eq!(Balances::free_balance(CHARLIE), initial_balance_charlie + reward);

			let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
			assert_eq!(Nft::owner(contract_collection_id, contract_id), None);
			assert_eq!(ContractAccepted::<Test>::get(contract_id), None);
			assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);
			assert_eq!(ContractIds::<Test>::get(BOB), None);

			// Check stats
			assert_eq!(ContractsStats::<Test>::get(CHARLIE).contracts_sniped, 1);

			// Check stats
			assert_eq!(ContractsStats::<Test>::get(BOB).contracts_lost, 1);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Sniped {
				by: CHARLIE,
				contract_id,
				rewards: contract.rewards,
			}));
		});
}

#[test]
fn works_with_nft_reward() {
	let stake_clauses = vec![
		(0, Clause::HasAttribute(RESERVED_COLLECTION_0, bounded_vec![4])),
		(1, Clause::HasAttribute(RESERVED_COLLECTION_0, bounded_vec![5])),
		(2, Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, bounded_vec![6], bounded_vec![7])),
	];
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_1, bounded_vec![12]))];
	let (stake_duration, claim_duration) = (2, 3);
	let reward_addr = NftId(RESERVED_COLLECTION_2, H256::random());
	let contract = Contract::default()
		.rewards(bounded_vec![Reward::Nft(reward_addr.clone())])
		.stake_duration(stake_duration)
		.claim_duration(claim_duration)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	let stake_addresses =
		stakes.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
	let fees = MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));
	let fee_addresses = fees.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract.clone())
		.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
		.create_sniper(CHARLIE, Contract::default().stake_duration(10).claim_duration(20))
		.build()
		.execute_with(|| {
			let accepted_at = ContractAccepted::<Test>::get(contract_id).unwrap();
			run_to_block(accepted_at + stake_duration + claim_duration + 1);

			assert_ok!(NftStake::snipe(RuntimeOrigin::signed(CHARLIE), contract_id));

			for NftId(collection_id, item_id) in stake_addresses {
				assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
			}
			for NftId(collection_id, item_id) in fee_addresses {
				assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
			}
			assert_eq!(Nft::owner(reward_addr.0, reward_addr.1), Some(CHARLIE));

			let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
			assert_eq!(Nft::owner(contract_collection_id, contract_id), None);
			assert_eq!(ContractAccepted::<Test>::get(contract_id), None);
			assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);
			assert_eq!(ContractIds::<Test>::get(BOB), None);

			// Check stats
			assert_eq!(ContractsStats::<Test>::get(CHARLIE).contracts_sniped, 1);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Sniped {
				by: CHARLIE,
				contract_id,
				rewards: contract.rewards,
			}));
		});
}

#[test]
fn rejects_unsigned_calls() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NftStake::snipe(RuntimeOrigin::none(), Default::default()),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn rejects_when_pallet_is_locked() {
	ExtBuilder::default().build().execute_with(|| {
		GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
		assert_noop!(
			NftStake::snipe(RuntimeOrigin::signed(ALICE), Default::default()),
			Error::<Test>::PalletLocked
		);
	});
}

#[test]
fn rejects_non_sniper_who_holds_no_contracts() {
	let contract_id = H256::random();
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
		.build()
		.execute_with(|| {
			assert_eq!(ContractIds::<Test>::get(BOB), None);
			assert_noop!(
				NftStake::snipe(RuntimeOrigin::signed(BOB), contract_id),
				Error::<Test>::NotSniper
			);
		});
}

#[test]
fn rejects_non_sniper_who_holds_no_contracts_being_staked() {
	let contract_id = H256::random();
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
		.accept_contract(
			vec![(BOB, Default::default())],
			vec![(BOB, Default::default())],
			contract_id,
			BOB,
		)
		.create_sniper(CHARLIE, Contract::default())
		.build()
		.execute_with(|| {
			let (stake_duration, claim_duration) = (1, 1);
			let sniper_contract_id = ContractIds::<Test>::get(CHARLIE).unwrap()[0];
			Contracts::<Test>::mutate(sniper_contract_id, |contract| {
				let contract = contract.as_mut().unwrap();
				contract.stake_duration = stake_duration;
				contract.claim_duration = claim_duration;
			});

			// After contract expires.
			run_to_block(stake_duration + claim_duration + 1);
			assert_noop!(
				NftStake::snipe(RuntimeOrigin::signed(CHARLIE), contract_id),
				Error::<Test>::NotSniper
			);
		});
}

#[test]
fn rejects_when_contract_is_not_accepted() {
	let contract_id = H256::random();
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
		.create_sniper(BOB, Contract::default())
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::snipe(RuntimeOrigin::signed(BOB), contract_id),
				Error::<Test>::Available
			);
		});
}

#[test]
fn rejects_when_sniping_own_contract() {
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
		.accept_contract(
			vec![(BOB, Default::default())],
			vec![(BOB, Default::default())],
			contract_id,
			BOB,
		)
		.create_sniper(BOB, Contract::default())
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::snipe(RuntimeOrigin::signed(BOB), contract_id),
				Error::<Test>::CannotSnipeOwnContract
			);
		});
}

#[test]
fn rejects_when_contract_is_staking() {
	let (stake_duration, claim_duration) = (3, 5);
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(
			contract_id,
			Contract::default()
				.stake_duration(stake_duration)
				.claim_duration(claim_duration),
		)
		.accept_contract(
			vec![(BOB, Default::default())],
			vec![(BOB, Default::default())],
			contract_id,
			BOB,
		)
		.create_sniper(CHARLIE, Contract::default().stake_duration(10).claim_duration(10))
		.build()
		.execute_with(|| {
			let accepted_at = ContractAccepted::<Test>::get(contract_id).unwrap();

			// During staking.
			for n in 0..stake_duration {
				run_to_block(accepted_at + n);
				assert_noop!(
					NftStake::snipe(RuntimeOrigin::signed(CHARLIE), contract_id),
					Error::<Test>::Claimable
				);
			}

			// Regression for happy case, after expiry.
			System::set_block_number(accepted_at + stake_duration + claim_duration + 1);
			assert_ok!(NftStake::snipe(RuntimeOrigin::signed(CHARLIE), contract_id));
		});
}

#[test]
fn snipe_with_maliciously_burned_contract_nft() {
	let stake_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, bounded_vec![4]))];
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_2, bounded_vec![33]))];
	let (stake_duration, claim_duration) = (4, 3);
	let reward = 135;
	let contract = Contract::default()
		.rewards(bounded_vec![Reward::Tokens(reward)])
		.stake_duration(stake_duration)
		.claim_duration(claim_duration)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	let fees = MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract.clone())
		.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
		.create_sniper(CHARLIE, Contract::default().stake_duration(10).claim_duration(20))
		.build()
		.execute_with(|| {
			let accepted_at = ContractAccepted::<Test>::get(contract_id).unwrap();
			run_to_block(accepted_at + stake_duration + claim_duration + 1);

			let contract_collection =
				ContractCollectionId::<Test>::get().expect("Should get collection");
			assert_ok!(pallet_nfts::Pallet::<Test>::burn(
				RuntimeOrigin::signed(BOB),
				contract_collection,
				contract_id
			));

			assert_ok!(NftStake::snipe(RuntimeOrigin::signed(CHARLIE), contract_id));

			let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();
			assert_eq!(Nft::owner(contract_collection_id, contract_id), None);
			assert_eq!(ContractAccepted::<Test>::get(contract_id), None);
			assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);
			assert_eq!(ContractIds::<Test>::get(BOB), Some(bounded_vec![contract_id]));

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Sniped {
				by: CHARLIE,
				contract_id,
				rewards: contract.rewards,
			}));
		});
}
