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

use crate::{mock::*, types::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{testing::H256, ArithmeticError, DispatchError};

fn create_avatars(season_id: SeasonId, account: MockAccountId, n: u8) -> Vec<AvatarIdOf<Test>> {
	(0..n)
		.into_iter()
		.map(|i| {
			let avatar = Avatar::default().season_id(season_id).dna(&[i; 32]);
			let avatar_id = H256::random();
			Avatars::<Test>::insert(avatar_id, (account, avatar));
			Owners::<Test>::try_append(account, avatar_id).unwrap();
			avatar_id
		})
		.collect()
}

mod pallet_accounts {
	use super::*;

	#[test]
	fn treasury_account_id_works() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(
				AAvatars::treasury_account_id(),
				(b"modl", AwesomeAvatarsPalletId::get(), b"treasury")
					.using_encoded(|x| MockAccountId::decode(&mut TrailingZeroInput::new(x)))
					.unwrap()
			)
		});
	}

	#[test]
	fn technical_account_id_works() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(
				AAvatars::technical_account_id(),
				(b"modl", AwesomeAvatarsPalletId::get(), b"technical")
					.using_encoded(|x| MockAccountId::decode(&mut TrailingZeroInput::new(x)))
					.unwrap()
			)
		});
	}
}

mod organizer {
	use super::*;

	#[test]
	fn set_organizer_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(AAvatars::organizer(), None);
			assert_ok!(AAvatars::set_organizer(RuntimeOrigin::root(), ALICE));
			assert_eq!(AAvatars::organizer(), Some(ALICE));
			System::assert_last_event(mock::RuntimeEvent::AAvatars(crate::Event::OrganizerSet {
				organizer: ALICE,
			}));
		});
	}

	#[test]
	fn set_organizer_should_reject_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::set_organizer(RuntimeOrigin::signed(ALICE), BOB),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_existing_organizer() {
		ExtBuilder::default().organizer(BOB).build().execute_with(|| {
			assert_ok!(AAvatars::set_organizer(RuntimeOrigin::root(), DAVE));
			assert_eq!(AAvatars::organizer(), Some(DAVE));
			System::assert_last_event(mock::RuntimeEvent::AAvatars(crate::Event::OrganizerSet {
				organizer: DAVE,
			}));
		});
	}

	#[test]
	fn ensure_organizer_should_reject_whe_no_organizer_is_set() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(AAvatars::organizer(), None);
			assert_noop!(
				AAvatars::ensure_organizer(RuntimeOrigin::signed(DAVE)),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::ensure_organizer(RuntimeOrigin::signed(DAVE)),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		ExtBuilder::default().organizer(CHARLIE).build().execute_with(|| {
			assert_ok!(AAvatars::ensure_organizer(RuntimeOrigin::signed(CHARLIE)));
		});
	}
}

mod treasury {
	use super::*;

	#[test]
	fn set_treasurer_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(AAvatars::treasurer(123), None);
			assert_ok!(AAvatars::set_treasurer(RuntimeOrigin::root(), 123, CHARLIE));
			assert_eq!(AAvatars::treasurer(123), Some(CHARLIE));
			System::assert_last_event(mock::RuntimeEvent::AAvatars(crate::Event::TreasurerSet {
				season_id: 123,
				treasurer: CHARLIE,
			}));
		});
	}

	#[test]
	fn set_treasurer_should_reject_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::set_treasurer(RuntimeOrigin::signed(ALICE), 1, CHARLIE),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn set_treasurer_should_replace_existing_treasurer() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(AAvatars::set_treasurer(RuntimeOrigin::root(), 333, ALICE));
			assert_ok!(AAvatars::set_treasurer(RuntimeOrigin::root(), 333, BOB));
			assert_eq!(AAvatars::treasurer(333), Some(BOB));
			System::assert_last_event(mock::RuntimeEvent::AAvatars(crate::Event::TreasurerSet {
				season_id: 333,
				treasurer: BOB,
			}));
		});
	}

	fn deposit_into_treasury(season_id: SeasonId, amount: MockBalance) {
		Treasury::<Test>::insert(season_id, amount);
		let _ = Balances::deposit_creating(&AAvatars::treasury_account_id(), amount);
	}

	#[test]
	fn claim_treasury_works() {
		let season_1 = Season::default().early_start(5).start(10).end(15);
		let mint_fees = MintFees { one: 12, three: 34, six: 56 };
		let initial_balance = MockExistentialDeposit::get() + 999_999;
		let total_supply = initial_balance;
		ExtBuilder::default()
			.seasons(&[(1, season_1.clone())])
			.mint_fees(mint_fees)
			.balances(&[(BOB, initial_balance)])
			.build()
			.execute_with(|| {
				let treasury_account = AAvatars::treasury_account_id();
				Treasurer::<Test>::insert(1, BOB);
				assert_eq!(AAvatars::treasury(1), 0);
				assert_eq!(Balances::total_balance(&BOB), initial_balance);
				assert_eq!(Balances::free_balance(&treasury_account), 0);
				assert_eq!(Balances::total_issuance(), total_supply);

				deposit_into_treasury(1, 333);
				assert_eq!(AAvatars::treasury(1), 333);
				assert_eq!(Balances::free_balance(&treasury_account), 333);
				assert_noop!(
					AAvatars::claim_treasury(RuntimeOrigin::signed(BOB), 1),
					Error::<Test>::CannotClaimDuringSeason
				);

				run_to_block(season_1.end + 1);
				assert_ok!(AAvatars::claim_treasury(RuntimeOrigin::signed(BOB), 1));
				assert_eq!(AAvatars::treasury(1), 0);
				assert_eq!(Balances::total_balance(&BOB), initial_balance + 333);
				assert_eq!(Balances::free_balance(&treasury_account), 0);
				assert_eq!(Balances::total_issuance(), total_supply + 333); // total supply increases from injection
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::TreasuryClaimed { season_id: 1, treasurer: BOB, amount: 333 },
				));
			})
	}

	#[test]
	fn claim_treasury_rejects_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::claim_treasury(RuntimeOrigin::none(), 123),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn claim_treasury_rejects_unknown_treasurer() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::claim_treasury(RuntimeOrigin::signed(ALICE), 1),
				Error::<Test>::UnknownTreasurer
			);
		})
	}

	#[test]
	fn claim_treasury_rejects_non_treasurer_calls() {
		ExtBuilder::default().build().execute_with(|| {
			Treasurer::<Test>::insert(3, BOB);
			assert_noop!(
				AAvatars::claim_treasury(RuntimeOrigin::signed(CHARLIE), 3),
				DispatchError::BadOrigin
			);
		})
	}

	#[test]
	fn claim_treasury_rejects_during_season() {
		let season_1 = Season::default().early_start(10).start(15).end(20);
		let season_2 = Season::default().early_start(25).start(30).end(35);
		ExtBuilder::default()
			.seasons(&[(1, season_1.clone()), (2, season_2.clone())])
			.balances(&[
				(ALICE, MockExistentialDeposit::get()),
				(BOB, MockExistentialDeposit::get()),
				(CHARLIE, MockExistentialDeposit::get()),
				(DAVE, MockExistentialDeposit::get()),
			])
			.build()
			.execute_with(|| {
				Treasurer::<Test>::insert(1, ALICE);
				Treasurer::<Test>::insert(2, BOB);
				Treasurer::<Test>::insert(3, CHARLIE);
				Treasurer::<Test>::insert(4, DAVE);

				// before season 1
				for (treasurer, season_id) in [(ALICE, 1), (BOB, 2), (CHARLIE, 3)] {
					for n in 0..season_1.early_start {
						run_to_block(n);
						assert_noop!(
							AAvatars::claim_treasury(RuntimeOrigin::signed(treasurer), season_id),
							Error::<Test>::CannotClaimDuringSeason
						);
					}
				}

				// during season 1
				for (treasurer, season_id) in [(ALICE, 1), (BOB, 2), (CHARLIE, 3)] {
					for iter in [
						season_1.early_start..=season_1.start, // 10..15
						season_1.start..=season_1.end,         // 15..20
					] {
						for n in iter {
							run_to_block(n);
							assert_noop!(
								AAvatars::claim_treasury(
									RuntimeOrigin::signed(treasurer),
									season_id
								),
								Error::<Test>::CannotClaimDuringSeason
							);
						}
					}
				}

				// before season 2
				for (treasurer, season_id) in [(BOB, 2), (CHARLIE, 3)] {
					for n in (season_1.end + 1)..season_2.early_start {
						run_to_block(n);
						deposit_into_treasury(1, 369);
						assert_ok!(AAvatars::claim_treasury(RuntimeOrigin::signed(ALICE), 1));
						assert_noop!(
							AAvatars::claim_treasury(RuntimeOrigin::signed(treasurer), season_id),
							Error::<Test>::CannotClaimDuringSeason
						);
					}
				}

				// during season 2
				for (treasurer, season_id) in [(BOB, 2), (CHARLIE, 3)] {
					for iter in [
						season_2.early_start..=season_2.start, // 25..30
						season_2.start..=season_2.end,         // 30..35
					] {
						for n in iter {
							run_to_block(n);
							deposit_into_treasury(1, 369);
							assert_ok!(AAvatars::claim_treasury(RuntimeOrigin::signed(ALICE), 1));
							assert_noop!(
								AAvatars::claim_treasury(
									RuntimeOrigin::signed(treasurer),
									season_id
								),
								Error::<Test>::CannotClaimDuringSeason
							);
						}
					}
				}

				// end of season 2
				for (treasurer, season_id) in [(CHARLIE, 3), (DAVE, 4)] {
					for n in (season_2.end + 1)..(season_2.end + 5) {
						run_to_block(n);
						deposit_into_treasury(1, 369);
						deposit_into_treasury(2, 369);
						assert_ok!(AAvatars::claim_treasury(RuntimeOrigin::signed(ALICE), 1));
						assert_ok!(AAvatars::claim_treasury(RuntimeOrigin::signed(BOB), 2));
						assert_noop!(
							AAvatars::claim_treasury(RuntimeOrigin::signed(treasurer), season_id),
							Error::<Test>::CannotClaimDuringSeason
						);
					}
				}
			})
	}

	#[test]
	fn claim_treasury_rejects_empty_treasury() {
		let season_1 = Season::default();
		ExtBuilder::default()
			.seasons(&[(1, season_1.clone())])
			.build()
			.execute_with(|| {
				run_to_block(season_1.end + 1);
				Treasurer::<Test>::insert(1, CHARLIE);
				assert_noop!(
					AAvatars::claim_treasury(RuntimeOrigin::signed(CHARLIE), 1),
					Error::<Test>::CannotClaimZero
				);
			})
	}

	#[test]
	fn claim_treasury_rejects_more_than_available() {
		let season_1 = Season::default();
		ExtBuilder::default()
			.seasons(&[(1, season_1.clone())])
			.build()
			.execute_with(|| {
				run_to_block(season_1.end + 1);
				Treasurer::<Test>::insert(1, CHARLIE);
				Treasury::<Test>::insert(1, 999);
				assert!(Balances::free_balance(&AAvatars::treasury_account_id()) < 999);
				assert_noop!(
					AAvatars::claim_treasury(RuntimeOrigin::signed(CHARLIE), 1),
					pallet_balances::Error::<Test>::InsufficientBalance
				);
			})
	}
}

mod season {
	use super::*;

