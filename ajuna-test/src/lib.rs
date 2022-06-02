use crate::sidechain::SigningKey;
use ajuna_solo_runtime::AccountId;

mod ajuna_node;
mod keyring;
mod sidechain;
mod traits;

// Some useful accounts
pub const SIDECHAIN_SIGNING_KEY: [u8; 32] = [0x1; 32];
pub const SUDO: [u8; 32] = [0x2; 32];
pub const PLAYER_1: [u8; 32] = [0x3; 32];
pub const PLAYER_2: [u8; 32] = [0x4; 32];

struct SideChainSigningKey;

impl SigningKey for SideChainSigningKey {
	fn account() -> AccountId {
		SIDECHAIN_SIGNING_KEY.into()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		ajuna_node::AjunaNode,
		sidechain::{AjunaBoard, Guess, SideChain},
		traits::{BlockProcessing, RuntimeBuilding},
	};
	use ajuna_solo_runtime::GameRegistry;
	use frame_support::assert_ok;

	struct Network {}

	impl Network {
		pub fn process(number_of_blocks: u8) {
			for _ in 0..number_of_blocks {
				// Produce a block at the node
				AjunaNode::move_forward();
				// Produce a sidechain block
				SideChain::<SideChainSigningKey>::move_forward();
			}
		}
	}

	struct Player {
		account_id: AccountId,
	}

	impl Player {
		pub fn queue(&self) {
			assert_ok!(GameRegistry::queue(ajuna_solo_runtime::Origin::signed(
				self.account_id.clone()
			)));
		}
		pub fn play_turn(&self, guess: Guess) {
			assert_ok!(AjunaBoard::play_turn(
				crate::sidechain::Origin::signed(self.account_id.clone()),
				crate::sidechain::PlayerTurn(guess)
			));
		}
	}

	#[test]
	fn play_a_guessing_game() {
		SideChain::<SideChainSigningKey>::build().execute_with(|| {
			AjunaNode::default()
				.account(SUDO.into())
				.sidechain(SIDECHAIN_SIGNING_KEY.into())
				.build()
				.execute_with(|| {
					// Queue players
					let player_1 = Player { account_id: PLAYER_1.into() };
					let player_2 = Player { account_id: PLAYER_2.into() };
					player_1.queue();
					assert!(GameRegistry::queued().is_none());
					player_2.queue();
					assert!(GameRegistry::queued().is_some());
					// We want to move forward by one block so the sidechain imports
					Network::process(1);
					// Game would be acknowledged by sidechain
					assert!(GameRegistry::queued().is_none());
					// Game should be created now and we can play
					player_1.play_turn(100);
					player_2.play_turn(101);
				});
		});
	}
}

#[cfg(test)]
mod vesting_tests {
	use super::*;
	use frame_support::assert_ok;

	use ajuna_solo_runtime::currency::*;
	use orml_vesting::{VestingSchedule, VESTING_LOCK_ID};
	use pallet_balances::{BalanceLock, Reasons};

	use crate::{keyring::*, sidechain::*};

	#[test]
	fn validate_simple_vesting_schedule() {
		SideChain::<SideChainSigningKey>::build().execute_with(|| {
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
				bob_id.clone(),
				//sp_runtime::MultiAddress::Id(bob_id.clone()),
				vesting_schedule.clone()
			));
			assert_eq!(Vesting::vesting_schedules(&bob_id), vec![vesting_schedule.clone()]);
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
		SideChain::<SideChainSigningKey>::build().execute_with(|| {
			let alice_id = alice();
			let bob_id = bob();

			let cliff_vesting_schedule = VestingSchedule {
				start: 10,
				period: 100,
				period_count: 1,
				per_period: 100 * MICRO_AJUNS,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id.clone()),
				bob_id.clone(),
				cliff_vesting_schedule.clone()
			));

			let slope_vesting_schedule = VestingSchedule {
				start: 110,
				period: 1,
				period_count: 2000,
				per_period: 450 * NANO_AJUNS,
			};

			assert_ok!(Vesting::vested_transfer(
				Origin::signed(alice_id.clone()),
				bob_id.clone(),
				slope_vesting_schedule.clone()
			));

			System::set_block_number(10);

			assert_ok!(Vesting::claim_for(Origin::signed(alice_id.clone()), bob_id.clone()));

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

			assert_ok!(Vesting::claim_for(Origin::signed(alice_id.clone()), bob_id.clone()));

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

			assert_ok!(Vesting::claim_for(Origin::signed(alice_id.clone()), bob_id.clone()));

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
				assert_ok!(Vesting::claim_for(Origin::signed(alice_id.clone()), bob_id.clone()));

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

			assert_ok!(Vesting::claim_for(Origin::signed(alice_id.clone()), bob_id.clone()));

			assert_eq!(Balances::locks(bob_id).get(0), None);
		});
	}
}
