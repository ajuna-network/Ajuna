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

use crate::{mock::*, Running, *};
use ajuna_common::{Runner, RunnerState, State};
use frame_support::{assert_noop, assert_ok};
#[test]
fn should_create_runner() {
	new_test_ext().execute_with(|| {
		let state: State = vec![].into();
		let identifier = Running::<Test>::create::<MockGetIdentifier>(state.clone())
			.expect("create a new runner");
		assert_eq!(
			AjunaRunner::runners(identifier),
			Some(RunnerState::Queued(state.clone())),
			"storage contains queued runner"
		);
		// Mock provides the same identifier
		assert!(
			Running::<Test>::create::<MockGetIdentifier>(state).is_none(),
			"duplicates not allowed"
		);
	});
}

#[test]
fn should_accept_runner_and_apply_state() {
	new_test_ext().execute_with(|| {
		let state: State = vec![].into();
		let new_state: State = b"accepted".to_vec().into();
		let identifier =
			Running::<Test>::create::<MockGetIdentifier>(state).expect("create a new runner");
		assert_ok!(Running::<Test>::accept(&identifier, Some(new_state.clone())));
		assert_eq!(
			AjunaRunner::runners(identifier),
			Some(RunnerState::Accepted(new_state)),
			"storage contains accepted runner"
		);
	});
}

#[test]
fn should_accept_runner_with_no_state_update() {
	new_test_ext().execute_with(|| {
		let state: State = vec![].into();
		let identifier = Running::<Test>::create::<MockGetIdentifier>(state.clone())
			.expect("create a new runner");
		assert_ok!(Running::<Test>::accept(&identifier, None));
		assert_eq!(
			AjunaRunner::runners(identifier),
			Some(RunnerState::Accepted(state)),
			"storage contains accepted runner"
		);
	});
}

#[test]
fn should_finish_runner_and_apply_state() {
	new_test_ext().execute_with(|| {
		let state: State = vec![].into();
		let identifier =
			Running::<Test>::create::<MockGetIdentifier>(state).expect("create a new runner");
		assert_ok!(Running::<Test>::accept(&identifier, None));
		let new_state: State = b"finished".to_vec().into();
		assert_ok!(Running::<Test>::finished(&identifier, Some(new_state.clone())));
		assert_eq!(
			AjunaRunner::runners(identifier),
			Some(RunnerState::Finished(new_state)),
			"storage contains accepted runner"
		);
	});
}

#[test]
fn should_finish_runner_no_state_update() {
	new_test_ext().execute_with(|| {
		let state: State = vec![].into();
		let identifier = Running::<Test>::create::<MockGetIdentifier>(state.clone())
			.expect("create a new runner");
		assert_ok!(Running::<Test>::accept(&identifier, None));
		assert_ok!(Running::<Test>::finished(&identifier, None));
		assert_eq!(
			AjunaRunner::runners(identifier),
			Some(RunnerState::Finished(state)),
			"storage contains accepted runner"
		);
	});
}

#[test]
fn should_remove_runner() {
	new_test_ext().execute_with(|| {
		let state: State = vec![].into();
		let identifier =
			Running::<Test>::create::<MockGetIdentifier>(state).expect("create a new runner");
		assert_ok!(Running::<Test>::remove(&identifier));
		assert_noop!(Running::<Test>::remove(&identifier), Error::<Test>::UnknownRunner);

		assert_eq!(AjunaRunner::runners(identifier), None, "storage doesn't contain runner");
	});
}