	#[test]
	fn season_hook_should_work() {
		let season_1 = Season::default().early_start(2).start(3).end(4);
		let season_2 = Season::default().early_start(5).start(7).end(10);
		let season_3 = Season::default().early_start(23).start(37).end(53);
		let seasons = &[(1, season_1.clone()), (2, season_2.clone()), (3, season_3.clone())];

		ExtBuilder::default().seasons(seasons).build().execute_with(|| {
			// Check default values at block 1
			run_to_block(1);
			assert_eq!(System::block_number(), 1);
			assert_eq!(AAvatars::current_season_id(), 1);
			assert_eq!(
				AAvatars::current_season_status(),
				SeasonStatus {
					early: false,
					active: false,
					early_ended: false,
					max_tier_avatars: 0
				}
			);
			assert!(AAvatars::seasons(1).is_some());
			assert!(AAvatars::seasons(2).is_some());
			assert!(AAvatars::seasons(3).is_some());

			// Season 1 early start (block 2..3)
			for n in season_1.early_start..season_1.start {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 1);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: true,
						active: false,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}
			// Season 1 start (block 3..4)
			for n in season_1.start..season_2.early_start {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 1);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: false,
						active: true,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}

			// Season 2 early start (block 5..6)
			for n in season_2.early_start..season_2.start {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 2);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: true,
						active: false,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}
			// Season 2 start (block 7..9)
			for n in season_2.start..season_2.end {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 2);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: false,
						active: true,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}
			// Season 2 end (block 10..22)
			for n in (season_2.end + 1)..season_3.early_start {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 3);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: false,
						active: false,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}

			// Season 3 early start (block 23..36)
			for n in season_3.early_start..season_3.start {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 3);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: true,
						active: false,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}
			// Season 3 start (block 37..53)
			for n in season_3.start..=season_3.end {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 3);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: false,
						active: true,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}
			// Season 3 end (block 54..63)
			for n in (season_3.end + 1)..=(season_3.end + 10) {
				run_to_block(n);
				assert_eq!(AAvatars::current_season_id(), 4);
				assert_eq!(
					AAvatars::current_season_status(),
					SeasonStatus {
						early: false,
						active: false,
						early_ended: false,
						max_tier_avatars: 0
					}
				);
			}

			// No further seasons exist
			assert!(AAvatars::seasons(AAvatars::current_season_id()).is_none());
		})
	}

	#[test]
	fn season_validate_should_mutate_correctly() {
		let mut season = Season::default()
			.tiers(&[RarityTier::Rare, RarityTier::Common, RarityTier::Epic])
			.single_mint_probs(&[20, 80])
			.batch_mint_probs(&[60, 40]);
		assert_ok!(season.validate::<Test>());

		// check for ascending order sort
		assert_eq!(
			season.tiers.to_vec(),
			vec![RarityTier::Common, RarityTier::Rare, RarityTier::Epic]
		);

		// check for descending order sort
		assert_eq!(season.single_mint_probs.to_vec(), vec![80, 20]);
		assert_eq!(season.batch_mint_probs.to_vec(), vec![60, 40]);
	}

	#[test]
	fn set_season_should_work() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(&[(1, Season::default())])
			.build()
			.execute_with(|| {
				let season_1 = Season::default().early_start(1).start(5).end(10);
				assert_ok!(AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, season_1.clone()));
				assert_eq!(AAvatars::seasons(1), Some(season_1.clone()));
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::UpdatedSeason { season_id: 1, season: season_1 },
				));

				let season_2 = Season::default().early_start(11).start(12).end(13);
				assert_ok!(AAvatars::set_season(RuntimeOrigin::signed(ALICE), 2, season_2.clone()));
				assert_eq!(AAvatars::seasons(2), Some(season_2.clone()));
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::UpdatedSeason { season_id: 2, season: season_2 },
				));
			});
	}

	#[test]
	fn set_season_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::set_season(RuntimeOrigin::signed(BOB), 7357, Season::default()),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn set_season_should_reject_when_early_start_is_earlier_than_previous_season_end() {
		let season_1 = Season::default();
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(&[(1, season_1.clone())])
			.build()
			.execute_with(|| {
				for i in 0..season_1.end {
					let season_2 = Season::default().early_start(i).start(i + 1).end(i + 2);
					assert!(season_2.early_start <= season_1.end);
					assert_noop!(
						AAvatars::set_season(RuntimeOrigin::signed(ALICE), 2, season_2),
						Error::<Test>::EarlyStartTooEarly
					);
				}
			});
	}

	#[test]
	fn set_season_should_reject_when_early_start_is_earlier_than_or_equal_to_start() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for i in 3..6 {
				let new_season = Season::default().early_start(i).start(3).end(10);
				assert!(new_season.early_start >= new_season.start);
				assert_noop!(
					AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, new_season),
					Error::<Test>::EarlyStartTooLate
				);
			}
		});
	}

	#[test]
	fn set_season_should_reject_when_start_is_later_than_end() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let new_season = Season::default().early_start(11).start(12).end(10);
			assert!(new_season.early_start < new_season.start);
			assert_noop!(
				AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn set_season_should_reject_when_rarity_tier_is_duplicated() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for duplicated_rarity_tiers in [
				vec![RarityTier::Common, RarityTier::Common],
				vec![RarityTier::Common, RarityTier::Common, RarityTier::Legendary],
			] {
				assert_noop!(
					AAvatars::set_season(
						RuntimeOrigin::signed(ALICE),
						1,
						Season::default().tiers(&duplicated_rarity_tiers)
					),
					Error::<Test>::DuplicatedRarityTier
				);
			}
		});
	}

	#[test]
	fn set_season_should_reject_when_sum_of_rarity_chance_is_incorrect() {
		let tiers = &[RarityTier::Common, RarityTier::Uncommon, RarityTier::Legendary];
		let season_0 = Season::default().tiers(tiers);
		let season_1 = Season::default().tiers(tiers);
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for incorrect_percentages in [vec![12, 39], vec![123, 10], vec![83, 1, 43]] {
				for season in [
					season_0.clone().single_mint_probs(&incorrect_percentages),
					season_1.clone().single_mint_probs(&incorrect_percentages),
				] {
					assert_noop!(
						AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, season),
						Error::<Test>::IncorrectRarityPercentages
					);
				}
			}
		});
	}

	#[test]
	fn set_season_should_reject_when_season_to_update_ends_after_next_season_start() {
		let season_1 = Season::default().early_start(1).start(5).end(10);
		let season_2 = Season::default().early_start(11).start(15).end(20);

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(&[(1, season_1), (2, season_2.clone())])
			.build()
			.execute_with(|| {
				let season_1_update = Season::default().early_start(1).start(5).end(14);
				assert!(season_1_update.end > season_2.early_start);
				assert_noop!(
					AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, season_1_update),
					Error::<Test>::SeasonEndTooLate
				);
			});
	}

	#[test]
	fn set_season_should_reject_season_id_underflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::set_season(
					RuntimeOrigin::signed(ALICE),
					SeasonId::MIN,
					Season::default()
				),
				ArithmeticError::Underflow
			);
		});
	}

	#[test]
	fn set_season_should_reject_season_id_overflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::set_season(
					RuntimeOrigin::signed(ALICE),
					SeasonId::MAX,
					Season::default()
				),
				ArithmeticError::Overflow
			);
		});
	}

	#[test]
	fn set_season_should_reject_out_of_bound_variations() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for (season, error) in [
				(Season::default().max_variations(0), Error::<Test>::MaxVariationsTooLow),
				(Season::default().max_variations(1), Error::<Test>::MaxVariationsTooLow),
				(Season::default().max_variations(16), Error::<Test>::MaxVariationsTooHigh),
				(Season::default().max_variations(100), Error::<Test>::MaxVariationsTooHigh),
			] {
				assert_noop!(AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, season), error);
			}
		});
	}

	#[test]
	fn set_season_should_reject_out_of_bound_components_bounds() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for (season, error) in [
				(Season::default().max_components(0), Error::<Test>::MaxComponentsTooLow),
				(Season::default().max_components(1), Error::<Test>::MaxComponentsTooLow),
				(Season::default().max_components(33), Error::<Test>::MaxComponentsTooHigh),
				(Season::default().max_components(100), Error::<Test>::MaxComponentsTooHigh),
			] {
				assert_noop!(AAvatars::set_season(RuntimeOrigin::signed(ALICE), 1, season), error);
			}
		});
	}

	#[test]
	fn set_season_should_reject_when_season_ids_are_not_sequential() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(&[(1, Season::default())])
			.build()
			.execute_with(|| {
				assert_noop!(
					AAvatars::set_season(RuntimeOrigin::signed(ALICE), 3, Season::default()),
					Error::<Test>::NonSequentialSeasonId,
				);
			});
	}
}

mod config {
	use super::*;

	#[test]
	fn update_global_config_should_work() {
		ExtBuilder::default()
			.existential_deposit(1)
			.organizer(ALICE)
			.build()
			.execute_with(|| {
				let config = GlobalConfigOf::<Test>::default()
					.mint_fees_one(2)
					.mint_fees_three(2)
					.mint_fees_six(2)
					.transfer_avatar_transfer_fee(2)
					.trade_min_fee(2)
					.account_storage_upgrade_fe(2);

				assert_ok!(AAvatars::update_global_config(
					RuntimeOrigin::signed(ALICE),
					config.clone()
				));
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::UpdatedGlobalConfig(config),
				));
			});
	}

	#[test]
	fn update_global_config_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::update_global_config(
					RuntimeOrigin::signed(BOB),
					GlobalConfigOf::<Test>::default()
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn update_global_config_should_reject_fees_lower_than_existential_deposit() {
		ExtBuilder::default()
			.existential_deposit(333)
			.organizer(CHARLIE)
			.build()
			.execute_with(|| {
				for config in [
					GlobalConfigOf::<Test>::default().mint_fees_one(12),
					GlobalConfigOf::<Test>::default().mint_fees_three(34),
					GlobalConfigOf::<Test>::default().mint_fees_six(56),
					GlobalConfigOf::<Test>::default().transfer_avatar_transfer_fee(78),
					GlobalConfigOf::<Test>::default().trade_min_fee(91),
					GlobalConfigOf::<Test>::default().account_storage_upgrade_fe(99),
					GlobalConfigOf::<Test>::default()
						.mint_fees_one(999)
						.mint_fees_three(999)
						.mint_fees_six(1)
						.transfer_avatar_transfer_fee(999)
						.trade_min_fee(2)
						.account_storage_upgrade_fe(999),
				] {
					assert_noop!(
						AAvatars::update_global_config(RuntimeOrigin::signed(CHARLIE), config),
						Error::<Test>::TooLowFees
					);
				}
			});
	}
}

mod minting {
	use super::*;
	use frame_support::traits::Currency;

