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

	const SEASON_ID: SeasonId = 1;

	#[test]
	fn new_season_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(BOB), Season::default(),),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn new_season_should_work() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let first_season = Season::default().early_start(1).start(5).end(10);
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));
			assert_eq!(AwesomeAvatars::seasons(1), Some(first_season.clone()));
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::NewSeasonCreated(
				first_season,
			)));

			let second_season = Season::default().early_start(11).start(12).end(13);
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), second_season.clone()));
			assert_eq!(AwesomeAvatars::seasons(2), Some(second_season.clone()));
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::NewSeasonCreated(
				second_season,
			)));
		});
	}

	#[test]
	fn new_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![Season::default().early_start(1).start(5).end(10)])
			.build()
			.execute_with(|| {
				let second_season = Season::default().early_start(3).start(7).end(10);
				assert!(second_season.early_start < second_season.start);
				assert_noop!(
					AwesomeAvatars::new_season(Origin::signed(ALICE), second_season),
					Error::<Test>::EarlyStartTooEarly
				);
			});
	}

	#[test]
	fn new_season_should_return_error_when_early_start_is_later_than_start() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let new_season = Season::default().early_start(6).start(3).end(10);
			assert!(new_season.early_start > new_season.start);
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), new_season,),
				Error::<Test>::EarlyStartTooLate
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_start_is_later_than_end() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let new_season = Season::default().early_start(11).start(12).end(10);
			assert!(new_season.early_start < new_season.start);
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_rarity_tier_is_duplicated() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			for duplicated_rarity_tiers in [
				test_rarity_tiers(vec![(RarityTier::Common, 1), (RarityTier::Common, 99)]),
				test_rarity_tiers(vec![
					(RarityTier::Common, 10),
					(RarityTier::Common, 10),
					(RarityTier::Legendary, 80),
				]),
			] {
				assert_noop!(
					AwesomeAvatars::new_season(
						Origin::signed(ALICE),
						Season::default().rarity_tiers(duplicated_rarity_tiers)
					),
					Error::<Test>::DuplicatedRarityTier
				);
			}
		});
	}

	#[test]
	fn new_season_should_return_error_when_sum_of_rarity_chance_is_incorrect() {
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
				assert_noop!(
					AwesomeAvatars::new_season(
						Origin::signed(ALICE),
						Season::default().rarity_tiers(incorrect_rarity_tiers)
					),
					Error::<Test>::IncorrectRarityPercentages
				);
			}
		});
	}

	#[test]
	fn update_season_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(BOB), 7357, Season::default()),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn update_season_should_work() {
		let first_season = Season::default().early_start(1).start(5).end(10);
		let second_season = Season::default().early_start(11).start(11).end(20);

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season, second_season.clone()])
			.build()
			.execute_with(|| {
				let first_season_update = Season::default().early_start(1).start(5).end(8);
				assert!(first_season_update.end < second_season.early_start);
				assert_ok!(AwesomeAvatars::update_season(
					Origin::signed(ALICE),
					1,
					first_season_update.clone()
				));
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::SeasonUpdated(first_season_update, 1),
				));
			});
	}

	#[test]
	fn update_season_should_return_error_when_season_not_found() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(
					Origin::signed(ALICE),
					10,
					Season::default().early_start(1).start(12).end(30)
				),
				Error::<Test>::UnknownSeason
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_season_to_update_ends_after_next_season_start() {
		let first_season = Season::default().early_start(1).start(5).end(10);
		let second_season = Season::default().early_start(11).start(15).end(20);

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season, second_season.clone()])
			.build()
			.execute_with(|| {
				let first_season_update = Season::default().early_start(1).start(5).end(14);
				assert!(first_season_update.end > second_season.early_start);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 1, first_season_update),
					Error::<Test>::SeasonEndTooLate
				);
			});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		let first_season = Season::default().early_start(1).start(5).end(10);
		let second_season = Season::default().early_start(11).start(15).end(20);

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season.clone(), second_season])
			.build()
			.execute_with(|| {
				let second_season_update = Season::default().early_start(8).start(15).end(20);
				assert!(second_season_update.early_start < first_season.end);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
					Error::<Test>::EarlyStartTooEarly
				);

				let second_season_update = Season::default().early_start(9).start(15).end(20);
				assert!(second_season_update.early_start < first_season.end);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
					Error::<Test>::EarlyStartTooEarly
				);

				let second_season_update = Season::default().early_start(10).start(15).end(20);
				assert!(second_season_update.early_start == first_season.end);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
					Error::<Test>::EarlyStartTooEarly
				);
			});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_is_earlier_than_or_equal_to_start() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let season_update = Season::default().early_start(5).start(1).end(10);
			assert!(season_update.early_start > season_update.start);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 111, season_update),
				Error::<Test>::EarlyStartTooLate
			);

			let season_update = Season::default().early_start(5).start(5).end(10);
			assert!(season_update.early_start == season_update.start);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 222, season_update),
				Error::<Test>::EarlyStartTooLate
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_start_is_later_than_end() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let season_update = Season::default().early_start(1).start(15).end(10);
			assert!(season_update.start > season_update.end);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 123, season_update),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn update_season_should_handle_underflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(
					Origin::signed(ALICE),
					SeasonId::MIN,
					Season::default()
				),
				ArithmeticError::Underflow
			);
		});
	}

	#[test]
	fn update_season_should_handle_overflow() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(
					Origin::signed(ALICE),
					SeasonId::MAX,
					Season::default()
				),
				ArithmeticError::Overflow
			);
		});
	}

	#[test]
	fn update_season_metadata_should_work() {
		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![Season::default()])
			.build()
			.execute_with(|| {
				let metadata = SeasonMetadata::default();

				assert_ok!(AwesomeAvatars::update_season_metadata(
					Origin::signed(ALICE),
					SEASON_ID,
					metadata.clone()
				));

				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::UpdatedSeasonMetadata {
						season_id: SEASON_ID,
						season_metadata: metadata.clone(),
					},
				));

				assert_eq!(AwesomeAvatars::seasons_metadata(SEASON_ID), Some(metadata));
			});
	}

	#[test]
	fn update_season_metadata_should_fail_if_caller_is_not_organizer() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season_metadata(
					Origin::signed(BOB),
					SEASON_ID,
					SeasonMetadata::default()
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn update_season_metadata_should_fail_with_invalid_season_id() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season_metadata(
					Origin::signed(ALICE),
					SEASON_ID + 10,
					SeasonMetadata::default()
				),
				Error::<Test>::UnknownSeason
			);
		});
	}

	#[test]
	fn active_season_hooks_should_work() {
		let season_1 = Season::default().early_start(1).start(5).end(10);
		let season_2 = Season::default().early_start(11).start(15).end(20);
		let season_3 = Season::default().early_start(30).start(31).end(32);

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![season_1.clone(), season_2.clone(), season_3.clone()])
			.build()
			.execute_with(|| {
				for _ in 0..season_1.early_start {
					assert_eq!(AwesomeAvatars::active_season_id(), None);
					assert_eq!(AwesomeAvatars::next_active_season_id(), 1);
				}

				for block_number in season_1.early_start..season_1.end {
					run_to_block(block_number + 1);
					assert_eq!(AwesomeAvatars::active_season_id(), Some(1));
					assert_eq!(AwesomeAvatars::next_active_season_id(), 2);
				}

				for block_number in season_2.early_start..season_2.end {
					run_to_block(block_number + 1);
					assert_eq!(AwesomeAvatars::active_season_id(), Some(2));
					assert_eq!(AwesomeAvatars::next_active_season_id(), 3);
				}

				for block_number in season_2.end..(season_3.early_start - 1) {
					run_to_block(block_number + 1);
					assert_eq!(AwesomeAvatars::active_season_id(), None);
					assert_eq!(AwesomeAvatars::next_active_season_id(), 3);
				}

				for block_number in season_3.early_start..season_3.end {
					run_to_block(block_number + 1);
					assert_eq!(AwesomeAvatars::active_season_id(), Some(3));
					assert_eq!(AwesomeAvatars::next_active_season_id(), 4);
				}
			});
	}

	#[test]
	fn active_season_hooks_should_do_nothing_if_no_season_exists() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert!(AwesomeAvatars::active_season_id().is_none());
			assert_eq!(AwesomeAvatars::next_active_season_id(), 1);

			run_to_block(2);
			assert!(AwesomeAvatars::active_season_id().is_none());
			assert_eq!(AwesomeAvatars::next_active_season_id(), 1);

			run_to_block(15);
			assert!(AwesomeAvatars::active_season_id().is_none());
			assert_eq!(AwesomeAvatars::next_active_season_id(), 1);
		});
	}
}

