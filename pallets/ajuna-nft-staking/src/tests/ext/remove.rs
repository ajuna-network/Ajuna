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
	let contract_id = H256::random();
	let reward_amount = 1_000;

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract(
			contract_id,
			Contract::default().rewards(bounded_vec![Reward::Tokens(reward_amount)]),
		)
		.build()
		.execute_with(|| {
			assert!(Contracts::<Test>::get(contract_id).is_some());

			let contract_collection_id = ContractCollectionId::<Test>::get().unwrap();

			assert_eq!(
				Nft::owner(contract_collection_id, contract_id),
				Some(NftStake::account_id())
			);

			let creator_balance = Balances::free_balance(ALICE);

			assert_ok!(NftStake::remove(RuntimeOrigin::signed(ALICE), contract_id));

			assert_eq!(
				Balances::free_balance(ALICE),
				creator_balance + reward_amount + ItemDeposit::get()
			);

			assert_eq!(Contracts::<Test>::get(contract_id), None);
			assert_eq!(ContractsMetadata::<Test>::get(contract_id), None);

			assert_eq!(Nft::owner(contract_collection_id, contract_id), None);

			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Removed {
				contract_id,
			}));
		});
}

#[test]
fn rejects_non_creator_calls() {
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(BOB), contract_id),
				DispatchError::BadOrigin
			);
		});
}

#[test]
fn rejects_when_pallet_is_locked() {
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
		.build()
		.execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(ALICE), contract_id),
				Error::<Test>::PalletLocked
			);
		});
}

#[test]
fn rejects_accepted_contracts() {
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
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(ALICE), contract_id),
				Error::<Test>::Staking
			);
		});
}

#[test]
fn rejects_unknown_contracts() {
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(ALICE), H256::random()),
				Error::<Test>::UnknownContract
			);
		});
}
