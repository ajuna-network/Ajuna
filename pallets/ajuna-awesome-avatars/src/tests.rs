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
	fn only_root_should_set_organizer() {
		new_test_ext().execute_with(|| {
			let root_origin = Origin::root();
			let non_root_origin = Origin::signed(ALICE);

			assert_noop!(
				AjunaAwesomeAvatars::set_organizer(non_root_origin, HILDA),
				Error::<Test>::InsufficientPrivileges
			);

			assert_ok!(AjunaAwesomeAvatars::set_organizer(root_origin, HILDA));

			assert_eq!(Organizer::<Test>::get(), Some(HILDA), "Organizer should be Hilda");

			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::OrganizerSet { organizer: HILDA }),
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_previous_organizer() {
		new_test_ext().execute_with(|| {
			let root_origin = Origin::root();

			assert_ok!(AjunaAwesomeAvatars::set_organizer(root_origin.clone(), BOB));

			assert_eq!(Organizer::<Test>::get(), Some(BOB), "Organizer should be Bob");

			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::OrganizerSet { organizer: BOB }),
			);

			assert_ok!(AjunaAwesomeAvatars::set_organizer(root_origin, FLORINA));

			assert_eq!(Organizer::<Test>::get(), Some(FLORINA), "Organizer should be Florina");

			assert_eq!(
				last_event(),
				mock::Event::AjunaAwesomeAvatars(crate::Event::OrganizerReplaced {
					prev_organizer: BOB,
					new_organizer: FLORINA
				}),
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_no_organizer_set() {
		new_test_ext().execute_with(|| {
			let organizer = Origin::signed(DELTHEA);

			assert_noop!(
				AjunaAwesomeAvatars::ensure_organizer(organizer),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_fail_if_account_is_no_organizer() {
		new_test_ext().execute_with(|| {
			let organizer = Origin::signed(DELTHEA);
			let root_origin = Origin::root();

			assert_ok!(AjunaAwesomeAvatars::set_organizer(root_origin.clone(), ERIN));
			assert_noop!(
				AjunaAwesomeAvatars::ensure_organizer(organizer),
				Error::<Test>::AccountIsNotOrganizer
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		new_test_ext().execute_with(|| {
			let organizer = Origin::signed(CHARLIE);
			let root_origin = Origin::root();

			assert_ok!(AjunaAwesomeAvatars::set_organizer(root_origin.clone(), CHARLIE));
			assert_ok!(AjunaAwesomeAvatars::ensure_organizer(organizer));
		});
	}
}
