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
			assert_eq!(AAvatars::organizer(), None);
			assert_ok!(AAvatars::set_organizer(Origin::root(), HILDA));
			assert_eq!(AAvatars::organizer(), Some(HILDA), "Organizer should be Hilda");
			System::assert_last_event(mock::Event::AAvatars(crate::Event::OrganizerSet {
				organizer: HILDA,
			}));
		});
	}

	#[test]
	fn set_organizer_should_reject_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::set_organizer(Origin::signed(ALICE), HILDA),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_existing_organizer() {
		ExtBuilder::default().organizer(BOB).build().execute_with(|| {
			assert_ok!(AAvatars::set_organizer(Origin::root(), FLORINA));
			assert_eq!(AAvatars::organizer(), Some(FLORINA), "Organizer should be Florina");
			System::assert_last_event(mock::Event::AAvatars(crate::Event::OrganizerSet {
				organizer: FLORINA,
			}));
		});
	}

	#[test]
	fn ensure_organizer_should_reject_whe_no_organizer_is_set() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(AAvatars::organizer(), None);
			assert_noop!(
				AAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ERIN).build().execute_with(|| {
			assert_noop!(
				AAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		ExtBuilder::default().organizer(CHARLIE).build().execute_with(|| {
			assert_ok!(AAvatars::ensure_organizer(Origin::signed(CHARLIE)));
		});
	}
}

mod season {
	use super::*;

	#[test]
	fn season_validate_should_mutate_correctly() {
		let mut season = Season::default()
			.tiers(vec![RarityTier::Rare, RarityTier::Common, RarityTier::Epic])
			.p_single_mint(vec![20, 80])
			.p_batch_mint(vec![60, 40]);
		assert_ok!(season.validate::<Test>());

		// check for ascending order sort
		assert_eq!(
			season.tiers.to_vec(),
			vec![RarityTier::Common, RarityTier::Rare, RarityTier::Epic]
		);

		// check for descending order sort
		assert_eq!(season.p_single_mint.to_vec(), vec![80, 20]);
		assert_eq!(season.p_batch_mint.to_vec(), vec![60, 40]);
	}

	#[test]
	fn upsert_season_should_work() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, Season::default())])
			.build()
			.execute_with(|| {
				let season_1 = Season::default().early_start(1).start(5).end(10);
				assert_ok!(AAvatars::upsert_season(Origin::signed(ALICE), 1, season_1.clone()));
				assert_eq!(AAvatars::seasons(1), Some(season_1.clone()));
				System::assert_last_event(mock::Event::AAvatars(crate::Event::UpdatedSeason {
					season_id: 1,
					season: season_1,
				}));

				let season_2 = Season::default().early_start(11).start(12).end(13);
				assert_ok!(AAvatars::upsert_season(Origin::signed(ALICE), 2, season_2.clone()));
				assert_eq!(AAvatars::seasons(2), Some(season_2.clone()));
				System::assert_last_event(mock::Event::AAvatars(crate::Event::UpdatedSeason {
					season_id: 2,
					season: season_2,
				}));
			});
	}

	#[test]
	fn upsert_season_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::upsert_season(Origin::signed(BOB), 7357, Season::default()),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn upsert_season_should_reject_when_early_start_is_earlier_than_previous_season_end() {
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
						AAvatars::upsert_season(Origin::signed(ALICE), 2, season_2),
						Error::<Test>::EarlyStartTooEarly
					);
				}
			});
	}

	#[test]
	fn upsert_season_should_reject_when_early_start_is_earlier_than_or_equal_to_start() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for i in 3..6 {
				let new_season = Season::default().early_start(i).start(3).end(10);
				assert!(new_season.early_start >= new_season.start);
				assert_noop!(
					AAvatars::upsert_season(Origin::signed(ALICE), 1, new_season),
					Error::<Test>::EarlyStartTooLate
				);
			}
		});
	}

	#[test]
	fn upsert_season_should_reject_when_start_is_later_than_end() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let new_season = Season::default().early_start(11).start(12).end(10);
			assert!(new_season.early_start < new_season.start);
			assert_noop!(
				AAvatars::upsert_season(Origin::signed(ALICE), 1, new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn upsert_season_should_reject_when_rarity_tier_is_duplicated() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for duplicated_rarity_tiers in [
				vec![RarityTier::Common, RarityTier::Common],
				vec![RarityTier::Common, RarityTier::Common, RarityTier::Legendary],
			] {
				assert_noop!(
					AAvatars::upsert_season(
						Origin::signed(ALICE),
						1,
						Season::default().tiers(duplicated_rarity_tiers)
					),
					Error::<Test>::DuplicatedRarityTier
				);
			}
		});
	}

	#[test]
	fn upsert_season_should_reject_when_sum_of_rarity_chance_is_incorrect() {
		let tiers = vec![RarityTier::Common, RarityTier::Uncommon, RarityTier::Legendary];
		let season_0 = Season::default().tiers(tiers.clone());
		let season_1 = Season::default().tiers(tiers);
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for incorrect_percentages in [vec![12, 39], vec![123, 10], vec![83, 1, 43]] {
				for season in [
					season_0.clone().p_single_mint(incorrect_percentages.clone()),
					season_1.clone().p_single_mint(incorrect_percentages),
				] {
					assert_noop!(
						AAvatars::upsert_season(Origin::signed(ALICE), 1, season),
						Error::<Test>::IncorrectRarityPercentages
					);
				}
			}
		});
	}

	#[test]
	fn upsert_season_should_reject_when_season_to_update_ends_after_next_season_start() {
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
					AAvatars::upsert_season(Origin::signed(ALICE), 1, season_1_update),
					Error::<Test>::SeasonEndTooLate
				);
			});
	}

	#[test]
	fn upsert_season_should_reject_season_id_underflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::upsert_season(Origin::signed(ALICE), SeasonId::MIN, Season::default()),
				ArithmeticError::Underflow
			);
		});
	}

	#[test]
	fn upsert_season_should_reject_season_id_overflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::upsert_season(Origin::signed(ALICE), SeasonId::MAX, Season::default()),
				ArithmeticError::Overflow
			);
		});
	}

	#[test]
	fn upsert_season_should_reject_out_of_bound_variations() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for (season, error) in [
				(Season::default().max_variations(0), Error::<Test>::MaxVariationsTooLow),
				(Season::default().max_variations(1), Error::<Test>::MaxVariationsTooLow),
				(Season::default().max_variations(16), Error::<Test>::MaxVariationsTooHigh),
				(Season::default().max_variations(100), Error::<Test>::MaxVariationsTooHigh),
			] {
				assert_noop!(AAvatars::upsert_season(Origin::signed(ALICE), 1, season), error);
			}
		});
	}

	#[test]
	fn upsert_season_should_reject_out_of_bound_components_bounds() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for (season, error) in [
				(Season::default().max_components(0), Error::<Test>::MaxComponentsTooLow),
				(Season::default().max_components(1), Error::<Test>::MaxComponentsTooLow),
				(Season::default().max_components(33), Error::<Test>::MaxComponentsTooHigh),
				(Season::default().max_components(100), Error::<Test>::MaxComponentsTooHigh),
			] {
				assert_noop!(AAvatars::upsert_season(Origin::signed(ALICE), 1, season), error);
			}
		});
	}

	#[test]
	fn upsert_season_should_reject_when_season_ids_are_not_sequential() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![(1, Season::default())])
			.build()
			.execute_with(|| {
				assert_noop!(
					AAvatars::upsert_season(Origin::signed(ALICE), 3, Season::default()),
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
			assert_ok!(AAvatars::update_global_config(Origin::signed(ALICE), config.clone()));
			System::assert_last_event(mock::Event::AAvatars(crate::Event::UpdatedGlobalConfig(
				config,
			)));
		});
	}

	#[test]
	fn update_global_config_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AAvatars::update_global_config(
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
		let mut owned_avatar_count = 0;
		let fees = MintFees { one: 12, three: 34, six: 56 };
		let mint_cooldown = 1;
		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
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
							assert_eq!(AAvatars::free_mints(ALICE), initial_free_mints),
					}
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert!(!AAvatars::is_season_active());

					// single mint
					run_to_block(season.early_start + 1);
					assert_ok!(AAvatars::mint(
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
							assert_eq!(AAvatars::free_mints(ALICE), initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment;
					owned_avatar_count += 1;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert!(AAvatars::is_season_active());
					System::assert_has_event(mock::Event::AAvatars(crate::Event::SeasonStarted(1)));
					System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarsMinted {
						avatar_ids: vec![AAvatars::owners(ALICE)[0]],
					}));

					// batch mint: three
					run_to_block(System::block_number() + 1 + mint_cooldown);
					assert_ok!(AAvatars::mint(
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
							assert_eq!(AAvatars::free_mints(ALICE), initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment * 3;
					owned_avatar_count += 3;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert!(AAvatars::is_season_active());
					System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarsMinted {
						avatar_ids: AAvatars::owners(ALICE)[1..=3].to_vec(),
					}));

					// batch mint: six
					run_to_block(System::block_number() + 1 + mint_cooldown);
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_ok!(AAvatars::mint(
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
							assert_eq!(AAvatars::free_mints(ALICE), initial_free_mints);
						},
					}
					expected_nonce += expected_nonce_increment * 6;
					owned_avatar_count += 6;
					assert_eq!(System::account_nonce(ALICE), expected_nonce);
					assert_eq!(AAvatars::owners(ALICE).len(), owned_avatar_count);
					assert!(AAvatars::is_season_active());
					System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarsMinted {
						avatar_ids: AAvatars::owners(ALICE)[4..=9].to_vec(),
					}));

					// check for season ending
					run_to_block(season.end + 1);
					assert_err!(
						AAvatars::mint(
							Origin::signed(ALICE),
							MintOption { count: MintPackSize::One, mint_type }
						),
						Error::<Test>::OutOfSeason
					);

					// check for minted avatars
					let minted = AAvatars::owners(ALICE)
						.into_iter()
						.map(|avatar_id| AAvatars::avatars(avatar_id).unwrap())
						.collect::<Vec<_>>();
					assert!(minted.iter().all(|(owner, _)| owner == &ALICE));
					assert!(minted.iter().all(|(_, avatar)| avatar.season_id == 1));
					assert!(minted
						.iter()
						.all(|(_, avatar)| avatar.souls >= 1 && avatar.souls <= 100));

					// reset for next iteration
					System::set_block_number(0);
					LastMintedBlockNumbers::<Test>::remove(ALICE);
					Owners::<Test>::remove(ALICE);
					IsSeasonActive::<Test>::set(false);
					CurrentSeasonId::<Test>::set(1);
					owned_avatar_count = 0;
				}
			});
	}

	#[test]
	fn mint_should_reject_when_minting_is_closed() {
		ExtBuilder::default().mint_open(false).build().execute_with(|| {
			for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
				for mint_type in [MintType::Normal, MintType::Free] {
					assert_noop!(
						AAvatars::mint(Origin::signed(ALICE), MintOption { count, mint_type }),
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
						AAvatars::mint(Origin::none(), MintOption { count, mint_type }),
						DispatchError::BadOrigin
					);
				}
			}
		});
	}

	#[test]
	fn mint_should_reject_when_season_is_inactive() {
		ExtBuilder::default()
			.mint_open(true)
			.balances(vec![(ALICE, 1_234_567_890_123_456)])
			.free_mints(vec![(ALICE, 10)])
			.build()
			.execute_with(|| {
				for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AAvatars::mint(Origin::signed(ALICE), MintOption { count, mint_type }),
							Error::<Test>::OutOfSeason
						);
					}
				}
			});
	}

	#[test]
	fn mint_should_reject_when_max_ownership_has_reached() {
		let max_avatars_per_player = 7;
		let avatar_ids = BoundedAvatarIdsOf::<Test>::try_from(
			(0..max_avatars_per_player)
				.map(|_| sp_core::H256::default())
				.collect::<Vec<_>>(),
		)
		.unwrap();
		assert_eq!(avatar_ids.len(), max_avatars_per_player);

		ExtBuilder::default()
			.seasons(vec![(SeasonId::default(), Season::default())])
			.max_avatars_per_player(max_avatars_per_player as u32)
			.mint_open(true)
			.balances(vec![(ALICE, 1_234_567_890_123_456)])
			.free_mints(vec![(ALICE, 10)])
			.build()
			.execute_with(|| {
				run_to_block(2);
				Owners::<Test>::insert(ALICE, avatar_ids);
				for count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for mint_type in [MintType::Normal, MintType::Free] {
						assert_noop!(
							AAvatars::mint(Origin::signed(ALICE), MintOption { count, mint_type }),
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
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.mint_cooldown(mint_cooldown)
			.balances(vec![(ALICE, 1_234_567_890_123_456)])
			.free_mints(vec![(ALICE, 10)])
			.build()
			.execute_with(|| {
				for mint_type in [MintType::Normal, MintType::Free] {
					run_to_block(season.start + 1);
					assert_ok!(AAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: mint_type.clone() }
					));

					for _ in 1..mint_cooldown {
						run_to_block(System::block_number() + 1);
						assert_noop!(
							AAvatars::mint(
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
					assert_eq!(System::block_number(), (season.start + 1) + mint_cooldown);
					assert_ok!(AAvatars::mint(
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
	fn mint_should_reject_when_balance_is_insufficient() {
		ExtBuilder::default()
			.mint_open(true)
			.seasons(vec![(1, Season::default())])
			.build()
			.execute_with(|| {
				for mint_count in [MintPackSize::One, MintPackSize::Three, MintPackSize::Six] {
					for (mint_type, error) in [
						(MintType::Normal, Error::<Test>::InsufficientFunds),
						(MintType::Free, Error::<Test>::InsufficientFreeMints),
					] {
						assert_err!(
							AAvatars::mint(
								Origin::signed(ALICE),
								MintOption { count: mint_count, mint_type }
							),
							error,
						);
					}
				}
			});
	}

	// TODO: refine free mint
	#[ignore]
	#[test]
	fn free_mint_is_still_possible_outside_season() {
		let season = Season::default().start(1).end(20);
		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.free_mints(vec![(ALICE, 100)])
			.build()
			.execute_with(|| {
				run_to_block(season.end + 1);
				assert!(!AAvatars::is_season_active());
				assert_ok!(AAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::One, mint_type: MintType::Free }
				));
				System::assert_has_event(mock::Event::AAvatars(crate::Event::AvatarsMinted {
					avatar_ids: vec![AAvatars::owners(ALICE)[0]],
				}));
			});
	}

	#[test]
	fn transfer_free_mints_should_work() {
		ExtBuilder::default()
			.free_mints(vec![(ALICE, 17), (BOB, 4)])
			.build()
			.execute_with(|| {
				assert_ok!(AAvatars::transfer_free_mints(Origin::signed(ALICE), BOB, 10));
				System::assert_last_event(mock::Event::AAvatars(
					crate::Event::FreeMintsTransferred { from: ALICE, to: BOB, how_many: 10 },
				));
				assert_eq!(AAvatars::free_mints(ALICE), 6);
				assert_eq!(AAvatars::free_mints(BOB), 14);

				assert_ok!(AAvatars::transfer_free_mints(Origin::signed(ALICE), CHARLIE, 2));
				System::assert_last_event(mock::Event::AAvatars(
					crate::Event::FreeMintsTransferred { from: ALICE, to: CHARLIE, how_many: 2 },
				));

				assert_eq!(AAvatars::free_mints(ALICE), 3);
				assert_eq!(AAvatars::free_mints(CHARLIE), 2);
			});
	}

	#[test]
	fn transfer_free_mints_should_reject_when_balance_is_insufficient() {
		ExtBuilder::default().free_mints(vec![(ALICE, 7)]).build().execute_with(|| {
			assert_noop!(
				AAvatars::transfer_free_mints(Origin::signed(ALICE), BOB, 10),
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
				assert_eq!(AAvatars::free_mints(BOB), 0);

				assert_ok!(AAvatars::issue_free_mints(Origin::signed(ALICE), BOB, 7));
				System::assert_last_event(mock::Event::AAvatars(crate::Event::FreeMintsIssued {
					to: BOB,
					how_many: 7,
				}));

				assert_eq!(AAvatars::free_mints(BOB), 7);

				assert_ok!(AAvatars::issue_free_mints(Origin::signed(ALICE), BOB, 3));
				System::assert_last_event(mock::Event::AAvatars(crate::Event::FreeMintsIssued {
					to: BOB,
					how_many: 3,
				}));

				assert_eq!(AAvatars::free_mints(BOB), 10);
			});
	}
}

mod forging {
	use super::*;
	use sp_runtime::testing::H256;
	use sp_std::collections::btree_set::BTreeSet;

	#[test]
	fn forge_should_work_for_matches() {
		let max_components = 5;
		let max_variations = 3;
		let tiers = vec![RarityTier::Common, RarityTier::Legendary];
		let season = Season::default()
			.end(99)
			.tiers(tiers.clone())
			.p_batch_mint(vec![100])
			.max_components(max_components)
			.max_variations(max_variations);
		let min_sacrifices = 1;
		let max_sacrifices = 2;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.mint_cooldown(1)
			.free_mints(vec![(BOB, 10)])
			.forge_open(true)
			.forge_min_sacrifices(min_sacrifices)
			.forge_max_sacrifices(max_sacrifices)
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
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
					Origin::signed(BOB),
					leader_id,
					sacrifice_ids.to_vec().try_into().unwrap()
				));
				let forged_leader = AAvatars::avatars(leader_id).unwrap().1;

				// check the result of the compare method
				for (sacrifice, result) in original_sacrifices
					.iter()
					.zip([(true, BTreeSet::from([1, 3])), (true, BTreeSet::from([0, 2, 3]))])
				{
					assert_eq!(
						original_leader.compare(
							sacrifice,
							max_variations,
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
				System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarForged {
					avatar_id: leader_id,
					upgraded_components: 2,
				}));

				// variations remain the same
				assert_eq!(
					original_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
					forged_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
				);
				// other components remain the same
				assert_eq!(
					original_leader.dna[2..max_components as usize],
					forged_leader.dna[2..max_components as usize]
				);
			});
	}

	#[test]
	fn forge_should_work_for_non_matches() {
		let max_components = 10;
		let max_variations = 12;
		let tiers =
			vec![RarityTier::Common, RarityTier::Uncommon, RarityTier::Rare, RarityTier::Legendary];
		let season = Season::default()
			.end(99)
			.tiers(tiers.clone())
			.p_batch_mint(vec![33, 33, 34])
			.max_components(max_components)
			.max_variations(max_variations);
		let min_sacrifices = 1;
		let max_sacrifices = 5;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.mint_cooldown(1)
			.free_mints(vec![(BOB, 10)])
			.forge_open(true)
			.forge_min_sacrifices(min_sacrifices)
			.forge_max_sacrifices(max_sacrifices)
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Free }
				));

				// forge
				let owned_avatar_ids = AAvatars::owners(BOB);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_id = owned_avatar_ids[1];

				let original_leader = AAvatars::avatars(leader_id).unwrap().1;
				let original_sacrifice = AAvatars::avatars(sacrifice_id).unwrap().1;

				assert_ok!(AAvatars::forge(
					Origin::signed(BOB),
					leader_id,
					[sacrifice_id].to_vec().try_into().unwrap()
				));
				let forged_leader = AAvatars::avatars(leader_id).unwrap().1;

				// check the result of the compare method
				assert_eq!(
					original_leader.compare(
						&original_sacrifice,
						max_variations,
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
				System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarForged {
					avatar_id: leader_id,
					upgraded_components: 0,
				}));
			});
	}

	#[test]
	fn forge_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::forge(
					Origin::none(),
					H256::default(),
					BoundedAvatarIdsOf::<Test>::default(),
				),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn forge_should_reject_out_of_bound_sacrifices() {
		let min_sacrifices = 3;
		let max_sacrifices = 5;

		ExtBuilder::default()
			.forge_open(true)
			.forge_min_sacrifices(min_sacrifices)
			.forge_max_sacrifices(max_sacrifices)
			.build()
			.execute_with(|| {
				for i in 0..min_sacrifices {
					assert_noop!(
						AAvatars::forge(
							Origin::signed(ALICE),
							H256::default(),
							BoundedAvatarIdsOf::<Test>::try_from(
								(0..i).map(|_| H256::default()).collect::<Vec<_>>()
							)
							.unwrap(),
						),
						Error::<Test>::TooFewSacrifices,
					);
				}

				for i in (max_sacrifices + 1)..(max_sacrifices + 5) {
					assert_noop!(
						AAvatars::forge(
							Origin::signed(ALICE),
							H256::default(),
							BoundedAvatarIdsOf::<Test>::try_from(
								(0..i).map(|_| H256::default()).collect::<Vec<_>>()
							)
							.unwrap(),
						),
						Error::<Test>::TooManySacrifices,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_out_of_season_calls() {
		ExtBuilder::default()
			.forge_open(true)
			.forge_min_sacrifices(1)
			.forge_max_sacrifices(1)
			.build()
			.execute_with(|| {
				assert_noop!(
					AAvatars::forge(
						Origin::signed(ALICE),
						H256::default(),
						BoundedAvatarIdsOf::<Test>::try_from(vec![H256::default()]).unwrap(),
					),
					Error::<Test>::OutOfSeason,
				);
			});
	}

	#[test]
	fn forge_should_reject_unknown_season_calls() {
		ExtBuilder::default()
			.forge_open(true)
			.forge_min_sacrifices(1)
			.forge_max_sacrifices(1)
			.build()
			.execute_with(|| {
				CurrentSeasonId::<Test>::put(123);
				IsSeasonActive::<Test>::put(true);
				assert_noop!(
					AAvatars::forge(
						Origin::signed(ALICE),
						H256::default(),
						BoundedAvatarIdsOf::<Test>::try_from(vec![H256::default()]).unwrap(),
					),
					Error::<Test>::UnknownSeason,
				);
			});
	}

	#[test]
	fn forge_should_reject_unknown_avatars() {
		let season = Season::default().end(99);
		let min_sacrifices = 1;
		let max_sacrifices = 3;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.mint_cooldown(0)
			.free_mints(vec![(ALICE, 10)])
			.forge_open(true)
			.forge_min_sacrifices(min_sacrifices)
			.forge_max_sacrifices(max_sacrifices)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				for _ in 0..max_sacrifices {
					assert_ok!(AAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Free }
					));
					run_to_block(System::block_number() + 1);
				}

				let owned_avatars = AAvatars::owners(ALICE);
				for (leader, sacrifices) in [
					(H256::default(), vec![owned_avatars[0], owned_avatars[2]].try_into().unwrap()),
					(owned_avatars[1], vec![H256::default(), H256::default()].try_into().unwrap()),
					(owned_avatars[1], vec![H256::default(), owned_avatars[2]].try_into().unwrap()),
				] {
					assert_noop!(
						AAvatars::forge(Origin::signed(ALICE), leader, sacrifices),
						Error::<Test>::UnknownAvatar,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_incorrect_ownership() {
		let season = Season::default().end(99);
		let min_sacrifices = 1;
		let max_sacrifices = 3;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.mint_cooldown(0)
			.free_mints(vec![(ALICE, 10), (BOB, 10)])
			.forge_open(true)
			.forge_min_sacrifices(min_sacrifices)
			.forge_max_sacrifices(max_sacrifices)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				for player in [ALICE, BOB] {
					for _ in 0..max_sacrifices {
						assert_ok!(AAvatars::mint(
							Origin::signed(player),
							MintOption { count: MintPackSize::One, mint_type: MintType::Free }
						));
						run_to_block(System::block_number() + 1);
					}
				}

				for (player, leader, sacrifices) in [
					(
						ALICE,
						AAvatars::owners(ALICE)[0],
						AAvatars::owners(BOB)[0..2].to_vec().try_into().unwrap(),
					),
					(
						ALICE,
						AAvatars::owners(BOB)[0],
						AAvatars::owners(ALICE)[0..2].to_vec().try_into().unwrap(),
					),
					(
						ALICE,
						AAvatars::owners(BOB)[0],
						AAvatars::owners(BOB)[0..2].to_vec().try_into().unwrap(),
					),
				] {
					assert_noop!(
						AAvatars::forge(Origin::signed(player), leader, sacrifices),
						Error::<Test>::Ownership,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_leader_in_sacrifice() {
		let season = Season::default().end(99);
		let min_sacrifices = 1;
		let max_sacrifices = 3;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.mint_open(true)
			.mint_cooldown(0)
			.free_mints(vec![(ALICE, 10)])
			.forge_open(true)
			.forge_min_sacrifices(min_sacrifices)
			.forge_max_sacrifices(max_sacrifices)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				for _ in 0..max_sacrifices {
					assert_ok!(AAvatars::mint(
						Origin::signed(ALICE),
						MintOption { count: MintPackSize::One, mint_type: MintType::Free }
					));
					run_to_block(System::block_number() + 1);
				}

				for (player, leader, sacrifices) in [
					(
						ALICE,
						AAvatars::owners(ALICE)[0],
						AAvatars::owners(ALICE)[0..2].to_vec().try_into().unwrap(),
					),
					(
						ALICE,
						AAvatars::owners(ALICE)[1],
						AAvatars::owners(ALICE)[0..2].to_vec().try_into().unwrap(),
					),
				] {
					assert_noop!(
						AAvatars::forge(Origin::signed(player), leader, sacrifices),
						Error::<Test>::LeaderSacrificed,
					);
				}
			});
	}

	#[test]
	fn forge_should_reject_avatars_in_trade() {
		let season = Season::default();
		let price = 321;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(ALICE, 6), (BOB, 6)])
			.mint_open(true)
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.forge_open(true)
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(ALICE),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::Six, mint_type: MintType::Normal }
				));

				let leader = AAvatars::owners(ALICE)[0];
				let sacrifices: BoundedAvatarIdsOf<Test> =
					AAvatars::owners(ALICE)[1..3].to_vec().try_into().unwrap();

				assert_ok!(AAvatars::set_price(Origin::signed(ALICE), leader, price));
				assert_noop!(
					AAvatars::forge(Origin::signed(ALICE), leader, sacrifices.clone()),
					Error::<Test>::AvatarInTrade
				);

				assert_ok!(AAvatars::set_price(Origin::signed(ALICE), sacrifices[1], price));
				assert_noop!(
					AAvatars::forge(Origin::signed(ALICE), leader, sacrifices.clone()),
					Error::<Test>::AvatarInTrade
				);

				assert_ok!(AAvatars::remove_price(Origin::signed(ALICE), leader));
				assert_noop!(
					AAvatars::forge(Origin::signed(ALICE), leader, sacrifices),
					Error::<Test>::AvatarInTrade
				);
			});
	}
}

mod trading {
	use super::*;

	#[test]
	fn set_price_should_work() {
		let season = Season::default();

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(BOB, 999)])
			.mint_open(true)
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				let avatar_for_sale = AAvatars::owners(BOB)[0];
				let price = 7357;

				assert_eq!(AAvatars::trade(avatar_for_sale), None);
				assert_ok!(AAvatars::set_price(Origin::signed(BOB), avatar_for_sale, price));
				assert_eq!(AAvatars::trade(avatar_for_sale), Some(price));
				System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarPriceSet {
					avatar_id: avatar_for_sale,
					price,
				}));
			});
	}

	#[test]
	fn set_price_should_reject_when_trading_is_closed() {
		ExtBuilder::default().trade_open(false).build().execute_with(|| {
			assert_noop!(
				AAvatars::set_price(Origin::signed(ALICE), sp_core::H256::default(), 1),
				Error::<Test>::TradeClosed,
			);
		});
	}

	#[test]
	fn set_price_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::set_price(Origin::none(), sp_core::H256::default(), 1),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn set_price_should_reject_incorrect_ownership() {
		let season = Season::default();

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(BOB, 999)])
			.mint_open(true)
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				assert_noop!(
					AAvatars::set_price(Origin::signed(CHARLIE), AAvatars::owners(BOB)[0], 101),
					Error::<Test>::Ownership
				);
			});
	}

	#[test]
	fn remove_price_should_work() {
		let season = Season::default();

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(BOB, 999)])
			.mint_open(true)
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				let avatar_for_sale = AAvatars::owners(BOB)[0];
				let price = 101;
				assert_ok!(AAvatars::set_price(Origin::signed(BOB), avatar_for_sale, price));

				assert_eq!(AAvatars::trade(avatar_for_sale), Some(101));
				assert_ok!(AAvatars::remove_price(Origin::signed(BOB), avatar_for_sale));
				assert_eq!(AAvatars::trade(avatar_for_sale), None);
				System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarPriceUnset {
					avatar_id: avatar_for_sale,
				}));
			});
	}

	#[test]
	fn remove_price_should_reject_when_trading_is_closed() {
		ExtBuilder::default().trade_open(false).build().execute_with(|| {
			assert_noop!(
				AAvatars::remove_price(Origin::signed(ALICE), sp_core::H256::default()),
				Error::<Test>::TradeClosed,
			);
		});
	}

	#[test]
	fn remove_price_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::remove_price(Origin::none(), sp_core::H256::default()),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn remove_price_should_reject_incorrect_ownership() {
		let season = Season::default();

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(BOB, 999)])
			.mint_open(true)
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				let avatar_for_sale = AAvatars::owners(BOB)[0];
				assert_ok!(AAvatars::set_price(Origin::signed(BOB), avatar_for_sale, 123));

				assert_noop!(
					AAvatars::remove_price(Origin::signed(CHARLIE), avatar_for_sale),
					Error::<Test>::Ownership
				);
			});
	}

	#[test]
	fn remove_price_should_reject_unlisted_avatar() {
		ExtBuilder::default().trade_open(true).build().execute_with(|| {
			assert_noop!(
				AAvatars::remove_price(Origin::signed(CHARLIE), sp_core::H256::default()),
				Error::<Test>::UnknownAvatarForSale,
			);
		});
	}

	#[test]
	fn buy_should_work() {
		let season = Season::default();
		let mint_fees = MintFees { one: 1, three: 3, six: 6 };
		let price = 310_984;
		let alice_initial_bal = price + 20_849;
		let bob_initial_bal = 103_598;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(ALICE, alice_initial_bal), (BOB, bob_initial_bal)])
			.mint_open(true)
			.mint_fees(mint_fees)
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				let owned_by_alice = AAvatars::owners(ALICE);
				let owned_by_bob = AAvatars::owners(BOB);

				let avatar_for_sale = AAvatars::owners(BOB)[0];
				assert_ok!(AAvatars::set_price(Origin::signed(BOB), avatar_for_sale, price));
				assert_ok!(AAvatars::buy(Origin::signed(ALICE), avatar_for_sale));

				// check for balance transfer
				assert_eq!(Balances::free_balance(ALICE), alice_initial_bal - price);
				assert_eq!(Balances::free_balance(BOB), bob_initial_bal + price - mint_fees.one);

				// check for ownership transfer
				assert_eq!(AAvatars::owners(ALICE).len(), owned_by_alice.len() + 1);
				assert_eq!(AAvatars::owners(BOB).len(), owned_by_bob.len() - 1);
				assert!(AAvatars::owners(ALICE).contains(&avatar_for_sale));
				assert!(!AAvatars::owners(BOB).contains(&avatar_for_sale));
				assert_eq!(AAvatars::avatars(avatar_for_sale).unwrap().0, ALICE);

				// check for removal from trade storage
				assert_eq!(AAvatars::trade(avatar_for_sale), None);

				// check events
				System::assert_last_event(mock::Event::AAvatars(crate::Event::AvatarTraded {
					avatar_id: avatar_for_sale,
					from: BOB,
					to: ALICE,
				}));
			});
	}

	#[test]
	fn buy_should_reject_when_trading_is_closed() {
		ExtBuilder::default().trade_open(false).build().execute_with(|| {
			assert_noop!(
				AAvatars::buy(Origin::signed(ALICE), sp_core::H256::default()),
				Error::<Test>::TradeClosed,
			);
		});
	}

	#[test]
	fn buy_should_reject_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::buy(Origin::none(), sp_core::H256::default()),
				DispatchError::BadOrigin,
			);
		});
	}

	#[test]
	fn buy_should_reject_unlisted_avatar() {
		ExtBuilder::default().trade_open(true).build().execute_with(|| {
			assert_noop!(
				AAvatars::buy(Origin::signed(BOB), sp_core::H256::default()),
				Error::<Test>::UnknownAvatarForSale,
			);
		});
	}

	#[test]
	fn buy_should_reject_insufficient_balance() {
		let season = Season::default();
		let price = 310_984;

		ExtBuilder::default()
			.seasons(vec![(1, season.clone())])
			.balances(vec![(ALICE, price - 1), (BOB, 999)])
			.mint_open(true)
			.mint_fees(MintFees { one: 1, three: 1, six: 1 })
			.trade_open(true)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);
				assert_ok!(AAvatars::mint(
					Origin::signed(BOB),
					MintOption { count: MintPackSize::One, mint_type: MintType::Normal }
				));

				let avatar_for_sale = AAvatars::owners(BOB)[0];
				assert_ok!(AAvatars::set_price(Origin::signed(BOB), avatar_for_sale, price));
				assert_noop!(
					AAvatars::buy(Origin::signed(ALICE), avatar_for_sale),
					Error::<Test>::InsufficientFunds
				);
			});
	}
}
