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
fn works() {
	let stake_clauses = vec![
		(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 4)),
		(1, Clause::HasAttribute(RESERVED_COLLECTION_1, 5)),
		(2, Clause::HasAttributeWithValue(RESERVED_COLLECTION_2, 6, 7)),
	];
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 123))];
	let stake_duration = 4;
	let contract = Contract::default()
		.reward(Reward::Tokens(123))
		.stake_duration(stake_duration)
		.stake_amt(3)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.fee_amt(1)
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	let stake_addresses = stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	let fees = MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));
	let fee_addresses = fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract.clone())
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
			for NftId(collection_id, item_id) in stake_addresses.clone() {
				assert_eq!(Nft::owner(collection_id, item_id), Some(NftStake::account_id()));
			}
			assert_eq!(
				ContractStakedItems::<Test>::get(contract_id),
				Some(stake_addresses.try_into().unwrap())
			);

			// Fee ownership transferred to creator
			for NftId(collection_id, item_id) in fee_addresses.clone() {
				assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
			}

			// Check contract accepted block and holder
			let current_block = <frame_system::Pallet<Test>>::block_number();
			assert_eq!(ContractAccepted::<Test>::get(contract_id), Some(current_block));
			assert_eq!(Contracts::<Test>::get(contract_id), Some(contract.activation(1)));
			assert_eq!(ContractIds::<Test>::get(BOB).unwrap().to_vec(), vec![contract_id]);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Accepted {
				by: BOB,
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
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(
			contract_id,
			Contract::default()
				.stake_amt(MaxStakingClauses::get() as u8)
				.fee_amt(0)
				.stake_clauses(
					AttributeNamespace::Pallet,
					(0..MaxStakingClauses::get())
						.map(|i| (i as u8, MockClause::HasAttribute(0, 0)))
						.collect::<Vec<_>>(),
				),
		)
		.build()
		.execute_with(|| {
			let stake_addresses = (0..MaxStakingClauses::get() + 1)
				.map(|_| NftId(RESERVED_COLLECTION_0, H256::random()))
				.collect::<Vec<_>>();
			assert!(stake_addresses.len() as u32 > MaxStakingClauses::get());
			assert_noop!(
				NftStake::accept(
					RuntimeOrigin::signed(ALICE),
					contract_id,
					stake_addresses,
					Default::default()
				),
				Error::<Test>::InvalidNFTStakeAmount
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
	let stake_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 2))];
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 123))];
	let contract = Contract::default()
		.reward(Reward::Tokens(123))
		.stake_duration(123)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone());
	let contract_id = H256::random();

	let alice_stakes =
		MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	let bob_stakes = alice_stakes
		.clone()
		.into_iter()
		.map(|(NftId(collection_id, _item_id), key, value)| {
			(NftId(collection_id, H256::random()), key, value)
		})
		.collect::<Vec<_>>();
	let charlie_stakes = bob_stakes
		.clone()
		.into_iter()
		.map(|(NftId(collection_id, _item_id), key, value)| {
			(NftId(collection_id, H256::random()), key, value)
		})
		.collect::<Vec<_>>();

	let alice_stake_addresses =
		alice_stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();
	let bob_stake_addresses =
		bob_stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();
	let charlie_stake_addresses =
		charlie_stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	let alice_fees =
		MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));
	let bob_fees = alice_fees
		.clone()
		.into_iter()
		.map(|(NftId(collection_id, _item_id), key, value)| {
			(NftId(collection_id, H256::random()), key, value)
		})
		.collect::<Vec<_>>();
	let charlie_fees = bob_fees
		.clone()
		.into_iter()
		.map(|(NftId(collection_id, _item_id), key, value)| {
			(NftId(collection_id, H256::random()), key, value)
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
		.create_contract_with_funds(contract_id, contract)
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

			for (staker, stake_addr, fee_addr) in [
				(ALICE, alice_stake_addresses, alice_fee_addresses),
				(BOB, bob_stake_addresses, bob_fee_addresses),
				(CHARLIE, charlie_stake_addresses, charlie_fee_addresses),
			] {
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(staker),
						contract_id,
						stake_addr,
						fee_addr
					),
					Error::<Test>::ContractOwnership
				);
			}
		});
}

