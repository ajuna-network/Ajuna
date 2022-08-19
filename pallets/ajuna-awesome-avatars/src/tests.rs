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

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

use season::*;

#[cfg(test)]
mod organizer {
	use super::*;

	const ALICE: u32 = 1;
	const BOB: u32 = 2;
	const CHARLIE: u32 = 3;
	const DELTHEA: u32 = 4;
	const ERIN: u32 = 5;
	const FLORINA: u32 = 6;
	const HILDA: u32 = 7;

	#[test]
	fn set_organizer_should_only_accept_root_caller() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AjunaAwesomeAvatars::set_organizer(Origin::signed(ALICE), HILDA),
				DispatchError::BadOrigin
			);
			assert_ok!(AjunaAwesomeAvatars::set_organizer(Origin::root(), HILDA));

			assert_eq!(Organizer::<Test>::get(), Some(HILDA), "Organizer should be Hilda");
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::OrganizerSet { organizer: HILDA }),
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_existing_organizer() {
		new_test_ext().execute_with(|| {
			assert_ok!(AjunaAwesomeAvatars::set_organizer(Origin::root(), BOB));
			assert_eq!(Organizer::<Test>::get(), Some(BOB), "Organizer should be Bob");
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::OrganizerSet { organizer: BOB }),
			);

			assert_ok!(AjunaAwesomeAvatars::set_organizer(Origin::root(), FLORINA));
			assert_eq!(Organizer::<Test>::get(), Some(FLORINA), "Organizer should be Florina");
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::OrganizerSet { organizer: FLORINA }),
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_no_organizer_set() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AjunaAwesomeAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_caller_is_not_organizer() {
		new_test_ext().execute_with(|| {
			assert_ok!(AjunaAwesomeAvatars::set_organizer(Origin::root(), ERIN));
			assert_noop!(
				AjunaAwesomeAvatars::ensure_organizer(Origin::signed(DELTHEA)),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		new_test_ext().execute_with(|| {
			assert_ok!(AjunaAwesomeAvatars::set_organizer(Origin::root(), CHARLIE));
			assert_ok!(AjunaAwesomeAvatars::ensure_organizer(Origin::signed(CHARLIE)));
		});
	}
}

pub mod new_season {
	use super::*;

	const ALICE: u32 = 1;

	#[test]
	fn new_season_should_create_first_season() {
		new_test_ext().execute_with(|| {
			let new_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };

			assert_ok!(AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), new_season));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(Season {
					early_start: 1,
					start: 5,
					end: 10,
					max_mints: 1,
					max_mythical_mints: 1
				}))
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_start_block_smaller_than_end_block_in_current_season() {
		new_test_ext().execute_with(|| {
			let new_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };

			assert_ok!(AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), new_season));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(Season {
					early_start: 1,
					start: 5,
					end: 10,
					max_mints: 1,
					max_mythical_mints: 1
				}))
			);

			// ensure new season’s early access start > last season’s end
			let new_season =
				Season { early_start: 3, start: 7, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::EarlyStartTooEarly
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_early_access_block_greater_than_start() {
		new_test_ext().execute_with(|| {
			// ensure new season’s early access start < new season’s start
			let new_season =
				Season { early_start: 6, start: 3, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::EarlyStartTooLate
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_start_block_greater_than_end() {
		new_test_ext().execute_with(|| {
			// ensure new season’s start < new season’s end
			let new_season =
				Season { early_start: 11, start: 12, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}
}

pub mod update_season {
	use super::*;

	const ALICE: u32 = 1;

	#[test]
	fn update_season_should_return_error_when_season_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AjunaAwesomeAvatars::update_season(
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
			// Create two seasons
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(
				Origin::signed(ALICE),
				first_season.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(first_season))
			);
			let second_season =
				Season { early_start: 11, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(
				Origin::signed(ALICE),
				second_season.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(second_season))
			);

			// Update the first one to end after the second has started
			let first_season_update =
				Season { early_start: 1, start: 5, end: 14, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(Origin::signed(ALICE), 0, first_season_update),
				Error::<Test>::SeasonEndTooLate
			);
		});
	}

	#[test]
	fn update_season_should_be_ok_when_season_to_update_ends_before_next_season_start() {
		new_test_ext().execute_with(|| {
			// Create two seasons
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(
				Origin::signed(ALICE),
				first_season.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(first_season))
			);

			let second_season =
				Season { early_start: 11, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(
				Origin::signed(ALICE),
				second_season.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(second_season))
			);

			// Update the first one to end before the second has started
			let first_season_update =
				Season { early_start: 1, start: 5, end: 8, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::update_season(
				Origin::signed(ALICE),
				0,
				first_season_update.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::SeasonUpdated(
					first_season_update,
					0
				))
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_early_start_set_before_or_equal_previous_season_end()
	{
		new_test_ext().execute_with(|| {
			// Create two seasons
			let first_season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(
				Origin::signed(ALICE),
				first_season.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(first_season))
			);
			let second_season =
				Season { early_start: 11, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(
				Origin::signed(ALICE),
				second_season.clone()
			));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(second_season))
			);

			// Update the second season and set early access start before previous season end
			let second_season_update =
				Season { early_start: 8, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(
					Origin::signed(ALICE),
					1,
					second_season_update.clone()
				),
				Error::<Test>::EarlyStartTooEarly
			);

			let second_season_update =
				Season { early_start: 9, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(Origin::signed(ALICE), 1, second_season_update),
				Error::<Test>::EarlyStartTooEarly
			);

			let second_season_update =
				Season { early_start: 10, start: 15, end: 20, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(Origin::signed(ALICE), 1, second_season_update),
				Error::<Test>::EarlyStartTooEarly
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_start_set_before_or_equal_early_start() {
		new_test_ext().execute_with(|| {
			let season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(season))
			);

			let season_update =
				Season { early_start: 5, start: 1, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::EarlyStartTooLate
			);

			let season_update =
				Season { early_start: 5, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::EarlyStartTooLate
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_start_set_after_end() {
		new_test_ext().execute_with(|| {
			let season =
				Season { early_start: 1, start: 5, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_ok!(AjunaAwesomeAvatars::new_season(Origin::signed(ALICE), season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::NewSeasonCreated(season))
			);

			// Update the second season and set early access start before previous season end
			let season_update =
				Season { early_start: 1, start: 15, end: 10, max_mints: 1, max_mythical_mints: 1 };
			assert_noop!(
				AjunaAwesomeAvatars::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::SeasonStartTooLate
			);
		});
	}
}
