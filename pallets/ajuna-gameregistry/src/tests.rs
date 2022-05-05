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
use ajuna_common::{Runner, RunnerState};
use frame_support::{assert_noop, assert_ok};

pub const ALICE: <Test as frame_system::Config>::AccountId = 1u64;
pub const BOB: <Test as frame_system::Config>::AccountId = 2u64;

#[test]
fn should_queue_player() {
	new_test_ext().execute_with(|| {
		assert_ok!(Registry::queue(Origin::signed(ALICE)));
		assert_noop!(Registry::queue(Origin::signed(ALICE)), Error::<Test>::AlreadyQueued);
		assert!(MockRunner::get_state(GLOBAL_IDENTIFIER).is_none());
	});
}

#[test]
fn should_create_game() {
	new_test_ext().execute_with(|| {
		assert_ok!(Registry::queue(Origin::signed(ALICE)));
		assert_ok!(Registry::queue(Origin::signed(BOB)));
		assert!(MockRunner::get_state(GLOBAL_IDENTIFIER).is_some());
	});
}

#[test]
fn should_allow_game_to_be_acknowledged() {
	new_test_ext().execute_with(|| {
		assert_ok!(Registry::queue(Origin::signed(ALICE)));
		assert_ok!(Registry::queue(Origin::signed(BOB)));
		assert_ok!(Registry::ack_game(Origin::signed(TEE_ID), vec![GLOBAL_IDENTIFIER]));
		let game = Game { players: vec![ALICE, BOB], tee_id: Some(TEE_ID), winner: None };
		assert_eq!(
			Some(RunnerState::Accepted(game.encode().into())),
			MockRunner::get_state(GLOBAL_IDENTIFIER)
		);
	});
}

#[test]
fn should_return_batch_too_large() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Registry::ack_game(
				Origin::signed(TEE_ID),
				vec![GLOBAL_IDENTIFIER, GLOBAL_IDENTIFIER, GLOBAL_IDENTIFIER]
			),
			Error::<Test>::AcknowledgeBatchTooLarge
		);
	});
}

#[test]
fn should_finish_game() {
	new_test_ext().execute_with(|| {
		assert_ok!(Registry::queue(Origin::signed(ALICE)));
		assert_ok!(Registry::queue(Origin::signed(BOB)));
		assert_ok!(Registry::ack_game(Origin::signed(TEE_ID), vec![GLOBAL_IDENTIFIER]));
		assert_ok!(Registry::finish_game(Origin::signed(TEE_ID), GLOBAL_IDENTIFIER, ALICE));
		let game = Game { players: vec![ALICE, BOB], tee_id: Some(TEE_ID), winner: Some(ALICE) };
		assert_eq!(
			Some(RunnerState::Finished(game.encode().into())),
			MockRunner::get_state(GLOBAL_IDENTIFIER)
		);
	});
}