	#[test]
	fn ensure_for_mint_works() {
		let season = Season::default().early_start(10).start(20).end(30);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.free_mints(&[(ALICE, 42)])
			.build()
			.execute_with(|| {
				// Outside a season, both mints are unavailable.
				for n in 0..season.early_start {
					run_to_block(n);
					assert_eq!(
						AAvatars::current_season_status(),
						SeasonStatus {
							active: false,
							early: false,
							early_ended: false,
							max_tier_avatars: 0
						}
					);
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AAvatars::ensure_for_mint(&ALICE, &mint_type),
							Error::<Test>::SeasonClosed
						);
					}
				}

				// At early start, both mints are available for whitelisted accounts.
				for n in season.early_start..season.start {
					run_to_block(n);
					assert!(AAvatars::current_season_status().early);
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_ok!(AAvatars::ensure_for_mint(&ALICE, &mint_type), 42);
					}
				}
				// At early start, only free mint is available for non-whitelisted accounts.
				for n in season.early_start..season.start {
					run_to_block(n);
					assert!(AAvatars::current_season_status().early);
					assert_noop!(
						AAvatars::ensure_for_mint(&BOB, &MintType::Normal),
						Error::<Test>::SeasonClosed
					);
					assert_ok!(AAvatars::ensure_for_mint(&BOB, &MintType::Free), 0);
				}

				// At official start, both mints are available for all accounts.
				for n in season.start..=season.end {
					run_to_block(n);
					assert!(AAvatars::current_season_status().active);
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_ok!(AAvatars::ensure_for_mint(&ALICE, &mint_type), 42);
						assert_ok!(AAvatars::ensure_for_mint(&BOB, &mint_type), 0);
					}
				}

				// At premature end, only free mint is available for all accounts.
				for n in season.start..=season.end {
					run_to_block(n);
					CurrentSeasonStatus::<Test>::mutate(|status| status.early_ended = true);
					assert_noop!(
						AAvatars::ensure_for_mint(&ALICE, &MintType::Normal),
						Error::<Test>::PrematureSeasonEnd
					);
					assert_ok!(AAvatars::ensure_for_mint(&ALICE, &MintType::Free), 42);
					assert_noop!(
						AAvatars::ensure_for_mint(&BOB, &MintType::Normal),
						Error::<Test>::PrematureSeasonEnd
					);
					assert_ok!(AAvatars::ensure_for_mint(&BOB, &MintType::Free), 0);
					CurrentSeasonStatus::<Test>::mutate(|status| status.early_ended = false);
				}

				// At season end, both mints are unavailable for all accounts.
				for n in season.end + 1..(season.end + 5) {
					run_to_block(n);
					assert_eq!(
						AAvatars::current_season_status(),
						SeasonStatus {
							active: false,
							early: false,
							early_ended: false,
							max_tier_avatars: 0
						}
					);
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AAvatars::ensure_for_mint(&ALICE, &mint_type),
							Error::<Test>::SeasonClosed
						);
						assert_noop!(
							AAvatars::ensure_for_mint(&BOB, &mint_type),
							Error::<Test>::SeasonClosed
						);
					}
				}
			});
	}

	#[test]
	fn mint_should_work() {
		let season_1 = Season::default().early_start(3).start(5).end(20).max_components(7);
		let season_2 = Season::default().early_start(23).start(35).end(40).max_components(17);

		let expected_nonce_increment = 1 as MockIndex;
		let fees = MintFees { one: 12, three: 34, six: 56 };
		let mint_cooldown = 1;

		let mut initial_balance = fees.one + fees.three + fees.six + MockExistentialDeposit::get();
		let mut initial_treasury_balance = 0;
		let mut initial_free_mints = 12;

		ExtBuilder::default()
			.seasons(&[(1, season_1.clone()), (2, season_2)])
			.mint_fees(fees)
			.mint_cooldown(mint_cooldown)
			.balances(&[(ALICE, initial_balance)])
			.free_mints(&[(ALICE, initial_free_mints)])
			.build()
			.execute_with(|| {
				for mint_type in [MintType::Normal, MintType::Free] {
					let mut expected_nonce = 0;
					let mut owned_avatar_count = 0;
					let mut season_minted_count = 0;
					let mut minted_count = 0;

					System::set_block_number(1);
					CurrentSeasonId::<Test>::set(1);
					SeasonStats::<Test>::mutate(1, ALICE, |info| info.minted = 0);
					SeasonStats::<Test>::mutate(2, ALICE, |info| info.minted = 0);
					Owners::<Test>::remove(ALICE);
					Accounts::<Test>::mutate(ALICE, |account| {
						account.stats.mint.first = 0;
						account.stats.mint.last = 0;
					});

					// initial checks
					match mint_type {
						MintType::Normal =>
							assert_eq!(Balances::total_balance(&ALICE), initial_balance),
						MintType::Free =>
							assert_eq!(AAvatars::accounts(ALICE).free_mints, initial_free_mints),
					}
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert!(!AAvatars::current_season_status().active);

					// single mint
					run_to_block(season_1.start);
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: mint_type.clone() }
					));
					match mint_type {
						MintType::Normal => {
							initial_balance -= fees.fee_for(&MintPackSize::One);
							initial_treasury_balance += fees.fee_for(&MintPackSize::One);

							assert_eq!(Balances::total_balance(&ALICE), initial_balance);
							assert_eq!(AAvatars::treasury(1), initial_treasury_balance);
						},
						MintType::Free => {
							initial_free_mints -= MintPackSize::One as MintCount;
							assert_eq!(AAvatars::accounts(ALICE).free_mints, initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment;
					owned_avatar_count += 1;
					minted_count += 1;
					season_minted_count += 1;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert_eq!(AAvatars::season_stats(1, ALICE).minted, season_minted_count);
					assert!(AAvatars::current_season_status().active);
					assert_eq!(AAvatars::accounts(ALICE).stats.mint.first, season_1.start);
					System::assert_has_event(mock::RuntimeEvent::AAvatars(
						crate::Event::SeasonStarted(1),
					));
					System::assert_last_event(mock::RuntimeEvent::AAvatars(
						crate::Event::AvatarsMinted {
							avatar_ids: vec![AAvatars::owners(ALICE)[0]],
						},
					));

					// batch mint: three
					run_to_block(System::block_number() + 1 + mint_cooldown);
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::Three, mint_type: mint_type.clone() }
					));
					match mint_type {
						MintType::Normal => {
							initial_balance -= fees.fee_for(&MintPackSize::Three);
							initial_treasury_balance += fees.fee_for(&MintPackSize::Three);

							assert_eq!(Balances::total_balance(&ALICE), initial_balance);
							assert_eq!(AAvatars::treasury(1), initial_treasury_balance);
						},
						MintType::Free => {
							initial_free_mints -= MintPackSize::Three as MintCount;
							assert_eq!(AAvatars::accounts(ALICE).free_mints, initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment * 3;
					owned_avatar_count += 3;
					minted_count += 3;
					season_minted_count += 3;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert_eq!(AAvatars::season_stats(1, ALICE).minted, season_minted_count);
					assert!(AAvatars::current_season_status().active);
					System::assert_last_event(mock::RuntimeEvent::AAvatars(
						crate::Event::AvatarsMinted {
							avatar_ids: AAvatars::owners(ALICE)[1..=3].to_vec(),
						},
					));

					// batch mint: six
					run_to_block(System::block_number() + 1 + mint_cooldown);
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::Six, mint_type: mint_type.clone() }
					));
					match mint_type {
						MintType::Normal => {
							initial_balance -= fees.fee_for(&MintPackSize::Six);
							initial_treasury_balance += fees.fee_for(&MintPackSize::Six);

							assert_eq!(Balances::total_balance(&ALICE), initial_balance);
							assert_eq!(AAvatars::treasury(1), initial_treasury_balance);
						},
						MintType::Free => {
							initial_free_mints -= MintPackSize::Six as MintCount;
							assert_eq!(AAvatars::accounts(ALICE).free_mints, initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment * 6;
					owned_avatar_count += 6;
					minted_count += 6;
					season_minted_count += 6;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert_eq!(AAvatars::season_stats(1, ALICE).minted, season_minted_count);
					assert!(AAvatars::current_season_status().active);
					System::assert_last_event(mock::RuntimeEvent::AAvatars(
						crate::Event::AvatarsMinted {
							avatar_ids: AAvatars::owners(ALICE)[4..=9].to_vec(),
						},
					));

					match mint_type {
						MintType::Normal => {
							// mint one more avatar to trigger reaping
							assert_eq!(
								Balances::total_balance(&ALICE),
								MockExistentialDeposit::get()
							);
							run_to_block(System::block_number() + mint_cooldown);
							assert_ok!(AAvatars::mint(
								RuntimeOrigin::signed(ALICE),
								MintOption {
									count: MintPackSize::One,
									mint_type: mint_type.clone()
								}
							));
							minted_count += 1;

							// account is reaped, nonce and balance are reset to 0
							assert_eq!(System::account_nonce(ALICE), 0);
							assert_eq!(Balances::total_balance(&ALICE), 0);
						},
						MintType::Free => {
							assert_eq!(System::account_nonce(ALICE), expected_nonce);
						},
					}

					// check for season ending
					run_to_block(season_1.end + 1);
					assert_noop!(
						AAvatars::mint(
							RuntimeOrigin::signed(ALICE),
							MintOption { count: MintPackSize::One, mint_type: mint_type.clone() }
						),
						Error::<Test>::SeasonClosed
					);

					// total minted count updates
					let seasons_participated =
						AAvatars::accounts(ALICE).stats.mint.seasons_participated;
					assert_eq!(
						seasons_participated
							.iter()
							.map(|season_id| AAvatars::season_stats(season_id, ALICE).minted)
							.sum::<u32>(),
						minted_count as u32
					);

					// check participation
					assert_eq!(seasons_participated.into_iter().collect::<Vec<_>>(), vec![1]);

					// current season minted count resets
					assert_eq!(AAvatars::current_season_id(), 2);
					assert_eq!(AAvatars::season_stats(2, ALICE).minted, 0);

					// check for minted avatars
					let minted = AAvatars::owners(ALICE)
						.into_iter()
						.map(|avatar_id| AAvatars::avatars(avatar_id).unwrap())
						.collect::<Vec<_>>();
					assert!(minted.iter().all(|(owner, avatar)| {
						owner == &ALICE &&
							(avatar.souls >= 1 && avatar.souls <= 100) &&
							avatar.season_id == 1
					}));
				}
			});
	}

	#[test]
	fn mint_should_reject_when_minting_is_closed() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.mint.open = false);
			run_to_block(season.start);
			for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				for mint_type in [MintType::Normal, MintType::Free] {
					assert_noop!(
						AAvatars::mint(
							RuntimeOrigin::signed(ALICE),
							MintOption { count, mint_type }
						),
						Error::<Test>::MintClosed
					);
				}
			}
		});
	}

	#[test]
	fn mint_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				for mint_type in [MintType::Normal, MintType::Free] {
					assert_noop!(
						AAvatars::mint(RuntimeOrigin::none(), MintOption { count, mint_type }),
						DispatchError::BadOrigin
					);
				}
			}
		});
	}

	#[test]
	fn mint_should_reject_non_whitelisted_accounts_when_season_is_inactive() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_234_567_890_123_456)])
			.free_mints(&[(ALICE, 0)])
			.build()
			.execute_with(|| {
				for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AAvatars::mint(
								RuntimeOrigin::signed(ALICE),
								MintOption { count, mint_type }
							),
							Error::<Test>::SeasonClosed
						);
					}
				}
			});
	}

	#[test]
	fn mint_should_reject_when_max_ownership_has_reached() {
		use sp_runtime::traits::Get;

		let season = Season::default();
		let avatar_ids = BoundedAvatarIdsOf::<Test>::try_from(
			(0..MaxAvatarsPerPlayer::get() as usize)
				.map(|_| sp_core::H256::default())
				.collect::<Vec<_>>(),
		)
		.unwrap();
		assert_eq!(avatar_ids.len(), MaxAvatarsPerPlayer::get() as usize);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, 1_234_567_890_123_456)])
			.free_mints(&[(ALICE, 10)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				Owners::<Test>::insert(ALICE, avatar_ids);
				for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AAvatars::mint(
								RuntimeOrigin::signed(ALICE),
								MintOption { count, mint_type }
							),
							Error::<Test>::MaxOwnershipReached
						);
					}
				}
			});
	}

	#[test]
	fn mint_should_wait_for_cooldown() {
		let season = Season::default().early_start(1).start(3).end(20);
		let mint_cooldown = 7;

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(mint_cooldown)
			.balances(&[(ALICE, 1_234_567_890_123_456)])
			.free_mints(&[(ALICE, 10)])
			.build()
			.execute_with(|| {
				for mint_type in [MintType::Normal, MintType::Free] {
					run_to_block(season.start + 1);
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: mint_type.clone() }
					));

					for _ in 1..mint_cooldown {
						run_to_block(System::block_number() + 1);
						assert_noop!(
							AAvatars::mint(
								RuntimeOrigin::signed(ALICE),
								MintOption {
									count: MintPackSize::One,
									mint_type: mint_type.clone()
								}
							),
							Error::<Test>::MintCooldown
						);
					}

					run_to_block(System::block_number() + 1);
					assert_eq!(System::block_number(), (season.start + 1) + mint_cooldown);
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
					));

					// reset for next iteration
					System::set_block_number(0);
					Accounts::<Test>::mutate(ALICE, |account| account.stats.mint.last = 0);
				}
			});
	}

	#[test]
	fn mint_should_reject_when_balance_is_insufficient() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);

			for mint_count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				assert_noop!(
					AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: mint_count, mint_type: MintType::Normal }
					),
					pallet_balances::Error::<Test>::InsufficientBalance
				);
			}

			for mint_count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				assert_noop!(
					AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: mint_count, mint_type: MintType::Free }
					),
					Error::<Test>::InsufficientFreeMints
				);
			}
		});
	}

	#[test]
	fn set_free_mints_works() {
		ExtBuilder::default()
			.organizer(ALICE)
			.free_mints(&[(BOB, 123)])
			.build()
			.execute_with(|| {
				assert_ok!(AAvatars::set_free_mints(RuntimeOrigin::signed(ALICE), BOB, 999));
				assert_eq!(AAvatars::accounts(BOB).free_mints, 999);

				assert_ok!(AAvatars::set_free_mints(RuntimeOrigin::signed(ALICE), BOB, 0));
				assert_eq!(AAvatars::accounts(BOB).free_mints, 0);
			})
	}

	#[test]
	fn set_free_mints_rejects_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for not_organizer in [BOB, CHARLIE] {
				assert_noop!(
					AAvatars::set_free_mints(RuntimeOrigin::signed(not_organizer), ALICE, 123),
					DispatchError::BadOrigin
				);
			}
		})
	}
}

