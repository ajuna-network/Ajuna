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

use frame_support::{assert_noop, assert_ok};

use crate::{ajuna_awesome_avatar::Season, mock::*, *};

const ALICE: u32 = 1;

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
