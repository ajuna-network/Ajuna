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

const ALICE: u32 = 1;

pub mod season {
	use crate::season::Season;

	#[test]
	fn season_ok() {
		let season = Season::new(1, 10, 20, 1, 1);
		assert!(!season.is_early_access_start_too_late());
		assert!(!season.is_season_start_too_late());
	}

	#[test]
	fn season_not_overlapped() {
		let first_season = Season::new(1, 10, 20, 1, 1);
		let second_season = Season::new(21, 30, 40, 1, 1);

		assert!(!Season::are_seasons_overlapped(&first_season, &second_season));
	}

	#[test]
	fn season_early_access_start_is_too_late_when_set_after_start() {
		let season = Season::new(30, 10, 20, 1, 1);

		assert!(season.is_early_access_start_too_late());
	}
}
pub mod new_season {
	use frame_support::{assert_noop, assert_ok};

	use crate::{season::Season, mock::*, tests::ALICE, *};

	#[test]
	fn new_season_should_create_first_season() {
		new_test_ext().execute_with(|| {
			let new_season = Season::new(1, 5, 10, 0, 0);

			assert_ok!(AAA::new_season(Origin::signed(ALICE), new_season));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(Season::new(1, 5, 10, 0, 0)))
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_start_block_smaller_than_end_block_in_current_season() {
		new_test_ext().execute_with(|| {
			let new_season = Season::new(1, 5, 10, 0, 0);

			assert_ok!(AAA::new_season(Origin::signed(ALICE), new_season));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(Season::new(1, 5, 10, 0, 0)))
			);

			// ensure new season’s early access start > last season’s end
			let new_season = Season::new(3, 7, 10, 0, 0);
			assert_noop!(
				AAA::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::EarlyAccessStartsTooEarly
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_early_access_block_greater_than_start() {
		new_test_ext().execute_with(|| {
			// ensure new season’s early access start < new season’s start
			let new_season = Season::new(6, 3, 10, 0, 0);
			assert_noop!(
				AAA::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::EarlyAccessStartsTooLate
			);
		});
	}

	#[test]
	fn new_season_should_return_error_when_start_block_greater_than_end() {
		new_test_ext().execute_with(|| {
			// ensure new season’s start < new season’s end
			let new_season = Season::new(11, 12, 10, 0, 0);
			assert_noop!(
				AAA::new_season(Origin::signed(ALICE), new_season),
				Error::<Test>::SeasonStartsTooLate
			);
		});
	}
}

pub mod update_season {
	use frame_support::{assert_noop, assert_ok};

	use crate::{season::Season, mock::*, tests::ALICE, *};

	#[test]
	fn update_season_should_return_error_when_season_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 10, Season::new(1, 12, 30, 1, 1)),
				Error::<Test>::UnknownSeason
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_season_to_update_ends_after_next_season_start() {
		new_test_ext().execute_with(|| {
			// Create two seasons
			let first_season = Season::new(1, 5, 10, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), first_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(first_season))
			);
			let second_season = Season::new(11, 15, 20, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), second_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(second_season))
			);

			// Update the first one to end after the second has started
			let first_season_update = Season::new(1, 5, 14, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 0, first_season_update),
				Error::<Test>::SeasonEndsTooLate
			);
		});
	}

	#[test]
	fn update_season_should_be_ok_when_season_to_update_ends_before_next_season_start() {
		new_test_ext().execute_with(|| {
			// Create two seasons
			let first_season = Season::new(1, 5, 10, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), first_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(first_season))
			);

			let second_season = Season::new(11, 15, 20, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), second_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(second_season))
			);

			// Update the first one to end before the second has started
			let first_season_update = Season::new(1, 5, 8, 0, 0);
			assert_ok!(AAA::update_season(Origin::signed(ALICE), 0, first_season_update.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::SeasonUpdated(first_season_update, 0))
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_early_access_start_set_before_or_equal_previous_season_end(
	) {
		new_test_ext().execute_with(|| {
			// Create two seasons
			let first_season = Season::new(1, 5, 10, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), first_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(first_season))
			);
			let second_season = Season::new(11, 15, 20, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), second_season.clone()));
			assert_eq!(
				last_event(),
				mock::Event::AAA(crate::Event::NewSeasonCreated(second_season))
			);

			// Update the second season and set early access start before previous season end
			let second_season_update = Season::new(8, 15, 20, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 1, second_season_update.clone()),
				Error::<Test>::EarlyAccessStartsTooEarly
			);

			let second_season_update = Season::new(9, 15, 20, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 1, second_season_update),
				Error::<Test>::EarlyAccessStartsTooEarly
			);

			let second_season_update = Season::new(10, 15, 20, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 1, second_season_update),
				Error::<Test>::EarlyAccessStartsTooEarly
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_start_set_before_or_equal_early_access_start() {
		new_test_ext().execute_with(|| {
			let season = Season::new(1, 5, 10, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), season.clone()));
			assert_eq!(last_event(), mock::Event::AAA(crate::Event::NewSeasonCreated(season)));

			let season_update = Season::new(5, 1, 10, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::EarlyAccessStartsTooLate
			);

			let season_update = Season::new(5, 5, 10, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::EarlyAccessStartsTooLate
			);
		});
	}

	#[test]
	fn update_season_should_return_error_when_start_set_after_end() {
		new_test_ext().execute_with(|| {
			let season = Season::new(1, 5, 10, 0, 0);
			assert_ok!(AAA::new_season(Origin::signed(ALICE), season.clone()));
			assert_eq!(last_event(), mock::Event::AAA(crate::Event::NewSeasonCreated(season)));

			// Update the second season and set early access start before previous season end
			let season_update = Season::new(1, 15, 10, 0, 0);
			assert_noop!(
				AAA::update_season(Origin::signed(ALICE), 0, season_update),
				Error::<Test>::SeasonStartsTooLate
			);
		});
	}
}
