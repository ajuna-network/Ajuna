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
	let contract = Contract::default().reward(Reward::Tokens(123));
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
		.build()
		.execute_with(|| {
			assert!(Contracts::<Test>::get(contract_id).is_some());
			assert_ok!(NftStake::remove(RuntimeOrigin::signed(ALICE), contract_id));
			assert!(Contracts::<Test>::get(contract_id).is_none());
			System::assert_last_event(RuntimeEvent::NftStake(crate::Event::Removed {
				contract_id,
			}));
		});
}

#[test]
fn rejects_non_creator_calls() {
	let contract = Contract::default().reward(Reward::Tokens(123));
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
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
	let contract = Contract::default().reward(Reward::Tokens(123));
	let contract_id = H256::random();

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, contract)
		.build()
		.execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(ALICE), contract_id),
				Error::<Test>::PalletLocked
			);
		});
}
