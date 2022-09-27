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
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_runtime::{ArithmeticError, DispatchError};

mod organizer {
	use super::*;

	#[test]
	fn set_organizer_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(AwesomeAvatars::organizer(), None);
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), HILDA));
			assert_eq!(AwesomeAvatars::organizer(), Some(HILDA), "Organizer should be Hilda");
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::OrganizerSet {
				organizer: HILDA,
			}));
		});
	}

	#[test]
	fn set_organizer_should_reject_non_root_caller() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::set_organizer(Origin::signed(ALICE), HILDA),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_existing_organizer() {
		ExtBuilder::default().organizer(BOB).build().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), FLORINA));
			assert_eq!(AwesomeAvatars::organizer(), Some(FLORINA), "Organizer should be Florina");
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::OrganizerSet {
				organizer: FLORINA,
			}));
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_no_organizer_set() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(AwesomeAvatars::organizer(), None);
			assert_noop!(
				AwesomeAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_caller_is_not_organizer() {
		ExtBuilder::default().organizer(ERIN).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		ExtBuilder::default().organizer(CHARLIE).build().execute_with(|| {
			assert_ok!(AwesomeAvatars::ensure_organizer(Origin::signed(CHARLIE)));
		});
	}
}

mod season {
	use super::*;

	#[test]
	fn upsert_season_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::upsert_season(Origin::signed(BOB), 7357, Season::default()),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn upsert_season_should_work() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, Season::default())])
			.build()
			.execute_with(|| {
				let season_1 = Season::default().early_start(1).start(5).end(10);
				assert_ok!(AwesomeAvatars::upsert_season(
					Origin::signed(ALICE),
					1,
					season_1.clone()
				));
				assert_eq!(AwesomeAvatars::seasons(1), Some(season_1.clone()));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::UpdatedSeason { season_id: 1, season: season_1 },
				));

				let season_2 = Season::default().early_start(11).start(12).end(13);
				assert_ok!(AwesomeAvatars::upsert_season(
					Origin::signed(ALICE),
					2,
					season_2.clone()
				));
				assert_eq!(AwesomeAvatars::seasons(2), Some(season_2.clone()));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::UpdatedSeason { season_id: 2, season: season_2 },
				));
			});
	}

	#[test]
	fn upsert_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		let season_1 = Season::default();
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, season_1.clone())])
			.build()
			.execute_with(|| {
				for i in 0..season_1.end {
					let season_2 = Season::default().early_start(i).start(i + 1).end(i + 2);
					assert!(season_2.early_start <= season_1.end);
					assert_noop!(
						AwesomeAvatars::upsert_season(Origin::signed(ALICE), 2, season_2),
						Error::<Test>::EarlyStartTooEarly
					);
				}
			});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_is_earlier_than_or_equal_to_start() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for i in 3..6 {
				let new_season = Season::default().early_start(i).start(3).end(10);
				assert!(new_season.early_start >= new_season.start);
				assert_noop!(
					AwesomeAvatars::upsert_season(Origin::signed(ALICE), 1, new_season),
					Error::<Test>::EarlyStartTooLate
				);
			}
		});
	}

	#[test]
	fn upsert_season_should_return_error_when_start_is_later_than_end() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let new_season = Season::default().early_start(11).start(12).end(10);
			assert!(new_season.early_start < new_season.start);
			assert_noop!(
				AwesomeAvatars::upsert_season(Origin::signed(ALICE), 1, new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn upsert_season_should_return_error_when_rarity_tier_is_duplicated() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for duplicated_rarity_tiers in [
				test_rarity_tiers(vec![(RarityTier::Common, 1), (RarityTier::Common, 99)]),
				test_rarity_tiers(vec![
					(RarityTier::Common, 10),
					(RarityTier::Common, 10),
					(RarityTier::Legendary, 80),
				]),
			] {
				for season in [
					Season::default().rarity_tiers_single_mint(duplicated_rarity_tiers.clone()),
					Season::default().rarity_tiers_batch_mint(duplicated_rarity_tiers),
				] {
					assert_noop!(
						AwesomeAvatars::upsert_season(Origin::signed(ALICE), 1, season),
						Error::<Test>::DuplicatedRarityTier
					);
				}
			}
		});
	}

	#[test]
	fn upsert_season_should_return_error_when_sum_of_rarity_chance_is_incorrect() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for incorrect_rarity_tiers in [
				test_rarity_tiers(vec![]),
				test_rarity_tiers(vec![(RarityTier::Common, 10), (RarityTier::Common, 10)]),
				test_rarity_tiers(vec![(RarityTier::Common, 100), (RarityTier::Common, 100)]),
				test_rarity_tiers(vec![
					(RarityTier::Common, 70),
					(RarityTier::Uncommon, 20),
					(RarityTier::Rare, 9),
				]),
				test_rarity_tiers(vec![
					(RarityTier::Epic, 70),
					(RarityTier::Legendary, 20),
					(RarityTier::Mythical, 11),
				]),
			] {
				for season in [
					Season::default().rarity_tiers_single_mint(incorrect_rarity_tiers.clone()),
					Season::default().rarity_tiers_batch_mint(incorrect_rarity_tiers),
				] {
					assert_noop!(
						AwesomeAvatars::upsert_season(Origin::signed(ALICE), 1, season,),
						Error::<Test>::IncorrectRarityPercentages
					);
				}
			}
		});
	}

	#[test]
	fn upsert_season_should_return_error_when_season_to_update_ends_after_next_season_start() {
		let season_1 = Season::default().early_start(1).start(5).end(10);
		let season_2 = Season::default().early_start(11).start(15).end(20);

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, season_1), (2, season_2.clone())])
			.build()
			.execute_with(|| {
				let season_1_update = Season::default().early_start(1).start(5).end(14);
				assert!(season_1_update.end > season_2.early_start);
				assert_noop!(
					AwesomeAvatars::upsert_season(Origin::signed(ALICE), 1, season_1_update),
					Error::<Test>::SeasonEndTooLate
				);
			});
	}

	#[test]
	fn upsert_season_should_handle_underflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::upsert_season(
					Origin::signed(ALICE),
					SeasonId::MIN,
					Season::default()
				),
				ArithmeticError::Underflow
			);
		});
	}

	#[test]
	fn upsert_season_should_handle_overflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::upsert_season(
					Origin::signed(ALICE),
					SeasonId::MAX,
					Season::default()
				),
				ArithmeticError::Overflow
			);
		});
	}

	#[test]
	fn upsert_season_should_return_error_when_season_ids_are_not_sequential() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, Season::default())])
			.build()
			.execute_with(|| {
				assert_noop!(
					AwesomeAvatars::upsert_season(Origin::signed(ALICE), 3, Season::default()),
					Error::<Test>::NonSequentialSeasonId,
				);
			});
	}
}