mod config {
	use super::*;

	#[test]
	fn update_mint_available_should_work() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert!(!AwesomeAvatars::mint_available());
			assert_ok!(AwesomeAvatars::update_mint_available(Origin::signed(ALICE), true));
			assert!(AwesomeAvatars::mint_available());
			System::assert_last_event(mock::Event::AwesomeAvatars(
				crate::Event::UpdatedMintAvailability { availability: true },
			));
		});
	}

	#[test]
	fn update_mint_available_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let original_mint_available = AwesomeAvatars::mint_available();
			assert_noop!(
				AwesomeAvatars::update_mint_available(Origin::signed(BOB), true),
				DispatchError::BadOrigin
			);
			assert_eq!(AwesomeAvatars::mint_available(), original_mint_available);
		});
	}

	#[test]
	fn update_mint_fee_should_work() {
		let original_fee = 550_000_000_000;
		let update_fee = 650_000_000_000;

		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_eq!(AwesomeAvatars::mint_fee(), original_fee);
			assert_ok!(AwesomeAvatars::update_mint_fee(Origin::signed(ALICE), update_fee));
			assert_eq!(AwesomeAvatars::mint_fee(), update_fee);
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::UpdatedMintFee {
				fee: update_fee,
			}));
		});
	}

	#[test]
	fn update_mint_fee_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let original_mint_fee = AwesomeAvatars::mint_fee();
			assert_noop!(
				AwesomeAvatars::update_mint_fee(Origin::signed(BOB), 123),
				DispatchError::BadOrigin
			);
			assert_eq!(AwesomeAvatars::mint_fee(), original_mint_fee);
		});
	}

	#[test]
	fn update_mint_cooldown_should_update_work() {
		let original_cd = 5;
		let update_cd = 10;

		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_eq!(AwesomeAvatars::mint_cooldown(), original_cd);
			assert_ok!(AwesomeAvatars::update_mint_cooldown(Origin::signed(ALICE), update_cd));
			assert_eq!(AwesomeAvatars::mint_cooldown(), update_cd);
			System::assert_last_event(mock::Event::AwesomeAvatars(
				crate::Event::UpdatedMintCooldown { cooldown: update_cd },
			));
		});
	}

	#[test]
	fn update_mint_cooldown_should_fail_for_not_organizer_account() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let original_cd = AwesomeAvatars::mint_cooldown();
			assert_noop!(
				AwesomeAvatars::update_mint_cooldown(Origin::signed(BOB), 120_934),
				DispatchError::BadOrigin
			);
			assert_eq!(AwesomeAvatars::mint_cooldown(), original_cd);
		});
	}
}