mod forging {
	use super::*;
	use sp_runtime::testing::H256;
	use sp_std::collections::btree_set::BTreeSet;

	fn create_avatar_for_bob(dna: &[u8]) -> AvatarIdOf<Test> {
		let avatar = Avatar::default().season_id(1).dna(dna);
		if avatar.min_tier() == RarityTier::Legendary as u8 {
			CurrentSeasonStatus::<Test>::mutate(|status| status.max_tier_avatars += 1);
		}

		let avatar_id = H256::random();
		Avatars::<Test>::insert(avatar_id, (BOB, avatar));
		Owners::<Test>::try_append(BOB, avatar_id).unwrap();

		avatar_id
	}

	#[test]
	fn forge_works_for_season_1() {
		let season = Season::default()
			.early_start(100)
			.start(200)
			.end(150_000)
			.max_tier_forges(100)
			.max_variations(6)
			.max_components(11)
			.min_sacrifices(1)
			.max_sacrifices(4)
			.tiers(&[RarityTier::Common, RarityTier::Rare, RarityTier::Legendary])
			.single_mint_probs(&[95, 5])
			.batch_mint_probs(&[80, 20])
			.base_prob(20)
			.per_period(20)
			.periods(12);

		let expected_upgraded_components = |dna_1: &[u8], dna_2: &[u8]| -> usize {
			dna_1.iter().zip(dna_2).filter(|(left, right)| left != right).count()
		};

		ExtBuilder::default()
			.seasons(&[(1, season)])
			.mint_cooldown(5)
			.build()
			.execute_with(|| {
				run_to_block(15_792);

				let dna_before = [0x23, 0x25, 0x24, 0x20, 0x05, 0x25, 0x01, 0x20, 0x02, 0x23, 0x23];
				let dna_after = [0x23, 0x25, 0x24, 0x20, 0x05, 0x25, 0x21, 0x20, 0x22, 0x23, 0x23];
				assert_eq!(expected_upgraded_components(&dna_before, &dna_after), 2);

				let leader_id = create_avatar_for_bob(&dna_before);
				let sacrifice_ids = [
					create_avatar_for_bob(&[
						0x00, 0x00, 0x04, 0x01, 0x03, 0x21, 0x00, 0x00, 0x03, 0x04, 0x04,
					]),
					create_avatar_for_bob(&[
						0x04, 0x02, 0x04, 0x21, 0x02, 0x05, 0x02, 0x21, 0x02, 0x23, 0x00,
					]),
					create_avatar_for_bob(&[
						0x02, 0x05, 0x22, 0x02, 0x23, 0x05, 0x02, 0x24, 0x05, 0x03, 0x03,
					]),
					create_avatar_for_bob(&[
						0x23, 0x24, 0x23, 0x21, 0x25, 0x23, 0x00, 0x25, 0x01, 0x22, 0x05,
					]),
				];

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					sacrifice_ids.to_vec()
				));
				let leader = AAvatars::avatars(leader_id).unwrap().1;
				assert_eq!(leader.dna.to_vec(), dna_after.to_vec());
				assert_eq!(leader.min_tier(), RarityTier::Common as u8);
			});
	}

	#[test]
	fn forge_should_work() {
		let season = Season::default()
			.tiers(&[RarityTier::Common, RarityTier::Uncommon, RarityTier::Legendary])
			.single_mint_probs(&[100, 0])
			.batch_mint_probs(&[100, 0])
			.base_prob(30)
			.per_period(1)
			.periods(6)
			.max_tier_forges(1)
			.max_components(8)
			.max_variations(6)
			.min_sacrifices(1)
			.max_sacrifices(4);

		let mut forged_count = 0;
		let mut assert_dna =
			|leader_id: &AvatarIdOf<Test>, expected_dna: &[u8], insert_dna: Option<&[u8]>| {
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption { count: MintPackSize::Three, mint_type: MintType::Normal }
				));
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				if let Some(dna) = insert_dna {
					AAvatars::owners(BOB)[1..=4].iter().for_each(|id| {
						Avatars::<Test>::mutate(id, |maybe_avatar| {
							if let Some((_, avatar)) = maybe_avatar {
								avatar.dna = dna.to_vec().try_into().unwrap();
							}
						});
					})
				}

				let original_leader_souls = AAvatars::avatars(leader_id).unwrap().1.souls;
				let sacrifice_ids = AAvatars::owners(BOB)[1..=4].to_vec();
				let sacrifice_souls = sacrifice_ids
					.iter()
					.map(|id| AAvatars::avatars(id).unwrap().1.souls)
					.sum::<SoulCount>();
				assert_ne!(sacrifice_souls, 0);

				assert_ok!(AAvatars::forge(RuntimeOrigin::signed(BOB), *leader_id, sacrifice_ids));
				assert_eq!(
					AAvatars::avatars(leader_id).unwrap().1.souls,
					original_leader_souls + sacrifice_souls
				);
				assert_eq!(AAvatars::avatars(leader_id).unwrap().1.dna.to_vec(), expected_dna);

				forged_count += 1;
				assert_eq!(
					AAvatars::accounts(BOB)
						.stats
						.forge
						.seasons_participated
						.iter()
						.map(|season_id| AAvatars::season_stats(season_id, BOB).forged)
						.sum::<u32>(),
					forged_count as u32
				);
				assert_eq!(AAvatars::season_stats(1, BOB).forged, forged_count);
				assert_eq!(AAvatars::accounts(BOB).stats.forge.first, season.start);
				assert_eq!(AAvatars::accounts(BOB).stats.forge.last, System::block_number());
			};

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(0)
			.mint_fees(MintFees { one: 1, three: 3, six: 6 })
			.balances(&[(BOB, MockBalance::max_value())])
			.free_mints(&[(BOB, 0)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));
				let leader_id = AAvatars::owners(BOB)[0];
				assert_eq!(
					AAvatars::avatars(&leader_id).unwrap().1.dna.to_vec(),
					&[0x03, 0x04, 0x00, 0x03, 0x03, 0x05, 0x05, 0x01]
				);

				// 1st mutation
				assert_dna(&leader_id, &[0x03, 0x04, 0x00, 0x03, 0x03, 0x05, 0x05, 0x01], None);

				// 2nd mutation
				assert_dna(&leader_id, &[0x13, 0x04, 0x00, 0x03, 0x03, 0x05, 0x05, 0x11], None);

				// 3rd mutation
				assert_dna(&leader_id, &[0x13, 0x14, 0x10, 0x03, 0x03, 0x05, 0x05, 0x11], None);

				// 4th mutation
				assert_dna(&leader_id, &[0x13, 0x14, 0x10, 0x03, 0x03, 0x05, 0x15, 0x11], None);

				// 5th mutation
				assert_dna(&leader_id, &[0x13, 0x14, 0x10, 0x13, 0x13, 0x05, 0x15, 0x11], None);

				// 6th mutation: force 6th component's variations to be similar
				assert_dna(
					&leader_id,
					&[0x13, 0x14, 0x10, 0x13, 0x13, 0x15, 0x15, 0x11],
					Some(&[0x13, 0x14, 0x10, 0x13, 0x13, 0x06, 0x15, 0x11]),
				);

				// force highest tier mint and assert for associated checks
				assert_dna(
					&leader_id,
					&[0x43, 0x44, 0x40, 0x13, 0x13, 0x15, 0x15, 0x11],
					Some(&[0x14, 0x15, 0x11, 0x14, 0x13, 0x15, 0x15, 0x11]),
				);
				assert_dna(
					&leader_id,
					&[0x43, 0x44, 0x40, 0x43, 0x43, 0x45, 0x45, 0x11],
					Some(&[0x43, 0x44, 0x40, 0x14, 0x14, 0x16, 0x16, 0x11]),
				);
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, 0);

				// fully upgraded
				assert_dna(
					&leader_id,
					&[0x43, 0x44, 0x40, 0x43, 0x43, 0x45, 0x45, 0x41],
					Some(&[0x43, 0x44, 0x40, 0x43, 0x43, 0x45, 0x45, 0x12]),
				);
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, 1);
				assert!(AAvatars::current_season_status().early_ended);
				assert_noop!(
					AAvatars::mint(
						RuntimeOrigin::signed(BOB),
						MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
					),
					Error::<Test>::PrematureSeasonEnd
				);

				// check stats for season 1
				assert_eq!(AAvatars::season_stats(1, BOB).forged, 9);
				assert_eq!(AAvatars::season_stats(1, BOB).minted, 37);

				// trigger season end and assert for associated checks
				run_to_block(season.end + 1);
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, 0);
				assert!(!AAvatars::current_season_status().early_ended);

				// check stats for season 2
				assert_eq!(AAvatars::season_stats(2, BOB).forged, 0);
				assert_eq!(AAvatars::season_stats(2, BOB).minted, 0);
			});
	}

	#[test]
	fn forge_should_update_max_tier_avatars() {
		let season = Season::default()
			.tiers(&[RarityTier::Common, RarityTier::Legendary])
			.max_components(8)
			.max_variations(6)
			.max_tier_forges(5);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(0)
			.free_mints(&[(BOB, MintCount::MAX)])
			.build()
			.execute_with(|| {
				run_to_block(season.early_start);

				let mut max_tier_avatars = 0;
				let common_avatar_ids = [
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x02]),
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x03]),
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x03]),
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x03]),
				];

				// `max_tier_avatars` increases when a legendary is forged
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, max_tier_avatars);
				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					common_avatar_ids[0],
					common_avatar_ids[1..].to_vec()
				));
				max_tier_avatars += 1;
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, max_tier_avatars);
				assert_eq!(AAvatars::owners(BOB).len(), 4 - 3);

				// `max_tier_avatars` decreases when legendaries are sacrificed
				let legendary_avatar_ids = [
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x42]),
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x42]),
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x42]),
					create_avatar_for_bob(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x44, 0x43, 0x42]),
				];
				max_tier_avatars += 4;
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, max_tier_avatars);

				// leader is already legendary so max_tier_avatars isn't incremented
				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					legendary_avatar_ids[0],
					legendary_avatar_ids[1..].to_vec()
				));
				assert_eq!(AAvatars::current_season_status().max_tier_avatars, max_tier_avatars);
				assert_eq!(AAvatars::owners(BOB).len(), (4 - 3) + (4 - 3));
				assert_eq!(
					AAvatars::accounts(BOB)
						.stats
						.forge
						.seasons_participated
						.into_iter()
						.collect::<Vec<_>>(),
					vec![1]
				);

				// NOTE: BOB forged twice so the seasonal forged count is 2
				assert_eq!(AAvatars::season_stats(1, BOB).forged, 2);
			});
	}

	#[test]
	fn forge_should_work_for_matches() {
		let tiers = &[RarityTier::Common, RarityTier::Legendary];
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[100])
			.max_components(5)
			.max_variations(3)
			.min_sacrifices(1)
			.max_sacrifices(2);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Free }
				));

				// forge
				let owned_avatar_ids = AAvatars::owners(BOB);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_ids = &owned_avatar_ids[1..3];

				let original_leader = AAvatars::avatars(leader_id).unwrap().1;
				let original_sacrifices = sacrifice_ids
					.iter()
					.map(|id| AAvatars::avatars(id).unwrap().1)
					.collect::<Vec<_>>();

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					sacrifice_ids.to_vec()
				));
				let forged_leader = AAvatars::avatars(leader_id).unwrap().1;

				// check the result of the compare method
				let upgradable_indexes = original_leader.upgradable_indexes::<Test>().unwrap();
				for (sacrifice, result) in original_sacrifices
					.iter()
					.zip([(true, BTreeSet::from([1, 3])), (true, BTreeSet::from([0, 2, 3]))])
				{
					assert_eq!(
						original_leader.compare(
							sacrifice,
							&upgradable_indexes,
							season.max_variations,
							tiers[tiers.len() - 1].clone() as u8
						),
						result
					)
				}

				// check all sacrifices are burned
				for sacrifice_id in sacrifice_ids {
					assert!(!Avatars::<Test>::contains_key(sacrifice_id));
				}
				// check for souls accumulation
				assert_eq!(
					forged_leader.souls,
					original_leader.souls +
						original_sacrifices.iter().map(|x| x.souls).sum::<SoulCount>(),
				);

				// check for the upgraded DNA
				assert_ne!(original_leader.dna[0..=1], forged_leader.dna[0..=1]);
				assert_eq!(original_leader.dna.to_vec()[0] >> 4, RarityTier::Common as u8);
				assert_eq!(original_leader.dna.to_vec()[1] >> 4, RarityTier::Common as u8);
				assert_eq!(forged_leader.dna.to_vec()[0] >> 4, RarityTier::Legendary as u8);
				assert_eq!(forged_leader.dna.to_vec()[1] >> 4, RarityTier::Legendary as u8);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarForged { avatar_id: leader_id, upgraded_components: 2 },
				));

				// variations remain the same
				assert_eq!(
					original_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
					forged_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
				);
				// other components remain the same
				assert_eq!(
					original_leader.dna[2..season.max_components as usize],
					forged_leader.dna[2..season.max_components as usize]
				);
			});
	}

	#[test]
	fn forge_should_work_for_non_matches() {
		let tiers =
			&[RarityTier::Common, RarityTier::Uncommon, RarityTier::Rare, RarityTier::Legendary];
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[33, 33, 34])
			.max_components(10)
			.max_variations(12)
			.min_sacrifices(1)
			.max_sacrifices(5);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Free }
				));

				// forge
				let owned_avatar_ids = AAvatars::owners(BOB);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_id = owned_avatar_ids[1];

				let original_leader = AAvatars::avatars(leader_id).unwrap().1;
				let original_sacrifice = AAvatars::avatars(sacrifice_id).unwrap().1;

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					vec![sacrifice_id]
				));
				let forged_leader = AAvatars::avatars(leader_id).unwrap().1;

				// check the result of the compare method
				let upgradable_indexes = original_leader.upgradable_indexes::<Test>().unwrap();
				assert_eq!(
					original_leader.compare(
						&original_sacrifice,
						&upgradable_indexes,
						season.max_variations,
						tiers[tiers.len() - 1].clone() as u8
					),
					(false, BTreeSet::new())
				);
				// check all sacrifices are burned
				assert!(!Avatars::<Test>::contains_key(sacrifice_id));
				// check for souls accumulation
				assert_eq!(forged_leader.souls, original_leader.souls + original_sacrifice.souls);

				// check DNAs are the same
				assert_eq!(original_leader.dna, forged_leader.dna);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarForged { avatar_id: leader_id, upgraded_components: 0 },
				));
			});
	}

	#[test]
	fn forge_should_ignore_low_tier_sacrifices() {
		let tiers = &[RarityTier::Common, RarityTier::Rare, RarityTier::Legendary];
		let season = Season::default()
			.tiers(tiers)
			.single_mint_probs(&[100, 0])
			.batch_mint_probs(&[100, 0])
			.max_tier_forges(1)
			.max_components(4)
			.max_variations(6)
			.min_sacrifices(1)
			.max_sacrifices(4);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(0)
			.free_mints(&[(ALICE, MintCount::MAX)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(ALICE),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Free }
				));
				let leader_id = AAvatars::owners(ALICE)[0];
				assert_eq!(
					AAvatars::avatars(&leader_id).unwrap().1.dna.to_vec(),
					&[0x04, 0x03, 0x05, 0x01]
				);
				assert_eq!(
					AAvatars::avatars(&leader_id).unwrap().1.min_tier(),
					tiers[0].clone() as u8,
				);

				// mutate the DNA of leader to make it a tier higher
				let mut leader_avatar = AAvatars::avatars(leader_id).unwrap();
				leader_avatar.1.dna = leader_avatar
					.1
					.dna
					.iter()
					.map(|x| ((tiers[1].clone() as u8) << 4) | (x & 0b0000_1111))
					.collect::<Vec<_>>()
					.try_into()
					.unwrap();
				Avatars::<Test>::insert(leader_id, &leader_avatar);
				assert_eq!(
					AAvatars::avatars(&leader_id).unwrap().1.min_tier(),
					tiers[1].clone() as u8
				);

				// forging doesn't take effect
				let sacrifice_ids = &AAvatars::owners(ALICE)[1..5];
				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(ALICE),
					leader_id,
					sacrifice_ids.to_vec()
				));
				assert_eq!(AAvatars::avatars(leader_id).unwrap().1.dna, leader_avatar.1.dna);
			});
	}

	#[test]
	fn forge_should_reject_when_forging_is_closed() {
		let season = Season::default().min_sacrifices(0);

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);
			GlobalConfigs::<Test>::mutate(|config| config.forge.open = false);
			assert_noop!(
				AAvatars::forge(RuntimeOrigin::signed(ALICE), H256::default(), Vec::new()),
				Error::<Test>::ForgeClosed,
			);
		});
	}

	#[test]
	fn forge_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::forge(RuntimeOrigin::none(), H256::default(), Vec::new()),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn forge_should_reject_out_of_bound_sacrifices() {
		let season = Season::default().min_sacrifices(3).max_sacrifices(5);

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);

			for i in 0..season.min_sacrifices {
				assert_noop!(
					AAvatars::forge(
						RuntimeOrigin::signed(ALICE),
						H256::default(),
						(0..i).map(|_| H256::default()).collect::<Vec<_>>(),
					),
					Error::<Test>::TooFewSacrifices,
				);
			}

			for i in (season.max_sacrifices + 1)..(season.max_sacrifices + 5) {
				assert_noop!(
					AAvatars::forge(
						RuntimeOrigin::signed(ALICE),
						H256::default(),
						(0..i).map(|_| H256::default()).collect::<Vec<_>>(),
					),
					Error::<Test>::TooManySacrifices,
				);
			}
		});
	}

	#[test]
	fn forge_should_not_be_interrupted_by_season_status() {
		let season_1 = Season::default().early_start(5).start(10).end(20);
		let season_2 = Season::default().early_start(30).start(40).end(50);
		let seasons = &[(1, season_1.clone()), (2, season_2.clone())];

		ExtBuilder::default()
			.seasons(seasons)
			.mint_cooldown(0)
			.free_mints(&[(ALICE, 100)])
			.build()
			.execute_with(|| {
				Accounts::<Test>::mutate(ALICE, |info| info.storage_tier = StorageTier::Four);

				run_to_block(season_1.early_start);
				for _ in 0..33 {
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::Three, mint_type: MintType::Free }
					));
				}

				for iter in [
					season_1.early_start..season_1.end, // block 5..19
					season_1.end..season_2.early_start, // block 20..29
				] {
					for n in iter {
						run_to_block(n);
						assert_ok!(AAvatars::forge(
							RuntimeOrigin::signed(ALICE),
							AAvatars::owners(ALICE)[0],
							AAvatars::owners(ALICE)[1..3].to_vec()
						));
					}
				}
			});
	}

	#[test]
	fn forge_should_reject_unknown_season_calls() {
		ExtBuilder::default().build().execute_with(|| {
			CurrentSeasonId::<Test>::put(123);
			CurrentSeasonStatus::<Test>::mutate(|status| status.active = true);
			assert_noop!(
				AAvatars::forge(
					RuntimeOrigin::signed(ALICE),
					H256::default(),
					vec![H256::default()]
				),
				Error::<Test>::UnknownSeason,
			);
		});
	}

	#[test]
	fn forge_should_reject_unknown_avatars() {
		let season = Season::default().min_sacrifices(1).max_sacrifices(3);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(0)
			.free_mints(&[(ALICE, 10)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				for _ in 0..season.max_sacrifices {
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Free }
					));
				}

				let owned_avatars = AAvatars::owners(ALICE);
				for (leader, sacrifices) in [
					(H256::default(), vec![owned_avatars[0], owned_avatars[2]]),
					(owned_avatars[1], vec![H256::default(), H256::default()]),
					(owned_avatars[1], vec![H256::default(), owned_avatars[2]]),
				] {
					assert_noop!(
						AAvatars::forge(RuntimeOrigin::signed(ALICE), leader, sacrifices),
						Error::<Test>::UnknownAvatar,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_incorrect_ownership() {
		let season = Season::default().min_sacrifices(1).max_sacrifices(3);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(0)
			.free_mints(&[(ALICE, 10), (BOB, 10)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				for player in [ALICE, BOB] {
					for _ in 0..season.max_sacrifices {
						assert_ok!(AAvatars::mint(
							RuntimeOrigin::signed(player),
							MintOption { count: MintPackSize::One, mint_type: MintType::Free }
						));
					}
				}

				for (player, leader, sacrifices) in [
					(ALICE, AAvatars::owners(ALICE)[0], AAvatars::owners(BOB)[0..2].to_vec()),
					(ALICE, AAvatars::owners(BOB)[0], AAvatars::owners(ALICE)[0..2].to_vec()),
					(ALICE, AAvatars::owners(BOB)[0], AAvatars::owners(BOB)[1..2].to_vec()),
				] {
					assert_noop!(
						AAvatars::forge(RuntimeOrigin::signed(player), leader, sacrifices),
						Error::<Test>::Ownership,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_leader_in_sacrifice() {
		let season = Season::default().min_sacrifices(1).max_sacrifices(3);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(0)
			.free_mints(&[(ALICE, 10)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				for _ in 0..season.max_sacrifices {
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Free }
					));
				}

				for (player, leader, sacrifices) in [
					(ALICE, AAvatars::owners(ALICE)[0], AAvatars::owners(ALICE)[0..2].to_vec()),
					(ALICE, AAvatars::owners(ALICE)[1], AAvatars::owners(ALICE)[0..2].to_vec()),
				] {
					assert_noop!(
						AAvatars::forge(RuntimeOrigin::signed(player), leader, sacrifices),
						Error::<Test>::LeaderSacrificed,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_avatars_in_trade() {
		let season = Season::default();
		let price = 321;
		let initial_balance = 6 + MockExistentialDeposit::get();

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, initial_balance), (BOB, 6 + initial_balance)])
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(ALICE),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));

				let leader = AAvatars::owners(ALICE)[0];
				let sacrifices = AAvatars::owners(ALICE)[1..3].to_vec();

				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), leader, price));
				assert_noop!(
					AAvatars::forge(RuntimeOrigin::signed(ALICE), leader, sacrifices.clone()),
					Error::<Test>::AvatarInTrade
				);

				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), sacrifices[1], price));
				assert_noop!(
					AAvatars::forge(RuntimeOrigin::signed(ALICE), leader, sacrifices.clone()),
					Error::<Test>::AvatarInTrade
				);

				assert_ok!(AAvatars::remove_price(RuntimeOrigin::signed(ALICE), leader));
				assert_noop!(
					AAvatars::forge(RuntimeOrigin::signed(ALICE), leader, sacrifices),
					Error::<Test>::AvatarInTrade
				);
			});
	}

	#[test]
	pub fn forge_should_reject_avatars_from_different_seasons() {
		let min_sacrifices = 1;
		let max_sacrifices = 3;
		let season1 = Season::default()
			.early_start(5)
			.start(45)
			.end(98)
			.min_sacrifices(min_sacrifices)
			.max_sacrifices(max_sacrifices);
		let season2 = Season::default()
			.early_start(100)
			.start(101)
			.end(199)
			.min_sacrifices(min_sacrifices)
			.max_sacrifices(max_sacrifices);

		ExtBuilder::default()
			.seasons(&[(1, season1.clone()), (2, season2.clone())])
			.mint_cooldown(0)
			.free_mints(&[(ALICE, 10)])
			.build()
			.execute_with(|| {
				run_to_block(season1.early_start + 1);
				for _ in 0..max_sacrifices {
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Free }
					));
					run_to_block(System::block_number() + 1);
				}
				run_to_block(season2.start + 2);

				for _ in 0..max_sacrifices {
					assert_ok!(AAvatars::mint(
						RuntimeOrigin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Free }
					));
					run_to_block(System::block_number() + 1);
				}
				run_to_block(season2.end + 1);

				for (player, leader, sacrifices) in [
					(ALICE, AAvatars::owners(ALICE)[0], AAvatars::owners(ALICE)[2..5].to_vec()),
					(ALICE, AAvatars::owners(ALICE)[0], AAvatars::owners(ALICE)[3..6].to_vec()),
					(ALICE, AAvatars::owners(ALICE)[5], AAvatars::owners(ALICE)[0..2].to_vec()),
				] {
					assert_noop!(
						AAvatars::forge(RuntimeOrigin::signed(player), leader, sacrifices),
						Error::<Test>::IncorrectAvatarSeason,
					);
				}
			});
	}
}

