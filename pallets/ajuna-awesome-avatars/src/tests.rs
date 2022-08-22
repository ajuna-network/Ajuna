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
use sp_runtime::DispatchError;

mod organizer {
	use super::*;

	#[test]
	fn set_organizer_should_only_accept_root_caller() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::set_organizer(Origin::signed(ALICE), HILDA),
				DispatchError::BadOrigin
			);
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), HILDA));

			assert_eq!(Organizer::<Test>::get(), Some(HILDA), "Organizer should be Hilda");
			assert_eq!(
				last_event(),
				mock::Event::AwesomeAvatars(crate::Event::OrganizerSet { organizer: HILDA }),
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_existing_organizer() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), BOB));
			assert_eq!(Organizer::<Test>::get(), Some(BOB), "Organizer should be Bob");
			assert_eq!(
				last_event(),
				mock::Event::AwesomeAvatars(crate::Event::OrganizerSet { organizer: BOB }),
			);

			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), FLORINA));
			assert_eq!(Organizer::<Test>::get(), Some(FLORINA), "Organizer should be Florina");
			assert_eq!(
				last_event(),
				mock::Event::AwesomeAvatars(crate::Event::OrganizerSet { organizer: FLORINA }),
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_no_organizer_set() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AwesomeAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_caller_is_not_organizer() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ERIN));
			assert_noop!(
				AwesomeAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), CHARLIE));
			assert_ok!(AwesomeAvatars::ensure_organizer(Origin::signed(CHARLIE)));
		});
	}
}

mod season {
	use super::*;

	#[test]
	fn new_season_should_reject_non_organizer_as_caller() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_noop!(
				AwesomeAvatars::new_season(
					Origin::signed(BOB),
					Season {
						early_start: 1,
						start: 2,
						end: 3,
						max_mints: 4,
						max_mythical_mints: 5,
					}
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn new_season_should_work() {
		new_test_ext().execute_with(|| {
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));
			assert_eq!(AwesomeAvatars::seasons(1), Some(first_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AwesomeAvatars(crate::Event::NewSeasonCreated(first_season))
			);

			let second_season =
				Season { early_start: 11, start: 12, end: 13, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), second_season.clone()));
			assert_eq!(AwesomeAvatars::seasons(2), Some(second_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AwesomeAvatars(crate::Event::NewSeasonCreated(second_season))
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		new_test_ext().execute_with(|| {
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));

			let second_season =
				Season { early_start: 3, start: 7, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert!(second_season.early_start < second_season.start);
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), second_season),
				Error::<Test>::EarlyStartTooEarly
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_early_start_is_later_than_start() {
		new_test_ext().execute_with(|| {
			let new_season =
				Season { early_start: 6, start: 3, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert!(new_season.early_start > new_season.start);
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), new_season,),
				Error::<Test>::EarlyStartTooLate
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_start_is_later_than_end() {
		new_test_ext().execute_with(|| {
			let new_season =
				Season { early_start: 11, start: 12, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert!(new_season.early_start < new_season.start);
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_noop!(
				AwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}

	#[test]
	fn update_season_should_reject_non_organizer_as_caller() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
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
					}
				),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn update_season_should_work() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			// Create two seasons
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));
			let second_season =
				Season { early_start: 11, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), second_season.clone()));

			// Update the first one to end before the second has started
			let first_season_update =
				Season { early_start: 1, start: 5, end: 8, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::update_season(
				Origin::signed(ALICE),
				1,
				first_season_update.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AwesomeAvatars(crate::Event::SeasonUpdated(first_season_update, 1))
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_season_not_found() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_noop!(
				AwesomeAvatars::update_season(
					Origin::signed(ALICE),
					10,
					Season {
						early_start: 1,
						start: 12,
						end: 30,
						max_mints: 1,
						max_mythical_mints: 1
					}
				),
				Error::<Test>::UnknownSeason
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_season_to_update_ends_after_next_season_start() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			// Create two seasons
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));
			let second_season =
				Season { early_start: 11, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), second_season.clone()));

			// Update the first one to end after the second has started
			let first_season_update =
				Season { early_start: 1, start: 5, end: 14, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 1, first_season_update),
				Error::<Test>::SeasonEndTooLate
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_is_earlier_than_previous_season_end() {
		new_test_ext().execute_with(|| {
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			// Create two seasons
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), first_season.clone()));
			let second_season =
				Season { early_start: 11, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), second_season.clone()));

			// Update the second season and set early start before previous season end
			let second_season_update =
				Season { early_start: 8, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
				Error::<Test>::EarlyStartTooEarly
			);

			let second_season_update =
				Season { early_start: 9, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
				Error::<Test>::EarlyStartTooEarly
			);

			let second_season_update =
				Season { early_start: 10, start: 15, end: 20, max_mints: 2, max_mythical_mints: 1 };
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 2, second_season_update),
				Error::<Test>::EarlyStartTooEarly
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_is_earlier_than_or_equal_to_start() {
		new_test_ext().execute_with(|| {
			let season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), season.clone()));

			let season_update =
				Season { early_start: 5, start: 1, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert!(season_update.early_start > season_update.start);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::EarlyStartTooLate
			);

			let season_update =
				Season { early_start: 5, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert!(season_update.early_start == season_update.start);
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::EarlyStartTooLate
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_start_is_later_than_end() {
		new_test_ext().execute_with(|| {
			let season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AwesomeAvatars::set_organizer(Origin::root(), ALICE));
			assert_ok!(AwesomeAvatars::new_season(Origin::signed(ALICE), season.clone()));

			// Update the second season and set early access start before previous season end
			let season_update =
				Season { early_start: 1, start: 15, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AwesomeAvatars::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}
}
