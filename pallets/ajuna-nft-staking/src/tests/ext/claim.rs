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
		Clause::HasAttribute(RESERVED_COLLECTION_0, 4),
		Clause::HasAttribute(RESERVED_COLLECTION_1, 5),
		Clause::HasAttributeWithValue(RESERVED_COLLECTION_2, 6, 7),
	];
	let fee_clauses = vec![Clause::HasAttribute(RESERVED_COLLECTION_2, 33)];
	let stake_duration = 4;
	let reward_amount = 135;
	let contract = Contract::default()
		.reward(Reward::Tokens(reward_amount))
		.stake_duration(stake_duration)
		.stake_clauses(stake_clauses.clone())
		.fee_clauses(fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses));
	let stake_addresses =
		stakes.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
	let fees = MockMints::from(MockClauses(fee_clauses));
	let fee_addresses = fees.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
	let initial_balance = 333;

	ExtBuilder::default()
		.set_creator(ALICE)
		.balances(vec![(BOB, initial_balance)])
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract.clone())
		.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
		.build()
		.execute_with(|| {
			run_to_block(System::block_number() + stake_duration);

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
			assert_eq!(ContractAccepted::<Test>::get(contract_id), None);
			assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Claimed {
				by: BOB,
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
	let stake_duration = 8;
	let reward_addr = NftAddress(RESERVED_COLLECTION_2, H256::random());
	let contract = Contract::default()
		.reward(Reward::Nft(reward_addr.clone()))
		.stake_duration(stake_duration)
		.stake_clauses(stake_clauses.clone())
		.fee_clauses(fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses));
	let stake_addresses =
		stakes.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();
	let fees = MockMints::from(MockClauses(fee_clauses));
	let fee_addresses = fees.clone().into_iter().map(|(address, _, _)| address).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract.clone())
		.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
		.build()
		.execute_with(|| {
			run_to_block(System::block_number() + stake_duration);

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
			assert_eq!(ContractAccepted::<Test>::get(contract_id), None);
			assert_eq!(ContractStakedItems::<Test>::get(contract_id), None);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Claimed {
				by: BOB,
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
	let contract_id = H256::random();
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
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
	let stake_duration = 3;
	let contract = Contract::default()
		.reward(Reward::Tokens(321))
		.stake_duration(stake_duration)
		.stake_clauses(stake_clauses.clone())
		.fee_clauses(fee_clauses.clone());
	let contract_id = H256::random();
	let stakes = MockMints::from(MockClauses(stake_clauses));
	let fees = MockMints::from(MockClauses(fee_clauses));

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
		.accept_contract(vec![(BOB, stakes)], vec![(BOB, fees)], contract_id, BOB)
		.build()
		.execute_with(|| {
			for _ in 0..(stake_duration - 1) {
				run_to_block(System::block_number() + 1);
				assert_noop!(
					NftStake::claim(RuntimeOrigin::signed(BOB), contract_id),
					Error::<Test>::ContractStillActive
				);
			}
		});
}