mod transferring {
	use super::*;

	#[test]
	fn transfer_free_mints_should_work() {
		ExtBuilder::default()
			.free_mints(&[(ALICE, 17), (BOB, 4)])
			.build()
			.execute_with(|| {
				assert_ok!(AAvatars::transfer_free_mints(RuntimeOrigin::signed(ALICE), BOB, 10));
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::FreeMintsTransferred { from: ALICE, to: BOB, how_many: 10 },
				));
				assert_eq!(AAvatars::accounts(ALICE).free_mints, 6);
				assert_eq!(AAvatars::accounts(BOB).free_mints, 14);

				assert_ok!(AAvatars::transfer_free_mints(RuntimeOrigin::signed(ALICE), CHARLIE, 2));
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::FreeMintsTransferred { from: ALICE, to: CHARLIE, how_many: 2 },
				));

				assert_eq!(AAvatars::accounts(ALICE).free_mints, 3);
				assert_eq!(AAvatars::accounts(CHARLIE).free_mints, 2);
			});
	}

	#[test]
	fn transfer_free_mints_should_reject_when_amount_is_lower_than_minimum_allowed() {
		ExtBuilder::default().free_mints(&[(ALICE, 11)]).build().execute_with(|| {
			let transfer = 5;
			GlobalConfigs::<Test>::mutate(|cfg| cfg.transfer.min_free_mint_transfer = transfer + 1);
			assert_noop!(
				AAvatars::transfer_free_mints(RuntimeOrigin::signed(ALICE), BOB, transfer),
				Error::<Test>::TooLowFreeMints
			);
		});
	}

	#[test]
	fn transfer_free_mints_should_reject_when_balance_is_insufficient() {
		ExtBuilder::default().free_mints(&[(ALICE, 7)]).build().execute_with(|| {
			assert_noop!(
				AAvatars::transfer_free_mints(RuntimeOrigin::signed(ALICE), BOB, 10),
				Error::<Test>::InsufficientFreeMints
			);
		});
	}

	#[test]
	fn transfer_free_mints_should_reject_transferring_to_self() {
		ExtBuilder::default().free_mints(&[(ALICE, 7)]).build().execute_with(|| {
			assert_noop!(
				AAvatars::transfer_free_mints(RuntimeOrigin::signed(ALICE), ALICE, 1),
				Error::<Test>::CannotTransferToSelf
			);
		});
	}

	#[test]
	fn transfer_avatar_works() {
		let avatar_transfer_fee = 888;
		let initial_balance = MockExistentialDeposit::get() + avatar_transfer_fee;
		let total_supply = initial_balance;
		let season_id = 999;

		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.avatar_transfer_fee(avatar_transfer_fee)
			.build()
			.execute_with(|| {
				let treasury_account = &AAvatars::treasury_account_id();
				let treasury_balance = 0;
				assert_eq!(Balances::free_balance(treasury_account), treasury_balance);
				assert_eq!(Balances::total_issuance(), total_supply);

				let mut alice_avatar_ids = create_avatars(1, ALICE, 3);
				let mut bob_avatar_ids = create_avatars(1, BOB, 6);
				let avatar_id = alice_avatar_ids[0];
				Avatars::<Test>::mutate(avatar_id, |maybe_avatar| {
					let (_, avatar) = maybe_avatar.as_mut().unwrap();
					avatar.season_id = season_id;
				});

				assert_ok!(AAvatars::transfer_avatar(RuntimeOrigin::signed(ALICE), BOB, avatar_id));
				alice_avatar_ids.retain(|x| x != &avatar_id);
				bob_avatar_ids.push(avatar_id);

				// transfer checks
				assert_eq!(AAvatars::owners(ALICE).len(), 3 - 1);
				assert_eq!(AAvatars::owners(ALICE), alice_avatar_ids);
				assert_eq!(AAvatars::owners(BOB).len(), 6 + 1);
				assert_eq!(AAvatars::owners(BOB), bob_avatar_ids);
				assert_eq!(AAvatars::avatars(avatar_id).unwrap().0, BOB);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarTransferred { from: ALICE, to: BOB, avatar_id },
				));

				// balance checks
				assert_eq!(Balances::free_balance(ALICE), initial_balance - avatar_transfer_fee);
				assert_eq!(AAvatars::treasury(season_id), avatar_transfer_fee);
				assert_eq!(
					Balances::free_balance(treasury_account),
					treasury_balance + avatar_transfer_fee
				);
				assert_eq!(Balances::total_issuance(), total_supply);

				// check organizer transfer
				GlobalConfigs::<Test>::mutate(|config| config.trade.open = false);
				Balances::make_free_balance_be(&BOB, avatar_transfer_fee);
				assert_ok!(AAvatars::set_organizer(RuntimeOrigin::root(), BOB));
				assert_ok!(AAvatars::transfer_avatar(
					RuntimeOrigin::signed(BOB),
					CHARLIE,
					bob_avatar_ids[0]
				));
				assert_eq!(AAvatars::owners(BOB).len(), 6);
				assert_eq!(AAvatars::owners(CHARLIE).len(), 1);
			});
	}

	#[test]
	fn transfer_avatar_rejects_on_transfer_closed() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.transfer.open = false);
			assert_noop!(
				AAvatars::transfer_avatar(RuntimeOrigin::signed(BOB), CHARLIE, H256::random()),
				Error::<Test>::TransferClosed
			);
		});
	}

	#[test]
	fn transfer_avatar_works_on_transfer_closed_with_organizer() {
		let avatar_transfer_fee = 135;
		ExtBuilder::default()
			.organizer(BOB)
			.balances(&[(BOB, MockExistentialDeposit::get() + avatar_transfer_fee)])
			.avatar_transfer_fee(avatar_transfer_fee)
			.build()
			.execute_with(|| {
				GlobalConfigs::<Test>::mutate(|config| config.transfer.open = false);
				let avatar_id = create_avatars(1, BOB, 1)[0];
				assert_ok!(AAvatars::transfer_avatar(RuntimeOrigin::signed(BOB), DAVE, avatar_id));
			});
	}

	#[test]
	fn transfer_avatar_rejects_transferring_to_self() {
		ExtBuilder::default().build().execute_with(|| {
			for who in [ALICE, BOB] {
				assert_noop!(
					AAvatars::transfer_avatar(RuntimeOrigin::signed(who), who, H256::random()),
					Error::<Test>::CannotTransferToSelf
				);
			}
		});
	}

	#[test]
	fn transfer_avatar_rejects_avatar_in_trade() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = create_avatars(1, CHARLIE, 1)[0];
			assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(CHARLIE), avatar_id, 999));
			assert_noop!(
				AAvatars::transfer_avatar(RuntimeOrigin::signed(CHARLIE), DAVE, avatar_id),
				Error::<Test>::AvatarInTrade
			);
		});
	}

	#[test]
	fn transfer_avatar_rejects_unowned_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = create_avatars(1, CHARLIE, 1)[0];
			assert_noop!(
				AAvatars::transfer_avatar(RuntimeOrigin::signed(ALICE), BOB, avatar_id),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn transfer_avatar_rejects_unknown_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::transfer_avatar(RuntimeOrigin::signed(ALICE), BOB, H256::random()),
				Error::<Test>::UnknownAvatar
			);
		});
	}

	#[test]
	fn transfer_avatar_rejects_on_max_ownership() {
		let avatar_transfer_fee = 369;
		ExtBuilder::default()
			.balances(&[(ALICE, MockExistentialDeposit::get() + avatar_transfer_fee)])
			.avatar_transfer_fee(avatar_transfer_fee)
			.build()
			.execute_with(|| {
				Accounts::<Test>::mutate(BOB, |info| info.storage_tier = StorageTier::Three);

				let avatar_id = create_avatars(1, ALICE, 1)[0];
				let _ = create_avatars(1, BOB, StorageTier::Three as u8);

				assert_noop!(
					AAvatars::transfer_avatar(RuntimeOrigin::signed(ALICE), BOB, avatar_id),
					Error::<Test>::MaxOwnershipReached
				);
			});
	}
}

