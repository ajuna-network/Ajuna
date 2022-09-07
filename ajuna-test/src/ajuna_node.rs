use crate::{
	impl_block_numbers,
	keyring::*,
	traits::{BlockProcessing, RuntimeBuilding},
};
use ajuna_solo_runtime::{AccountId, Balance, BlockNumber, Runtime, System};
use sp_runtime::{BoundedVec, Storage};

pub struct AjunaNode {
	/// The account owning the node(sudo)
	account_id: AccountId,
	sidechain: AccountId,
}

use ajuna_solo_runtime::{currency::NANO_AJUNS, ObserversConfig, SudoConfig};
use sp_runtime::BuildStorage;

const EXISTENTIAL_DEPOSIT: Balance = 100 * NANO_AJUNS;

impl_block_numbers!(System, BlockNumber);
impl RuntimeBuilding<Runtime, BlockNumber, RuntimeBlocks> for AjunaNode {
	fn configure_storages(&self, storage: &mut Storage) {
		ajuna_solo_runtime::GenesisConfig {
			sudo: SudoConfig { key: Some(self.account_id.clone()) },
			observers: ObserversConfig {
				members: BoundedVec::try_from(vec![self.sidechain.clone()]).unwrap(),
				..Default::default()
			},
			balances: pallet_balances::GenesisConfig {
				balances: vec![
					(alice(), 10_000 * EXISTENTIAL_DEPOSIT),
					(bob(), 20_000 * EXISTENTIAL_DEPOSIT),
					(charlie(), 30_000 * EXISTENTIAL_DEPOSIT),
					(dave(), 40_000 * EXISTENTIAL_DEPOSIT),
					(eve(), 10_000 * EXISTENTIAL_DEPOSIT),
					(ferdie(), 9_999_000 * EXISTENTIAL_DEPOSIT),
				],
			},
			vesting: orml_vesting::GenesisConfig { vesting: vec![] },
			..Default::default()
		}
		.assimilate_storage(storage)
		.unwrap();
	}
}

impl Default for AjunaNode {
	fn default() -> Self {
		Self { account_id: [0x0; 32].into(), sidechain: [0x0; 32].into() }
	}
}

#[cfg(test)]
impl AjunaNode {
	pub fn account(mut self, account_id: AccountId) -> Self {
		self.account_id = account_id;
		self
	}

	pub fn sidechain(mut self, sidechain: AccountId) -> Self {
		self.sidechain = sidechain;
		self
	}

	pub fn build() -> sp_io::TestExternalities {
		let mut storage = Storage::default();

		Self::default().configure_storages(&mut storage);

		let mut ext: sp_io::TestExternalities = storage.build_storage().unwrap().into();

		ext.execute_with(|| {
			System::set_block_number(1);
		});

		ext
	}
}

impl BlockProcessing<BlockNumber, RuntimeBlocks> for AjunaNode {
	fn on_block() {}
}

#[cfg(test)]
mod vesting_tests {
	use super::*;
	use frame_support::assert_ok;

	use ajuna_solo_runtime::{currency::*, Balances, Origin, System, Vesting};
	use orml_vesting::{VestingSchedule, VESTING_LOCK_ID};
	use pallet_balances::{BalanceLock, Reasons};

	#[test]
	fn validate_simple_vesting_schedule() {
		AjunaNode::build().execute_with(|| {
			let alice_id = alice();
			let bob_id = bob();

			assert_eq!(
				Vesting::vesting_schedules(alice_id.clone()),
				vec![],
				"Alice's vesting should be empty"
			);

			let vesting_schedule = VestingSchedule {
				start: 10,
				period: 10,
				period_count: 5,
				per_period: 25 * MICRO_AJUNS,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id),
				sp_runtime::MultiAddress::Id(bob_id.clone()),
				vesting_schedule.clone()
			));
			assert_eq!(Vesting::vesting_schedules(&bob_id), vec![vesting_schedule]);
		});
	}

	/// This test demonstrates the work of the following vesting schedule:
	/// * Total issuance is 1000 µAJUNS
	/// * The vesting begins at block 10
	/// * There's an initial cliff period of 100 blocks
	/// * After the cliff 25% of the total issuance gets unlocked (100 µAJUNS)
	/// * Past that point the remaining issuance gets linearly unlocked during the following 2000
	///   blocks, which means that each block unlock 450 nAJUNS
	///
	/// In order to perform such scheme we need to define two chained VestingSchedule blocks
	#[test]
	fn validate_complex_vesting_schedule() {
		AjunaNode::build().execute_with(|| {
			let alice_id = alice();
			let bob_id = bob();
			let bob_multi_address = sp_runtime::MultiAddress::Id(bob_id.clone());

			let cliff_vesting_schedule = VestingSchedule {
				start: 10,
				period: 100,
				period_count: 1,
				per_period: 100 * MICRO_AJUNS,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id.clone()),
				bob_multi_address.clone(),
				cliff_vesting_schedule
			));

			let slope_vesting_schedule = VestingSchedule {
				start: 110,
				period: 1,
				period_count: 2000,
				per_period: 450 * NANO_AJUNS,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id.clone()),
				bob_multi_address.clone(),
				slope_vesting_schedule
			));

			System::set_block_number(10);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multi_address.clone()
			));

			// We assert that BOB has 1000 µAJUNS locked
			assert_eq!(
				Balances::locks(bob_id.clone()).get(0),
				Some(&BalanceLock {
					id: VESTING_LOCK_ID,
					amount: 1_000 * MICRO_AJUNS,
					reasons: Reasons::All,
				})
			);

			System::set_block_number(50);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multi_address.clone()
			));

			// We assert that BOB has still 1000 µAJUNS locked
			assert_eq!(
				Balances::locks(bob_id.clone()).get(0),
				Some(&BalanceLock {
					id: VESTING_LOCK_ID,
					amount: 1_000 * MICRO_AJUNS,
					reasons: Reasons::All,
				})
			);

			System::set_block_number(110);

			assert_ok!(Vesting::claim_for(
				Origin::signed(alice_id.clone()),
				bob_multi_address.clone()
			));

			// We assert that BOB has now only 900 µAJUNS locked
			assert_eq!(
				Balances::locks(bob_id.clone()).get(0),
				Some(&BalanceLock {
					id: VESTING_LOCK_ID,
					amount: 900 * MICRO_AJUNS,
					reasons: Reasons::All,
				})
			);

			// We validate that for each block starting at block 110, 450 µAJUNS become
			// claimable into BOB's account
			for i in (111..=2109).step_by(100) {
				System::set_block_number(i);
				assert_ok!(Vesting::claim_for(
					Origin::signed(alice_id.clone()),
					bob_multi_address.clone()
				));

				assert_eq!(
					Balances::locks(bob_id.clone()).get(0),
					Some(&BalanceLock {
						id: VESTING_LOCK_ID,
						amount: (900 * MICRO_AJUNS) - (450 * NANO_AJUNS * (i as u128 - 110)),
						reasons: Reasons::All,
					})
				);
			}

			// Finally at block 2110 all vested tokens are reclaimed and no more AJUNS are
			// locked in BOB's account
			System::set_block_number(2110);

			assert_ok!(Vesting::claim_for(Origin::signed(alice_id), bob_multi_address));

			assert_eq!(Balances::locks(bob_id).get(0), None);
		});
	}
}