mod minting {
	use super::*;

	#[test]
	fn mint_should_work() {
		let max_components = 7;
		let season = Season::default().end(20).max_components(max_components);

		let expected_nonce_increment = 2 * max_components as MockIndex;
		let mut expected_nonce = 0;

		let mint_cooldown = 5;

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![season.clone()])
			.mint_availability(true)
			.mint_cooldown(mint_cooldown)
			.build()
			.execute_with(|| {
				run_to_block(season.early_start + 1);

				assert_eq!(System::account_nonce(ALICE), expected_nonce);
				assert_ok!(AwesomeAvatars::mint(Origin::signed(ALICE)));
				expected_nonce += expected_nonce_increment;
				assert_eq!(System::account_nonce(ALICE), expected_nonce);
				assert_eq!(AwesomeAvatars::owners(ALICE).len(), 1);
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::AvatarMinted { avatar_id: AwesomeAvatars::owners(ALICE)[0] },
				));

				assert_eq!(System::account_nonce(ALICE), expected_nonce);
				run_to_block(System::block_number() + 1 + mint_cooldown);
				assert_ok!(AwesomeAvatars::mint(Origin::signed(ALICE)));
				expected_nonce += expected_nonce_increment;
				assert_eq!(AwesomeAvatars::owners(ALICE).len(), 2);
				assert_eq!(System::account_nonce(ALICE), expected_nonce);
				System::assert_last_event(mock::Event::AwesomeAvatars(
					crate::Event::AvatarMinted { avatar_id: AwesomeAvatars::owners(ALICE)[1] },
				));

