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

use frame_support::assert_ok;
use orml_vesting::{VestingSchedule, VESTING_LOCK_ID};
use pallet_balances::{BalanceLock, Reasons};

use crate::{keyring::*, mock::ExtBuilder};

use super::*;

#[test]
fn validate_genesis_vesting_schedules() {
	ExtBuilder::default()
		.existential_deposit(1_000 * BAJUN)
		.build()
		.execute_with(|| {
			let alice_free_balance = Balances::free_balance(alice());
			let bob_free_balance = Balances::free_balance(bob());
			let charlie_free_balance = Balances::free_balance(charlie());
			assert_eq!(alice_free_balance, 1_000 * BAJUN * 10_000); // Account 1 has free balance
			assert_eq!(bob_free_balance, 1_000 * BAJUN * 20_000); // Account 2 has free balance
			assert_eq!(charlie_free_balance, 1_000 * BAJUN * 30_000); // Account 12 has free balance

			let alice_vesting = VestingSchedule {
				start: 0,
				period: 1,
				period_count: 1_000,
				per_period: 5_000 * BAJUN,
			};
			assert_eq!(
				Vesting::vesting_schedules(alice()),
				vec![alice_vesting],
				"Alice's vesting should have expected values"
			);

			let bob_vesting = VestingSchedule {
				start: 10,
				period: 1,
				period_count: 2_000,
				per_period: 3_000 * BAJUN,
			};
			assert_eq!(
				Vesting::vesting_schedules(bob()),
				vec![bob_vesting],
				"Bob's vesting should have expected values"
			);

			let charlie_vesting = VestingSchedule {
				start: 10,
				period: 100,
				period_count: 2_000,
				per_period: 5_000 * BAJUN,
			};
			assert_eq!(
				Vesting::vesting_schedules(charlie()),
				vec![charlie_vesting],
				"Charlie's vesting should have expected values"
			);
		});
}

#[test]
fn validate_simple_vesting_schedule() {
	ExtBuilder::default()
		.existential_deposit(1000 * BAJUN)
		.vesting_genesis_config(vec![])
		.build()
		.execute_with(|| {
			let alice_id = alice();
			let bob_id = bob();

			assert_eq!(
				Vesting::vesting_schedules(alice_id.clone()),
				vec![],
				"Alice's vesting should be empty"
			);

			let vesting_schedule =
				VestingSchedule { start: 10, period: 10, period_count: 5, per_period: 15 * BAJUN };

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id),
				sp_runtime::MultiAddress::Id(bob_id.clone()),
				vesting_schedule.clone()
			));
			assert_eq!(Vesting::vesting_schedules(&bob_id), vec![vesting_schedule.clone()]);
		});
}

/// This test demonstrates the work of the following vesting schedule:
/// * Total issuance is 1000 BAJUN
/// * The vesting begins at block 10
/// * There's an initial cliff period of 100 blocks
/// * After the cliff 10% of the total issuance gets unlocked (100 BAJUN)
/// * Past that point the remaining issuance gets linearly unlocked during the following 1000
///   blocks, which means that each block unlock 900 mBAJUN
///
/// In order to perform such scheme we need to define two chained VestingSchedule blocks
#[test]
fn validate_complex_vesting_schedule() {
	ExtBuilder::default()
		.existential_deposit(1000 * EXISTENTIAL_DEPOSIT)
		.vesting_genesis_config(vec![])
		.build()
		.execute_with(|| {
			let alice_id = alice();
			let bob_id = bob();
			let bob_multiaddress = sp_runtime::MultiAddress::Id(bob_id.clone());

			let cliff_vesting_schedule = VestingSchedule {
				start: 10,
				period: 100,
				period_count: 1,
				per_period: 100 * BAJUN,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id.clone()),
				bob_multiaddress.clone(),
				cliff_vesting_schedule.clone()
			));

			let slope_vesting_schedule = VestingSchedule {
				start: 110,
				period: 1,
				period_count: 1000,
				per_period: 900 * MILLI_BAJUN,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id.clone()),
				bob_multiaddress.clone(),
				slope_vesting_schedule.clone()
			));

			System::set_block_number(10);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multiaddress.clone()
			));

			// We assert that BOB has 1000 BAJUN locked
			assert_eq!(
				Balances::locks(bob_id.clone()).get(0),
				Some(&BalanceLock {
					id: VESTING_LOCK_ID,
					amount: 1_000 * BAJUN,
					reasons: Reasons::All,
				})
			);

			System::set_block_number(50);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multiaddress.clone()
			));

			// We assert that BOB has still 1000 BAJUN locked
			assert_eq!(
				Balances::locks(bob_id.clone()).get(0),
				Some(&BalanceLock {
					id: VESTING_LOCK_ID,
					amount: 1_000 * BAJUN,
					reasons: Reasons::All,
				})
			);

			System::set_block_number(110);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multiaddress.clone()
			));

			// We assert that BOB has now only 900 BAJUN locked
			assert_eq!(
				Balances::locks(bob_id.clone()).get(0),
				Some(&BalanceLock {
					id: VESTING_LOCK_ID,
					amount: 900 * BAJUN,
					reasons: Reasons::All,
				})
			);
			// We validate that for each block starting at block 110, 900 mBAJUN become claimable
			// into BOB's account
			for i in (111..=1109).step_by(100) {
				System::set_block_number(i);
				assert_ok!(Vesting::claim_for(
					Origin::signed(alice_id.clone()),
					bob_multiaddress.clone()
				));

				assert_eq!(
					Balances::locks(bob_id.clone()).get(0),
					Some(&BalanceLock {
						id: VESTING_LOCK_ID,
						amount: (900 * BAJUN) - (900 * MILLI_BAJUN * (i as u128 - 110)),
						reasons: Reasons::All,
					})
				);
			}

			// Finally at block 2110 all vested tokens are reclaimed and no more BAJUN are locked
			// in BOB's account
			System::set_block_number(1110);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multiaddress.clone()
			));

			assert_eq!(Balances::locks(bob_id.clone()).get(0), None);
		});
}