mod trading {
	use super::*;

	#[test]
	fn set_price_should_work() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);
			let avatar_for_sale = create_avatars(1, BOB, 1)[0];
			let price = 7357;

			assert_eq!(AAvatars::trade(avatar_for_sale), None);
			assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, price));
			assert_eq!(AAvatars::trade(avatar_for_sale), Some(price));
			System::assert_last_event(mock::RuntimeEvent::AAvatars(crate::Event::AvatarPriceSet {
				avatar_id: avatar_for_sale,
				price,
			}));
		});
	}

	#[test]
	fn set_price_should_reject_when_trading_is_closed() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.trade.open = false);
			assert_noop!(
				AAvatars::set_price(RuntimeOrigin::signed(ALICE), sp_core::H256::default(), 1),
				Error::<Test>::TradeClosed,
			);
		});
	}

	#[test]
	fn set_price_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::set_price(RuntimeOrigin::none(), sp_core::H256::default(), 1),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn set_price_should_reject_incorrect_ownership() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);
			let avatar_ids = create_avatars(1, BOB, 2);

			assert_noop!(
				AAvatars::set_price(RuntimeOrigin::signed(CHARLIE), avatar_ids[0], 101),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn remove_price_should_work() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);
			let avatar_ids = create_avatars(1, BOB, 2);
			let avatar_for_sale = avatar_ids[0];
			let price = 101;

			assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, price));

			assert_eq!(AAvatars::trade(avatar_for_sale), Some(101));
			assert_ok!(AAvatars::remove_price(RuntimeOrigin::signed(BOB), avatar_for_sale));
			assert_eq!(AAvatars::trade(avatar_for_sale), None);
			System::assert_last_event(mock::RuntimeEvent::AAvatars(
				crate::Event::AvatarPriceUnset { avatar_id: avatar_for_sale },
			));
		});
	}

	#[test]
	fn remove_price_should_reject_when_trading_is_closed() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.trade.open = false);
			assert_noop!(
				AAvatars::remove_price(RuntimeOrigin::signed(ALICE), sp_core::H256::default()),
				Error::<Test>::TradeClosed,
			);
		});
	}

	#[test]
	fn remove_price_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::remove_price(RuntimeOrigin::none(), sp_core::H256::default()),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn remove_price_should_reject_incorrect_ownership() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);
			let avatar_ids = create_avatars(1, BOB, 3);
			let avatar_for_sale = avatar_ids[0];

			assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, 123));
			assert_noop!(
				AAvatars::remove_price(RuntimeOrigin::signed(CHARLIE), avatar_for_sale),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn remove_price_should_reject_unlisted_avatar() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::remove_price(RuntimeOrigin::signed(CHARLIE), sp_core::H256::default()),
				Error::<Test>::UnknownAvatarForSale,
			);
		});
	}

	#[test]
	fn buy_should_work() {
		let season = Season::default();
		let price = 310_984;
		let min_fee = 54_321;
		let alice_initial_bal = price + min_fee + 20_849;
		let bob_initial_bal = 103_598;
		let charlie_initial_bal = MockExistentialDeposit::get() + min_fee + 1357;
		let total_supply = alice_initial_bal + bob_initial_bal + charlie_initial_bal;

		ExtBuilder::default()
			.existential_deposit(0)
			.seasons(&[(1, season.clone())])
			.balances(&[
				(ALICE, alice_initial_bal),
				(BOB, bob_initial_bal),
				(CHARLIE, charlie_initial_bal),
			])
			.trade_min_fee(min_fee)
			.build()
			.execute_with(|| {
				let mut treasury_balance_season_1 = 0;
				let treasury_account = AAvatars::treasury_account_id();

				assert_eq!(AAvatars::treasury(1), treasury_balance_season_1);
				assert_eq!(Balances::free_balance(&treasury_account), treasury_balance_season_1);
				assert_eq!(Balances::total_issuance(), total_supply);

				run_to_block(season.start);
				let avatar_ids = create_avatars(1, BOB, 3);
				assert_eq!(AAvatars::treasury(1), treasury_balance_season_1);
				assert_eq!(Balances::free_balance(&treasury_account), treasury_balance_season_1);

				let owned_by_alice = AAvatars::owners(ALICE);
				let owned_by_bob = AAvatars::owners(BOB);

				let avatar_for_sale = avatar_ids[0];
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, price));
				assert_ok!(AAvatars::buy(RuntimeOrigin::signed(ALICE), avatar_for_sale));
				treasury_balance_season_1 += min_fee;

				// check for balance transfer
				assert_eq!(Balances::free_balance(ALICE), alice_initial_bal - price - min_fee);
				assert_eq!(Balances::free_balance(BOB), bob_initial_bal + price);
				assert_eq!(AAvatars::treasury(1), treasury_balance_season_1);
				assert_eq!(Balances::free_balance(&treasury_account), treasury_balance_season_1);
				assert_eq!(Balances::total_issuance(), total_supply);

				// check for ownership transfer
				assert_eq!(AAvatars::owners(ALICE).len(), owned_by_alice.len() + 1);
				assert_eq!(AAvatars::owners(BOB).len(), owned_by_bob.len() - 1);
				assert!(AAvatars::owners(ALICE).contains(&avatar_for_sale));
				assert!(!AAvatars::owners(BOB).contains(&avatar_for_sale));
				assert_eq!(AAvatars::avatars(avatar_for_sale).unwrap().0, ALICE);

				// check for removal from trade storage
				assert_eq!(AAvatars::trade(avatar_for_sale), None);

				// check for account stats
				assert_eq!(AAvatars::accounts(ALICE).stats.trade.bought, 1);
				assert_eq!(AAvatars::accounts(BOB).stats.trade.sold, 1);

				// check events
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarTraded { avatar_id: avatar_for_sale, from: BOB, to: ALICE },
				));

				// charlie buys from bob
				let avatar_for_sale = avatar_ids[1];
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, 1357));
				assert_ok!(AAvatars::buy(RuntimeOrigin::signed(CHARLIE), avatar_for_sale));
				treasury_balance_season_1 += min_fee;
				assert_eq!(AAvatars::accounts(CHARLIE).stats.trade.bought, 1);
				assert_eq!(AAvatars::accounts(BOB).stats.trade.sold, 2);

				// check season id
				let avatar_on_sale = create_avatars(33, ALICE, 1)[0];
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), avatar_on_sale, 369));
				assert_ok!(AAvatars::buy(RuntimeOrigin::signed(BOB), avatar_on_sale));
				assert_eq!(AAvatars::treasury(33), min_fee);
				assert_eq!(AAvatars::treasury(1), treasury_balance_season_1);
			});
	}

	#[test]
	fn buy_fee_should_be_calculated_correctly() {
		let season = Season::default();
		let min_fee = 123;
		let percent_fee = 30;
		let mut alice_balance = 999_999;
		let mut bob_balance = 999_999;
		let mut treasury_balance = 0;

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, alice_balance), (BOB, bob_balance)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				let avatar_ids = create_avatars(1, ALICE, 2);

				GlobalConfigs::<Test>::mutate(|cfg| {
					cfg.trade.min_fee = min_fee;
					cfg.trade.percent_fee = percent_fee;
				});

				// when price is much greater (> 30%) than min_fee, percent_fee should be charged
				let price = 9_999;
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), avatar_ids[0], price));
				assert_ok!(AAvatars::buy(RuntimeOrigin::signed(BOB), avatar_ids[0]));
				let expected_fee = price * percent_fee as u64 / 100_u64;
				bob_balance -= price + expected_fee;
				alice_balance += price;
				treasury_balance += expected_fee;
				assert_eq!(Balances::free_balance(BOB), bob_balance);
				assert_eq!(Balances::free_balance(ALICE), alice_balance);
				assert_eq!(AAvatars::treasury(1), treasury_balance);

				// when price is less than min_fee, min_fee should be charged
				let price = 100;
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), avatar_ids[1], price));
				assert_ok!(AAvatars::buy(RuntimeOrigin::signed(BOB), avatar_ids[1]));
				bob_balance -= price + min_fee;
				alice_balance += price;
				treasury_balance += min_fee;
				assert_eq!(Balances::free_balance(BOB), bob_balance);
				assert_eq!(Balances::free_balance(ALICE), alice_balance);
				assert_eq!(AAvatars::treasury(1), treasury_balance);
			});
	}

	#[test]
	fn buy_should_reject_when_trading_is_closed() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|config| config.trade.open = false);
			assert_noop!(
				AAvatars::buy(RuntimeOrigin::signed(ALICE), sp_core::H256::default()),
				Error::<Test>::TradeClosed,
			);
		});
	}

	#[test]
	fn buy_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::buy(RuntimeOrigin::none(), sp_core::H256::default()),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn buy_should_reject_unlisted_avatar() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::buy(RuntimeOrigin::signed(BOB), sp_core::H256::default()),
				Error::<Test>::UnknownAvatarForSale,
			);
		});
	}

	#[test]
	fn buy_should_reject_insufficient_balance() {
		let season = Season::default();
		let price = 310_984;

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, price - 1)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				let avatar_ids = create_avatars(1, BOB, 3);
				let avatar_for_sale = avatar_ids[0];

				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, price));
				assert_noop!(
					AAvatars::buy(RuntimeOrigin::signed(ALICE), avatar_for_sale),
					pallet_balances::Error::<Test>::InsufficientBalance
				);
			});
	}

	#[test]
	fn buy_should_reject_when_buyer_buys_its_own_avatar() {
		let season = Season::default();

		ExtBuilder::default().seasons(&[(1, season.clone())]).build().execute_with(|| {
			run_to_block(season.start);
			let avatar_ids = create_avatars(1, BOB, 3);
			let avatar_for_sale = avatar_ids[0];

			assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(BOB), avatar_for_sale, 123));
			assert_noop!(
				AAvatars::buy(RuntimeOrigin::signed(BOB), avatar_for_sale),
				Error::<Test>::AlreadyOwned
			);
		});
	}
}