				let avatar_ids = AwesomeAvatars::owners(ALICE);
				let (player_0, avatar_0) = AwesomeAvatars::avatars(avatar_ids[0]).unwrap();
				let (player_1, avatar_1) = AwesomeAvatars::avatars(avatar_ids[1]).unwrap();

				assert_eq!(player_0, player_1);
				assert_eq!(player_0, ALICE);
				assert_eq!(player_1, ALICE);

				assert_eq!(avatar_0.season, avatar_1.season);
				assert_eq!(avatar_0.season, AwesomeAvatars::active_season_id().unwrap());
				assert_eq!(avatar_1.season, AwesomeAvatars::active_season_id().unwrap());

				assert_ne!(avatar_0.dna, avatar_1.dna);
				assert_eq!(avatar_0.dna.len(), (2 * max_components as usize) / 2);
				assert_eq!(avatar_1.dna.len(), (2 * max_components as usize) / 2);
			});
	}

	#[test]
	fn mint_should_return_error_when_minting_is_unavailable() {
		ExtBuilder::default().mint_availability(false).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::mint(Origin::signed(ALICE)),
				Error::<Test>::MintUnavailable
			);
		});
	}

	#[test]
	fn mint_should_reject_unsigned_caller() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(AwesomeAvatars::mint(Origin::none()), DispatchError::BadOrigin);
		});
	}

	#[test]
	fn mint_should_return_error_when_season_is_inactive() {
		ExtBuilder::default()
			.organizer(ALICE)
			.mint_availability(true)
			.build()
			.execute_with(|| {
				assert_noop!(
					AwesomeAvatars::mint(Origin::signed(ALICE)),
					Error::<Test>::OutOfSeason
				);
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
			.seasons(vec![Season::default()])
			.mint_availability(true)
			.build()
			.execute_with(|| {
				run_to_block(2);
				Owners::<Test>::insert(ALICE, avatar_ids);
				assert_noop!(
					AwesomeAvatars::mint(Origin::signed(ALICE)),
					Error::<Test>::MaxOwnershipReached
				);
			});
	}

	#[test]
	fn mint_should_wait_for_cooldown() {
		let season = Season::default().early_start(1).start(3).end(20);
		let mint_cooldown = 7;

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![season.clone()])
			.mint_availability(true)
			.mint_cooldown(mint_cooldown)
			.build()
			.execute_with(|| {
				run_to_block(season.start + 1);
				assert_ok!(AwesomeAvatars::mint(Origin::signed(ALICE)));

				for _ in 0..mint_cooldown {
					run_to_block(System::block_number() + 1);
					assert_noop!(
						AwesomeAvatars::mint(Origin::signed(ALICE)),
						Error::<Test>::MintCooldown
					);
				}

				run_to_block(System::block_number() + 1);
				assert_eq!(System::block_number(), (season.start + 1) + (mint_cooldown + 1));
				assert_ok!(AwesomeAvatars::mint(Origin::signed(ALICE)));
			});
	}
}
