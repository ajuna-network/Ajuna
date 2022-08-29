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

use crate::{dot4gravity::*, mock::*, *};
use frame_support::{assert_noop, assert_ok};

const ALICE: u32 = 1;
const BOB: u32 = 2;
const CHARLIE: u32 = 3;
const ERIN: u32 = 5;

const BOARD_ID: u32 = 1;
const TEST_COORD: Coordinates = Coordinates::new(0, 0);
// The seed below generates the following board, where o is empty and x is block:
// [o, o, o, o, o, o, o, o, o, o],
// [o, o, o, x, o, o, o, o, o, o],
// [o, o, x, o, o, o, o, o, o, o],
// [o, x, o, o, o, o, o, o, x, o],
// [o, o, o, o, x, o, x, o, o, o],
// [o, o, o, o, o, o, o, o, x, o],
// [o, o, o, x, o, o, x, o, o, o],
// [o, o, o, o, o, o, o, o, o, o],
// [x, o, o, o, o, o, o, o, o, o],
// [o, o, o, o, o, o, o, o, o, o],
const TEST_SEED: u32 = 7357;

#[test]
fn should_create_new_game() {
	new_test_ext().execute_with(|| {
		// Board ids start at 1, hence the first game will be 1
		let board_id = 1;
		// We can't start a board game without any players
		assert_noop!(
			AjunaBoard::new_game(Origin::signed(ALICE), board_id, BTreeSet::new()),
			Error::<Test>::NotEnoughPlayers
		);

		// We are limited to the number of players we can have
		assert_noop!(
			AjunaBoard::new_game(
				Origin::signed(ALICE),
				board_id,
				BTreeSet::from([BOB, CHARLIE, ERIN])
			),
			Error::<Test>::TooManyPlayers
		);

		// And trying to create a new game will fail
		assert_noop!(
			AjunaBoard::new_game(Origin::signed(ALICE), board_id, BTreeSet::from([BOB])),
			Error::<Test>::InvalidStateFromGame
		);

		// Create a new game with players; Alice, Bob and Charlie
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id,
			BTreeSet::from([BOB, CHARLIE])
		));
		assert_noop!(
			AjunaBoard::new_game(Origin::signed(ALICE), board_id, BTreeSet::from([BOB, CHARLIE])),
			Error::<Test>::BoardExists
		);

		// Try to create a new game with same players
		let new_board_id = board_id + 1;
		assert_noop!(
			AjunaBoard::new_game(
				Origin::signed(ALICE),
				new_board_id,
				BTreeSet::from([BOB, CHARLIE])
			),
			Error::<Test>::PlayerAlreadyInGame
		);

		// Confirm the board game we have created is what we intended
		let board_game = BoardStates::<Test>::get(board_id).expect("board_id should exist");

		assert_eq!(
			board_game.players.into_inner(),
			[BOB, CHARLIE],
			"we should have the following players; Bob and Charlie"
		);

		assert!(PlayerBoards::<Test>::contains_key(BOB), "Bob should be on the board");
		assert!(PlayerBoards::<Test>::contains_key(CHARLIE), "Charlie should be on the board");
		assert!(!PlayerBoards::<Test>::contains_key(ALICE), "Alice should not be on the board");

		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameCreated {
				board_id,
				players: vec![BOB, CHARLIE],
			}),
		);
	});

	new_test_ext().execute_with(|| {
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			BOARD_ID,
			BTreeSet::from([BOB, CHARLIE])
		));
		System::assert_last_event(mock::Event::AjunaBoard(crate::Event::GameCreated {
			board_id: BOARD_ID,
			players: vec![BOB, CHARLIE],
		}));
		assert!(BoardStates::<Test>::contains_key(BOARD_ID));
		assert!(Seed::<Test>::get().is_none());

		let tests_for_errors = vec![
			(BTreeSet::from([BOB, CHARLIE]), BOARD_ID, Error::<Test>::BoardExists),
			(BTreeSet::from([ALICE, BOB]), 11, Error::<Test>::PlayerAlreadyInGame),
			(BTreeSet::from([ALICE, CHARLIE]), 22, Error::<Test>::PlayerAlreadyInGame),
			// TODO: should we early reject with NotEnoughPlayer?
			(BTreeSet::from([ALICE]), 33, Error::<Test>::InvalidStateFromGame),
			// TODO: should we early reject with NotEnoughPlayer?
			(BTreeSet::from([BOB]), 44, Error::<Test>::PlayerAlreadyInGame),
			(BTreeSet::from([CHARLIE]), 44, Error::<Test>::PlayerAlreadyInGame),
			// TODO: should we early reject with TooManyPlayers?
			(BTreeSet::from([ALICE, BOB, CHARLIE]), 55, Error::<Test>::PlayerAlreadyInGame),
		];
		for (players, board_id, expected_error) in tests_for_errors {
			assert_noop!(
				AjunaBoard::new_game(Origin::signed(ALICE), board_id, players),
				expected_error
			);
		}
	});
}