mod account {
	use super::*;

	#[test]
	fn upgrade_storage_should_work() {
		let upgrade_fee = 12_345 as MockBalance;
		let num_storage_tiers = sp_std::mem::variant_count::<StorageTier>();
		let alice_balance = num_storage_tiers as MockBalance * upgrade_fee;
		let mut treasury_balance = 0;
		let total_supply = treasury_balance + alice_balance;

		ExtBuilder::default()
			.balances(&[(ALICE, alice_balance)])
			.build()
			.execute_with(|| {
				GlobalConfigs::<Test>::mutate(|cfg| cfg.account.storage_upgrade_fee = upgrade_fee);

				assert_eq!(AAvatars::accounts(ALICE).storage_tier, StorageTier::One);
				assert_eq!(AAvatars::accounts(ALICE).storage_tier as isize, 25);
				assert_eq!(
					Balances::free_balance(&AAvatars::treasury_account_id()),
					treasury_balance
				);
				assert_eq!(Balances::total_issuance(), total_supply);

				assert_ok!(AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)));
				assert_eq!(AAvatars::accounts(ALICE).storage_tier, StorageTier::Two);
				assert_eq!(AAvatars::accounts(ALICE).storage_tier as isize, 50);
				treasury_balance += upgrade_fee;
				assert_eq!(
					Balances::free_balance(&AAvatars::treasury_account_id()),
					treasury_balance
				);
				assert_eq!(Balances::total_issuance(), total_supply);

				assert_ok!(AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)));
				assert_eq!(AAvatars::accounts(ALICE).storage_tier, StorageTier::Three);
				assert_eq!(AAvatars::accounts(ALICE).storage_tier as isize, 75);
				treasury_balance += upgrade_fee;
				assert_eq!(
					Balances::free_balance(&AAvatars::treasury_account_id()),
					treasury_balance
				);
				assert_eq!(Balances::total_issuance(), total_supply);

				assert_ok!(AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)));
				assert_eq!(AAvatars::accounts(ALICE).storage_tier, StorageTier::Four);
				assert_eq!(AAvatars::accounts(ALICE).storage_tier as isize, 100);
				treasury_balance += upgrade_fee;
				assert_eq!(
					Balances::free_balance(&AAvatars::treasury_account_id()),
					treasury_balance
				);
				assert_eq!(Balances::total_issuance(), total_supply);

				assert_ok!(AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)));
				assert_eq!(AAvatars::accounts(ALICE).storage_tier, StorageTier::Five);
				assert_eq!(AAvatars::accounts(ALICE).storage_tier as isize, 150);

				assert_ok!(AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)));
				assert_eq!(AAvatars::accounts(ALICE).storage_tier, StorageTier::Max);
				assert_eq!(AAvatars::accounts(ALICE).storage_tier as isize, 200);
			});
	}

	#[test]
	fn upgrade_storage_should_reject_insufficient_balance() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)),
				pallet_balances::Error::<Test>::InsufficientBalance
			);
		});
	}

	#[test]
	fn upgrade_storage_should_reject_fully_upgraded_storage() {
		ExtBuilder::default().build().execute_with(|| {
			GlobalConfigs::<Test>::mutate(|cfg| cfg.account.storage_upgrade_fee = 0);
			Accounts::<Test>::mutate(ALICE, |account| account.storage_tier = StorageTier::Max);

			assert_noop!(
				AAvatars::upgrade_storage(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::MaxStorageTierReached
			);
		});
	}
}

