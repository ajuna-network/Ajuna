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
	let contract_id = CONTRACT_ID;

	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(contract_id, Contract::default())
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
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(CONTRACT_ID, Contract::default())
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(BOB), CONTRACT_ID),
				DispatchError::BadOrigin
			);
		});
}

#[test]
fn rejects_when_pallet_is_locked() {
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(CONTRACT_ID, Contract::default())
		.build()
		.execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.pallet_locked = true);
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(ALICE), CONTRACT_ID),
				Error::<Test>::PalletLocked
			);
		});
}

#[test]
fn rejects_accepted_contracts() {
	ExtBuilder::default()
		.set_creator(ALICE)
		.create_contract_collection()
		.create_contract_with_funds(CONTRACT_ID, Contract::default())
		.accept_contract(
			vec![(BOB, Default::default())],
			vec![(BOB, Default::default())],
			CONTRACT_ID,
			BOB,
		)
		.build()
		.execute_with(|| {
			assert_noop!(
				NftStake::remove(RuntimeOrigin::signed(ALICE), CONTRACT_ID),
				Error::<Test>::Staking
			);
		});
}