#[test]
fn should_play_and_mutate_game_state() {
	new_test_ext().execute_with(|| {
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			BOARD_ID,
			BTreeSet::from([BOB, CHARLIE])
		));
		let board_state_before = BoardStates::<Test>::get(BOARD_ID).unwrap();

		assert_ok!(AjunaBoard::play_turn(Origin::signed(CHARLIE), Turn::DropBomb(TEST_COORD)));
		let board_state_after = BoardStates::<Test>::get(BOARD_ID).unwrap();

		assert_ne!(board_state_before.state.board, board_state_after.state.board);
		assert_eq!(board_state_before.state.bombs, [(BOB, 3), (CHARLIE, 3)]);
		assert_eq!(board_state_after.state.bombs, [(BOB, 3), (CHARLIE, 3 - 1)]);
	});
}

#[test]
fn should_play_turn_and_finish_game() {
	new_test_ext().execute_with(|| {
		Seed::<Test>::put(TEST_SEED);
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			BOARD_ID,
			BTreeSet::from([BOB, ERIN])
		));
		assert_noop!(
			AjunaBoard::play_turn(Origin::signed(ALICE), Turn::DropBomb(TEST_COORD)),
			Error::<Test>::NotPlaying
		);

		// Bomb phase
		let play_drop_bomb = |coord: Coordinates| {
			let _ = AjunaBoard::play_turn(Origin::signed(BOB), Turn::DropBomb(coord));
			let _ = AjunaBoard::play_turn(Origin::signed(ERIN), Turn::DropBomb(coord));
		};
		play_drop_bomb(Coordinates::new(9, 9));
		play_drop_bomb(Coordinates::new(8, 8));
		play_drop_bomb(Coordinates::new(7, 7));

		// Play phase
		let play_drop_stone = || {
			let win_position = (Side::North, 0);
			let lose_position = (Side::North, 9);
			let _ = AjunaBoard::play_turn(Origin::signed(BOB), Turn::DropStone(win_position));
			let _ = AjunaBoard::play_turn(Origin::signed(ERIN), Turn::DropStone(lose_position));
		};
		play_drop_stone();
		play_drop_stone();
		play_drop_stone();
		play_drop_stone();

		// check if game has finished
		System::assert_last_event(mock::Event::AjunaBoard(crate::Event::GameFinished {
			board_id: BOARD_ID,
			winner: BOB,
		}));
		assert_eq!(BoardWinners::<Test>::get(BOARD_ID), Some(BOB));
		assert_ne!(Seed::<Test>::get(), Some(TEST_SEED));

		// finish game and check
		assert_ok!(AjunaBoard::finish_game(Origin::signed(ALICE), BOARD_ID));
		assert!(BoardStates::<Test>::get(BOARD_ID).is_none());
		assert!(BoardWinners::<Test>::get(BOARD_ID).is_none());
	})
}

#[test]
fn should_be_able_to_dispute_a_stale_board() {
	new_test_ext().execute_with(|| {
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			BOARD_ID,
			BTreeSet::from([BOB, CHARLIE])
		));

		assert_ok!(AjunaBoard::play_turn(Origin::signed(CHARLIE), Turn::DropBomb(TEST_COORD)));

		// We shouldn't be able to dispute an active game
		assert_noop!(
			AjunaBoard::dispute_game(Origin::signed(ALICE), BOARD_ID),
			Error::<Test>::DisputeFailed
		);

		// Jump to the future when the game is now stale
		System::set_block_number(IdleBoardTimeout::get() + 1);

		assert_ok!(AjunaBoard::dispute_game(Origin::signed(ALICE), BOARD_ID));

		// Some final checks that the dispute awarding the game actually awards and clears the game
		System::assert_last_event(mock::Event::AjunaBoard(crate::Event::GameFinished {
			board_id: BOARD_ID,
			winner: BOB,
		}));

		assert_eq!(BoardWinners::<Test>::get(BOARD_ID), Some(BOB));
		assert_ne!(Seed::<Test>::get(), Some(TEST_SEED));

		// finish game and check
		assert_ok!(AjunaBoard::finish_game(Origin::signed(ALICE), BOARD_ID));
		assert!(BoardStates::<Test>::get(BOARD_ID).is_none());
		assert!(BoardWinners::<Test>::get(BOARD_ID).is_none());
	});
}