mod nft_transfer {
	use super::*;
	use frame_support::{
		bounded_vec,
		traits::tokens::{nonfungibles_v2::Inspect, AttributeNamespace},
	};
	use pallet_ajuna_nft_transfer::traits::{AttributeCode, NftConvertible};

	#[test]
	fn set_collection_id_works() {
		ExtBuilder::default().organizer(CHARLIE).build().execute_with(|| {
			let collection_id = 369;
			assert_ok!(AAvatars::set_collection_id(RuntimeOrigin::signed(CHARLIE), collection_id));
			assert_eq!(AAvatars::collection_id(), Some(collection_id));
		});
	}

	#[test]
	fn set_collection_id_rejects_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::set_collection_id(RuntimeOrigin::signed(BOB), 333),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn can_lock_avatar_successfully() {
		let season = Season::default().max_components(8).max_variations(5);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(ALICE),
					MintOption { count: MintPackSize::Three, mint_type: MintType::Normal }
				));
				let avatar_ids = AAvatars::owners(ALICE);
				let avatar_id = avatar_ids[0];

				assert_ok!(AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
				assert!(LockedAvatars::<Test>::contains_key(avatar_id));
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarLocked { avatar_id },
				));

				let (_, avatar) = AAvatars::avatars(avatar_id).unwrap();
				assert_eq!(
					avatar,
					Avatar {
						season_id: 1,
						dna: bounded_vec![0x02, 0x04, 0x00, 0x03, 0x01, 0x12, 0x00, 0x23],
						souls: 64
					}
				);

				// Ensure correct encoding
				assert_eq!(
					<Nft as Inspect<MockAccountId>>::typed_attribute::<
						pallet_ajuna_nft_transfer::traits::ItemCode,
						Avatar,
					>(
						&AAvatars::collection_id().unwrap(),
						&avatar_id,
						&AttributeNamespace::Pallet,
						&<Avatar as NftConvertible>::ITEM_CODE,
					)
					.unwrap(),
					avatar,
				);

				// Ensure attributes encoding
				for (attribute_code, encoded_attribute) in
					Avatar::get_attribute_codes().iter().zip([
						avatar.dna.encode(),
						avatar.souls.encode(),
						RarityTier::Common.encode(),
						Force::Thermal.encode(),
					]) {
					assert_eq!(
						<Nft as Inspect<MockAccountId>>::typed_attribute::<AttributeCode, Vec<u8>>(
							&AAvatars::collection_id().unwrap(),
							&avatar_id,
							&AttributeNamespace::Pallet,
							attribute_code,
						)
						.unwrap(),
						encoded_attribute
					);
				}

				// Ensure ownership transferred to technical account
				let technical_account = AAvatars::technical_account_id();
				assert!(!AAvatars::owners(ALICE).contains(&avatar_id));
				assert_eq!(AAvatars::owners(technical_account)[0], avatar_id);
				assert_eq!(AAvatars::avatars(avatar_id).unwrap().0, technical_account);

				// Ensure locked avatars cannot be used in trading

				// Ensure locked avatars cannot be used in trading, transferring and forging
				for extrinsic in [
					AAvatars::set_price(RuntimeOrigin::signed(technical_account), avatar_id, 1_000),
					AAvatars::transfer_avatar(
						RuntimeOrigin::signed(technical_account),
						BOB,
						avatar_id,
					),
					AAvatars::forge(
						RuntimeOrigin::signed(technical_account),
						avatar_id,
						avatar_ids[1..3].to_vec(),
					),
				] {
					assert_noop!(extrinsic, Error::<Test>::AvatarLocked);
				}
			});
	}

	#[test]
	fn cannot_lock_when_nft_transfer_is_closed() {
		ExtBuilder::default()
			.balances(&[(ALICE, MockExistentialDeposit::get())])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, ALICE, 1)[0];
				GlobalConfigs::<Test>::mutate(|config| config.nft_transfer.open = false);
				assert_noop!(
					AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					Error::<Test>::NftTransferClosed
				);
			})
	}

	#[test]
	fn cannot_lock_unowned_avatar() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000_000_000_000), (BOB, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, BOB, 1)[0];
				assert_noop!(
					AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					Error::<Test>::Ownership
				);
			});
	}

	#[test]
	fn cannot_lock_avatar_on_trade() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, ALICE, 1)[0];
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), avatar_id, 1_000));
				assert_noop!(
					AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					Error::<Test>::AvatarInTrade
				);
			});
	}

	#[test]
	fn cannot_lock_already_locked_avatar() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, ALICE, 1)[0];
				assert_ok!(AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
				assert_noop!(
					AAvatars::lock_avatar(
						RuntimeOrigin::signed(AAvatars::technical_account_id()),
						avatar_id
					),
					Error::<Test>::AvatarLocked
				);
			});
	}

	#[test]
	fn can_unlock_avatar_successfully() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, ALICE, 1)[0];
				assert_ok!(AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
				assert_ok!(AAvatars::unlock_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
				assert_eq!(LockedAvatars::<Test>::get(avatar_id), None);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarUnlocked { avatar_id },
				));
			});
	}

	#[test]
	fn cannot_unlock_when_nft_transfer_is_closed() {
		ExtBuilder::default()
			.balances(&[(ALICE, MockExistentialDeposit::get())])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, ALICE, 1)[0];
				assert_ok!(AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
				GlobalConfigs::<Test>::mutate(|config| config.nft_transfer.open = false);
				assert_noop!(
					AAvatars::unlock_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					Error::<Test>::NftTransferClosed
				);
			})
	}

	#[test]
	fn cannot_unlock_unowned_avatar() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000_000_000_000), (BOB, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				let avatar_id = create_avatars(1, BOB, 1)[0];
				assert_ok!(AAvatars::lock_avatar(RuntimeOrigin::signed(BOB), avatar_id));
				assert_noop!(
					AAvatars::unlock_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					pallet_nfts::Error::<Test>::NoPermission
				);
			});
	}

	#[test]
	fn cannot_unlock_transferred_avatar() {
		let season = Season::default();

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, 1_000_000_000_000)])
			.create_nft_collection(true)
			.build()
			.execute_with(|| {
				run_to_block(season.start);

				let avatar_id = create_avatars(1, ALICE, 1)[0];
				assert_ok!(AAvatars::lock_avatar(RuntimeOrigin::signed(ALICE), avatar_id));

				pallet_ajuna_nft_transfer::NftStatuses::<Test>::insert(
					AAvatars::collection_id().unwrap(),
					avatar_id,
					pallet_ajuna_nft_transfer::NftStatus::Uploaded,
				);

				assert_noop!(
					AAvatars::unlock_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					pallet_ajuna_nft_transfer::Error::<Test>::NftOutsideOfChain
				);
			});
	}
}

mod fix_variation {
	use super::*;

	#[test]
	fn fix_variation_successfully() {
		let season = Season::default();

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.balances(&[(ALICE, 1_000_000_000_000)])
			.build()
			.execute_with(|| {
				run_to_block(season.start);

				let avatar_id = create_avatars(1, ALICE, 1)[0];

				let (_, mut avatar_before) = Avatars::<Test>::get(avatar_id).unwrap();

				avatar_before.dna[1] = 0b0001_1100;
				avatar_before.dna[2] = 0b0100_0010;

				Avatars::<Test>::insert(avatar_id, (ALICE, avatar_before));

				assert_ok!(AAvatars::fix_variation(RuntimeOrigin::signed(ALICE), avatar_id));

				let (_, avatar_after) = Avatars::<Test>::get(avatar_id).unwrap();
				assert_eq!(avatar_after.dna[1], 0b0001_1100);
				assert_eq!(avatar_after.dna[2], 0b0100_1100);
			});
	}
}