#[test]
fn rejects_unowned_stakes() {
	let stake_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 2))];
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 2))];
	let contract = Contract::default()
		.reward(Reward::Tokens(123))
		.stake_duration(123)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone());
	let contract_id = H256::random();

	let stakes = MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	let stake_addresses = stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	let fees = MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));
	let fee_addresses = fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
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
		(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 123)),
		(1, Clause::HasAttribute(RESERVED_COLLECTION_0, 456)),
		(2, Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 12, 34)),
		(3, Clause::HasAttributeWithValue(RESERVED_COLLECTION_1, 56, 78)),
	];
	let contract = Contract::default()
		.reward(Reward::Tokens(123))
		.stake_duration(123)
		.stake_clauses(AttributeNamespace::Pallet, stake_clauses.clone())
		.stake_amt(4)
		.fee_amt(0);
	let contract_id = H256::random();

	let mut stakes =
		MockMints::from(MockClauses(stake_clauses.into_iter().map(|(_, c)| c).collect()));
	stakes.iter_mut().for_each(|(_, key, value)| {
		*key += 1;
		*value += 1;
	});
	let stake_addresses = stakes.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
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
	let fee_clauses = vec![(0, Clause::HasAttribute(RESERVED_COLLECTION_0, 123))];
	let contract = Contract::default()
		.reward(Reward::Tokens(123))
		.stake_duration(123)
		.fee_clauses(AttributeNamespace::Pallet, fee_clauses.clone())
		.stake_amt(0);
	let contract_id = H256::random();

	let mut fees = MockMints::from(MockClauses(fee_clauses.into_iter().map(|(_, c)| c).collect()));
	fees.iter_mut().for_each(|(_, key, value)| {
		*key += 1;
		*value += 1;
	});
	let fee_addresses = fees.clone().into_iter().map(|(addr, _, _)| addr).collect::<Vec<_>>();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
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

#[test]
fn rejects_unknown_activation() {
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default().stake_amt(0).fee_amt(0))
		.build()
		.execute_with(|| {
			// NOTE: technically all contracts should have activation set via `create`
			Contracts::<Test>::mutate(contract_id, |contract| {
				contract.as_mut().unwrap().activation = None
			});

			assert_noop!(
				NftStake::accept(
					RuntimeOrigin::signed(BOB),
					contract_id,
					Default::default(),
					Default::default()
				),
				Error::<Test>::UnknownActivation
			);
		});
}

#[test]
fn rejects_inactive_contracts() {
	let activation = 3;
	let active_duration = 2;
	let contract = Contract::default()
		.activation(activation)
		.active_duration(active_duration)
		.stake_amt(0)
		.fee_amt(0);
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
		.build()
		.execute_with(|| {
			// Before activation.
			for n in 0..activation {
				run_to_block(n);
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(BOB),
						contract_id,
						Default::default(),
						Default::default()
					),
					Error::<Test>::Inactive
				);
			}

			// After active duration.
			let end_of_active = activation + active_duration;
			for n in (end_of_active + 1)..(end_of_active + 3) {
				run_to_block(n);
				assert_noop!(
					NftStake::accept(
						RuntimeOrigin::signed(BOB),
						contract_id,
						Default::default(),
						Default::default()
					),
					Error::<Test>::Inactive
				);
			}

			// Regression for happy case, after activation.
			System::set_block_number(activation + 1);
			assert_ok!(NftStake::accept(
				RuntimeOrigin::signed(BOB),
				contract_id,
				Default::default(),
				Default::default()
			));
		});
}
