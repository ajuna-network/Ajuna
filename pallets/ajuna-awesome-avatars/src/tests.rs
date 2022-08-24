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

use crate::{mock::*, season::*, *};
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

	fn get_rarity_tiers() -> RarityTiers {
		let mut tiers = RarityTiers::new();

		tiers.try_insert(RarityTier::Common, 50).expect("Should insert element");
		tiers.try_insert(RarityTier::Uncommon, 30).expect("Should insert element");
		tiers.try_insert(RarityTier::Rare, 12).expect("Should insert element");
		tiers.try_insert(RarityTier::Epic, 5).expect("Should insert element");
		tiers.try_insert(RarityTier::Legendary, 2).expect("Should insert element");
		tiers.try_insert(RarityTier::Mythical, 1).expect("Should insert element");

		tiers
	}

	#[test]
	fn new_season_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::new_season(
					Origin::signed(BOB),
					Season {
						early_start: 1,
						start: 2,
						end: 3,
						max_mints: 4,
						max_mythical_mints: 5,
						rarity_tiers: get_rarity_tiers(),
						max_variations: 1
					}
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn new_season_should_work() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let first_season = Season {
				early_start: 1,
				start: 5,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));
			assert_eq!(AwesomeAvatars::seasons(1), Some(first_season.clone()));
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::NewSeasonCreated(
				first_season,
			)));

			let second_season = Season {
				early_start: 11,
				start: 12,
				end: 13,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), second_season.clone()));
			assert_eq!(AwesomeAvatars::seasons(2), Some(second_season.clone()));
			System::assert_last_event(mock::Event::AwesomeAvatars(crate::Event::NewSeasonCreated(
				second_season,
			)));
		});
	}

	#[test]
	fn new_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		let first_season = Season {
			early_start: 1,
			start: 5,
			end: 10,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season])
			.build()
			.execute_with(|| {
				let second_season = Season {
					early_start: 3,
					start: 7,
					end: 10,
					max_mints: 1,
					max_mythical_mints: 1,
					rarity_tiers: get_rarity_tiers(),
					max_variations: 1,
				};
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
			let new_season = Season {
				early_start: 6,
				start: 3,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
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
			let new_season = Season {
				early_start: 11,
				start: 12,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
			assert!(new_season.early_start < new_season.start);
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn new_season_should_error_when_rarity_tier_sum_is_incorrect() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			let mut incorrect_tiers = get_rarity_tiers();
			incorrect_tiers.try_insert(RarityTier::Epic, 100).expect("Should insert item");
			let new_season = Season {
				early_start: 1,
				start: 5,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: incorrect_tiers,
				max_variations: 1,
			};
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::IncorrectRarityChances
			);
		});
	}

	#[test]
	fn update_season_should_reject_non_organizer_as_caller() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(
					Origin::signed(BOB),
					7357,
					Season {
						early_start: 1,
						start: 2,
						end: 3,
						max_mints: 4,
						max_mythical_mints: 5,
						rarity_tiers: get_rarity_tiers(),
						max_variations: 1
					}
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn update_season_should_work() {
		let first_season = Season {
			early_start: 1,
			start: 5,
			end: 10,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};
		let second_season = Season {
			early_start: 11,
			start: 15,
			end: 20,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season, second_season.clone()])
			.build()
			.execute_with(|| {
				let first_season_update = Season {
					early_start: 1,
					start: 5,
					end: 8,
					max_mints: 1,
					max_mythical_mints: 1,
					rarity_tiers: get_rarity_tiers(),
					max_variations: 1,
				};
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
					Season {
						early_start: 1,
						start: 12,
						end: 30,
						max_mints: 1,
						max_mythical_mints: 1,
						rarity_tiers: get_rarity_tiers(),
						max_variations: 1,
					}
				),
				Error::<Test>::UnknownSeason
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_season_to_update_ends_after_next_season_start() {
		let first_season = Season {
			early_start: 1,
			start: 5,
			end: 10,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};
		let second_season = Season {
			early_start: 11,
			start: 15,
			end: 20,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season, second_season.clone()])
			.build()
			.execute_with(|| {
				let first_season_update = Season {
					early_start: 1,
					start: 5,
					end: 14,
					max_mints: 1,
					max_mythical_mints: 1,
					rarity_tiers: get_rarity_tiers(),
					max_variations: 1,
				};
				assert!(first_season_update.end > second_season.early_start);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 1, first_season_update),
					Error::<Test>::SeasonEndTooLate
				);
			});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		let first_season = Season {
			early_start: 1,
			start: 5,
			end: 10,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};
		let second_season = Season {
			early_start: 11,
			start: 15,
			end: 20,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season.clone(), second_season])
			.build()
			.execute_with(|| {
				let second_season_update = Season {
					early_start: 8,
					start: 15,
					end: 20,
					max_mints: 1,
					max_mythical_mints: 1,
					rarity_tiers: get_rarity_tiers(),
					max_variations: 1,
				};
				assert!(second_season_update.early_start < first_season.end);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
					Error::<Test>::EarlyStartTooEarly
				);

				let second_season_update = Season {
					early_start: 9,
					start: 15,
					end: 20,
					max_mints: 1,
					max_mythical_mints: 1,
					rarity_tiers: get_rarity_tiers(),
					max_variations: 1,
				};
				assert!(second_season_update.early_start < first_season.end);
				assert_noop!(
					AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
					Error::<Test>::EarlyStartTooEarly
				);

				let second_season_update = Season {
					early_start: 10,
					start: 15,
					end: 20,
					max_mints: 2,
					max_mythical_mints: 1,
					rarity_tiers: get_rarity_tiers(),
					max_variations: 1,
				};
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
			let season_update = Season {
				early_start: 5,
				start: 1,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
			assert!(season_update.early_start > season_update.start);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 111, season_update),
				Error::<Test>::EarlyStartTooLate
			);

			let season_update = Season {
				early_start: 5,
				start: 5,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
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
			let season_update = Season {
				early_start: 1,
				start: 15,
				end: 10,
				max_mints: 1,
				max_mythical_mints: 1,
				rarity_tiers: get_rarity_tiers(),
				max_variations: 1,
			};
			assert!(season_update.start > season_update.end);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 123, season_update),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn update_season_should_handle_underflow() {
		let season_update = Season {
			early_start: 1,
			start: 2,
			end: 3,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), SeasonId::MIN, season_update),
				ArithmeticError::Underflow
			);
		});
	}

	#[test]
	fn update_season_should_handle_overflow() {
		let season_update = Season {
			early_start: 1,
			start: 2,
			end: 3,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), SeasonId::MAX, season_update),
				ArithmeticError::Overflow
			);
		});
	}

	#[test]
	fn update_season_metadata_should_work() {
		let first_season = Season {
			early_start: 1,
			start: 5,
			end: 10,
			max_mints: 1,
			max_mythical_mints: 1,
			rarity_tiers: get_rarity_tiers(),
			max_variations: 1,
		};

		ExtBuilder::default()
			.organizer(ALICE)
			.seasons(vec![first_season])
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
}