mod config {
	use super::*;

	#[test]
	fn update_global_config_should_work() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let config = GlobalConfigOf::<Test>::default();
			assert_ok!(AwesomeAvatars::update_global_config(Origin::signed(ALICE), config.clone()));
			System::assert_last_event(mock::Event::AwesomeAvatars(
				crate::Event::UpdatedGlobalConfig(config),
			));
		});
	}

	#[test]
	fn update_global_config_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_global_config(
					Origin::signed(BOB),
					GlobalConfigOf::<Test>::default()
				),
				DispatchError::BadOrigin
			);
		});
	}
}

mod minting {
	use super::*;
	use frame_support::traits::Currency;

	#[test]
	fn mint_should_work() {
		let max_components = 7;
		let season = Season::default().end(20).max_components(max_components);

		let expected_nonce_increment = 1 as MockIndex;
		let mut expected_nonce = 0;
		let mut initial_balance = 1_234_567_890_123_456;
		let mut initial_free_mints = 11;
		let mut owned_avatars = 0;
		let fees = MintFees { one: 12, three: 34, six: 56 };
		let mint_cooldown = 1;
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, season.clone())])
			.mint_availability(true)
			.mint_fees(fees)
			.mint_cooldown(mint_cooldown)
			.balances(vec![(ALICE, initial_balance)])
			.free_mints(vec![(ALICE, initial_free_mints)])
			.build()
			.execute_with(|| {
				for mint_type in [MintType::Normal, MintType::Free] {
					// initial checks
					match mint_type {
						MintType::Normal =>
							assert_eq!(Balances::total_balance(&ALICE), initial_balance),
						MintType::Free =>
							assert_eq!(AwesomeAvatars::free_mints(ALICE), initial_free_mints),
					}
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AwesomeAvatars::owners(ALICE).len(), owned_avatars);
					assert!(!AwesomeAvatars::is_season_active());

					// single mint
					run_to_block(season.early_start + 1);
					assert_ok!(AwesomeAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: mint_type.clone() }
					));
					match mint_type {
						MintType::Normal => {
							initial_balance -= fees.fee_for(MintPackSize::One);
							assert_eq!(Balances::total_balance(&ALICE), initial_balance);
						},
						MintType::Free => {
							initial_free_mints -= MintPackSize::One as MintCount;
							assert_eq!(AwesomeAvatars::free_mints(ALICE), initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment;
					owned_avatars += 1;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AwesomeAvatars::owners(ALICE).len(), owned_avatars);
					assert!(AwesomeAvatars::is_season_active());
					System::assert_has_event(mock::Event::AwesomeAvatars(
						crate::Event::SeasonStarted(1),
					));
					System::assert_last_event(mock::Event::AwesomeAvatars(
						crate::Event::AvatarsMinted {
							avatar_ids: vec![AwesomeAvatars::owners(ALICE)[0]],
						},
					));

					// batch mint: three
					run_to_block(System::block_number() + 1 + mint_cooldown);
					assert_ok!(AwesomeAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::Three, mint_type: mint_type.clone() }
					));
					match mint_type {
						MintType::Normal => {
							initial_balance -= fees.fee_for(MintPackSize::Three);
							assert_eq!(Balances::total_balance(&ALICE), initial_balance);
						},
						MintType::Free => {
							initial_free_mints -= MintPackSize::Three as MintCount;
							assert_eq!(AwesomeAvatars::free_mints(ALICE), initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment * 3;
					owned_avatars += 3;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AwesomeAvatars::owners(ALICE).len(), owned_avatars);
					assert!(AwesomeAvatars::is_season_active());
					System::assert_last_event(mock::Event::AwesomeAvatars(
						crate::Event::AvatarsMinted {
							avatar_ids: AwesomeAvatars::owners(ALICE)[1..=3].to_vec(),
						},
					));

					// batch mint: six
					run_to_block(System::block_number() + 1 + mint_cooldown);
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_ok!(AwesomeAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::Six, mint_type: mint_type.clone() }
					));
					match mint_type {
						MintType::Normal => {
							initial_balance -= fees.fee_for(MintPackSize::Six);
							assert_eq!(Balances::total_balance(&ALICE), initial_balance);
						},
						MintType::Free => {
							initial_free_mints -= MintPackSize::Six as MintCount;
							assert_eq!(AwesomeAvatars::free_mints(ALICE), initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment * 6;
					owned_avatars += 6;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AwesomeAvatars::owners(ALICE).len(), owned_avatars);
					assert!(AwesomeAvatars::is_season_active());
					System::assert_last_event(mock::Event::AwesomeAvatars(
						crate::Event::AvatarsMinted {
							avatar_ids: AwesomeAvatars::owners(ALICE)[4..=9].to_vec(),
						},
					));

					// check for season ending
					run_to_block(season.end + 1);
					assert_err!(
						AwesomeAvatars::mint(
							Origin::signed(ALICE),
							MintOption { count: MintPackSize::One, mint_type }
						),
						Error::<Test>::OutOfSeason
					);

					// reset for next iteration
					System::set_block_number(0);
					LastMintedBlockNumbers::<Test>::remove(ALICE);
					Owners::<Test>::remove(ALICE);
					IsSeasonActive::<Test>::set(false);
					CurrentSeasonId::<Test>::set(1);
					owned_avatars = 0;
				}
			});
	}

	#[test]
	fn mint_should_work_when_rare_tier_avatars_are_minted() {
		let season_1 = Season::default().early_start(10).start(11).end(12).rarity_tiers_batch_mint(
			test_rarity_tiers(vec![(RarityTier::Common, 99), (RarityTier::Legendary, 1)]),
		);
		let season_2 = Season::default().early_start(13).start(14).end(15).rarity_tiers_batch_mint(
			test_rarity_tiers(vec![(RarityTier::Common, 50), (RarityTier::Legendary, 50)]),
		);
		let season_3 = Season::default().early_start(16).start(17).end(18).rarity_tiers_batch_mint(
			test_rarity_tiers(vec![(RarityTier::Common, 50), (RarityTier::Mythical, 50)]),
		);
		let seasons = vec![(1, season_1.clone()), (2, season_2.clone()), (3, season_3.clone())];

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(seasons)
			.mint_availability(true)
			.mint_cooldown(0)
			.mint_fees(MintFees { one: 0, three: 0, six: 0 })
			.build()
			.execute_with(|| {
				let count_high_tier = |season_id: SeasonId| -> MintCount {
					AwesomeAvatars::owners(ALICE)
						.into_iter()
						.map(|avatar_id| {
							let (_player, avatar) = AwesomeAvatars::avatars(avatar_id).unwrap();
							if avatar.season == season_id &&
								avatar
									.dna
									.into_iter()
									.all(|x| (x >> 4) >= RarityTier::Legendary as u8)
							{
								1
							} else {
								0
							}
						})
						.sum()
				};

				run_to_block(season_1.early_start + 1);
				assert_eq!(AwesomeAvatars::active_season_rare_mints(), 0);
				assert_ok!(AwesomeAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));
				let season_1_high_tiers = count_high_tier(1);
				assert_eq!(season_1_high_tiers, 0);
				assert_eq!(AwesomeAvatars::active_season_rare_mints(), season_1_high_tiers);

				run_to_block(season_2.early_start + 1);
				assert_ok!(AwesomeAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));
				let season_2_high_tiers = count_high_tier(2);
				assert_eq!(season_2_high_tiers, 4);
				assert_eq!(AwesomeAvatars::active_season_rare_mints(), season_2_high_tiers);
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::RareAvatarsMinted { count: count_high_tier(2) },
				));

				run_to_block(season_3.early_start + 1);
				assert_ok!(AwesomeAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));
				let season_3_high_tiers = count_high_tier(3);
				assert_eq!(season_3_high_tiers, 2);
				// assert_eq!(AwesomeAvatars::active_season_rare_mints(), season_3_high_tiers);
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::RareAvatarsMinted { count: count_high_tier(3) },
				));
			});
	}

	#[test]
	fn mint_should_return_error_when_minting_is_unavailable() {
		ExtBuilder::default().mint_availability(false).build().execute_with(|| {
			for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				for mint_type in [MintType::Normal, MintType::Free] {
					assert_noop!(
						AwesomeAvatars::mint(
							Origin::signed(ALICE),
							MintOption { count, mint_type }
						),
						Error::<Test>::MintUnavailable
					);
				}
			}
		});
	}

	#[test]
	fn mint_should_reject_unsigned_caller() {
		ExtBuilder::default().build().execute_with(|| {
			for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				for mint_type in [MintType::Normal, MintType::Free] {
					assert_noop!(
						AwesomeAvatars::mint(Origin::none(), MintOption { count, mint_type }),
						DispatchError::BadOrigin
					);
				}
			}
		});
	}

	#[test]
	fn mint_should_return_error_when_season_is_inactive() {
		ExtBuilder::default()
			.organizer(ALICE)
			.mint_availability(true)
			.balances(vec![(ALICE, 1_234_567_890_123_456)])
			.free_mints(vec![(ALICE, 10)])
			.build()
			.execute_with(|| {
				for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AwesomeAvatars::mint(
								Origin::signed(ALICE),
								MintOption { count, mint_type }
							),
							Error::<Test>::OutOfSeason
						);
					}
				}
			});
	}

	#[test]
	fn mint_should_return_error_when_max_ownership_has_reached() {
		let avatar_ids = BoundedAvatarIdsOf::<Test>::try_from(
			(0..MAX_AVATARS_PER_PLAYER)
				.map(|_| sp_core::H256::default())
				.collect::<Vec<_>>(),
		)
		.unwrap();
		assert!(avatar_ids.is_full());

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(SeasonId::default(), Season::default())])
			.mint_availability(true)
			.balances(vec![(ALICE, 1_234_567_890_123_456)])
			.free_mints(vec![(ALICE, 10)])
			.build()
			.execute_with(|| {
				run_to_block(2);
				Owners::<Test>::insert(ALICE, avatar_ids);
				for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AwesomeAvatars::mint(
								Origin::signed(ALICE),
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
			.organizer(ALICE)
			.seasons(vec![(1, season.clone())])
			.mint_availability(true)
			.mint_cooldown(mint_cooldown)
			.balances(vec![(ALICE, 1_234_567_890_123_456)])
			.free_mints(vec![(ALICE, 10)])
			.build()
			.execute_with(|| {
				for mint_type in [MintType::Normal, MintType::Free] {
					run_to_block(season.start + 1);
					assert_ok!(AwesomeAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: mint_type.clone() }
					));

					for _ in 0..mint_cooldown {
						run_to_block(System::block_number() + 1);
						assert_noop!(
							AwesomeAvatars::mint(
								Origin::signed(ALICE),
								MintOption {
									count: MintPackSize::One,
									mint_type: mint_type.clone()
								}
							),
							Error::<Test>::MintCooldown
						);
					}

					run_to_block(System::block_number() + 1);
					assert_eq!(System::block_number(), (season.start + 1) + (mint_cooldown + 1));
					assert_ok!(AwesomeAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
					));

					// reset for next iteration
					System::set_block_number(0);
					LastMintedBlockNumbers::<Test>::remove(ALICE);
				}
			});
	}

	#[test]
	fn mint_should_return_error_on_insufficient_funds() {
		let season = Season::default().end(20);

		ExtBuilder::default()
			.organizer(ALICE)
			.mint_availability(true)
			.seasons(vec![(1, season)])
			.build()
			.execute_with(|| {
				for mint_count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for (mint_type, error) in [
						(MintType::Normal, Error::<Test>::InsufficientFunds),
						(MintType::Free, Error::<Test>::InsufficientFreeMints),
					] {
						assert_err!(
							AwesomeAvatars::mint(
								Origin::signed(ALICE),
								MintOption { count: mint_count, mint_type }
							),
							error,
						);
					}
				}
			});
	}

	#[test]
	fn free_mint_is_still_possible_on_premature_season_end() {
		let season = Season::default()
			.start(1)
			.end(20)
			.rarity_tiers_single_mint(test_rarity_tiers(vec![(RarityTier::Mythical, 100)]));

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, season.clone())])
			.mint_availability(true)
			.free_mints(vec![(ALICE, 100)])
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_eq!(
					AwesomeAvatars::seasons(AwesomeAvatars::current_season_id()),
					Some(season.clone())
				);
				assert_ok!(AwesomeAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::One, mint_type: MintType::Free }
				));
				System::assert_has_event(mock::Event::AwesomeAvatars(
					crate::Event::AvatarsMinted {
						avatar_ids: vec![AwesomeAvatars::owners(ALICE)[0]],
					},
				));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::RareAvatarsMinted { count: 1 },
				));

				run_to_block(season.start + 10);
				assert_eq!(
					AwesomeAvatars::seasons(AwesomeAvatars::current_season_id()),
					Some(season)
				);
				assert_ok!(AwesomeAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::One, mint_type: MintType::Free }
				));

				System::assert_has_event(mock::Event::AwesomeAvatars(
					crate::Event::AvatarsMinted {
						avatar_ids: vec![AwesomeAvatars::owners(ALICE)[1]],
					},
				));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::RareAvatarsMinted { count: 1 },
				));
			});
	}

	#[test]
	fn transfer_free_mints_should_work() {
		ExtBuilder::default()
			.free_mints(vec![(ALICE, 17), (BOB, 4)])
			.build()
			.execute_with(|| {
				assert_ok!(AwesomeAvatars::transfer_free_mints(Origin::signed(ALICE), BOB, 10));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::FreeMintsTransferred { from: ALICE, to: BOB, how_many: 10 },
				));
				assert_eq!(AwesomeAvatars::free_mints(ALICE), 6);
				assert_eq!(AwesomeAvatars::free_mints(BOB), 14);

				assert_ok!(AwesomeAvatars::transfer_free_mints(Origin::signed(ALICE), CHARLIE, 2));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::FreeMintsTransferred { from: ALICE, to: CHARLIE, how_many: 2 },
				));

				assert_eq!(AwesomeAvatars::free_mints(ALICE), 3);
				assert_eq!(AwesomeAvatars::free_mints(CHARLIE), 2);
			});
	}

	#[test]
	fn transfer_free_mints_should_return_error_if_not_enough_funds_available() {
		ExtBuilder::default().free_mints(vec![(ALICE, 7)]).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::transfer_free_mints(Origin::signed(ALICE), BOB, 10),
				Error::<Test>::InsufficientFreeMints
			);
		});
	}

	#[test]
	fn issue_free_mints_should_work() {
		ExtBuilder::default()
			.organizer(ALICE)
			.free_mints(vec![(ALICE, 7)])
			.build()
			.execute_with(|| {
				assert_eq!(AwesomeAvatars::free_mints(BOB), 0);

				assert_ok!(AwesomeAvatars::issue_free_mints(Origin::signed(ALICE), BOB, 7));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::FreeMintsIssued { to: BOB, how_many: 7 },
				));

				assert_eq!(AwesomeAvatars::free_mints(BOB), 7);

				assert_ok!(AwesomeAvatars::issue_free_mints(Origin::signed(ALICE), BOB, 3));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::FreeMintsIssued { to: BOB, how_many: 3 },
				));

				assert_eq!(AwesomeAvatars::free_mints(BOB), 10);
			});
	}
}
