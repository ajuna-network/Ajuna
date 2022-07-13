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

use crate::{guessing::THE_NUMBER, mock::*, *};
use frame_support::{assert_noop, assert_ok};

const ALICE: u32 = 1;
const BOB: u32 = 2;
const CHARLIE: u32 = 3;
const ERIN: u32 = 4;

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
}

#[test]
fn should_play_a_turn_for_a_player() {
	new_test_ext().execute_with(|| {
		// Create a game with Bob and Charlie as players
		// Play the game until someone wins
		let board_id = 1;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id,
			BTreeSet::from([BOB, CHARLIE])
		));
		assert_eq!(board_id, BoardStates::<Test>::get(board_id).expect("the board").board_id);
		assert_noop!(AjunaBoard::play_turn(Origin::signed(ALICE), 0u32), Error::<Test>::NotPlaying);
		assert_ok!(AjunaBoard::play_turn(Origin::signed(BOB), 1u32));
		assert_noop!(AjunaBoard::play_turn(Origin::signed(BOB), 1u32), Error::<Test>::InvalidTurn);
		assert_ok!(AjunaBoard::play_turn(Origin::signed(CHARLIE), 1u32));
		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameCreated {
				board_id,
				players: vec![BOB, CHARLIE],
			}),
			"Board with Bob and Charlie created"
		);
	});
}

#[test]
fn should_finish_game_and_not_allow_new_game() {
	new_test_ext().execute_with(|| {
		let board_id = 1;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id,
			BTreeSet::from([BOB, CHARLIE])
		));
		assert_ok!(AjunaBoard::play_turn(Origin::signed(BOB), THE_NUMBER));
		// Game is now finished, but until we `finish_game` the players won't be able to play a new
		// game
		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameFinished { board_id, winner: BOB }),
			"Bob won"
		);
		assert_eq!(
			BoardWinners::<Test>::get(board_id).unwrap(),
			BOB,
			"Board stored to state with winner as Bob"
		);
		assert_noop!(AjunaBoard::play_turn(Origin::signed(BOB), 1u32), Error::<Test>::InvalidTurn);
		assert_eq!(
			PlayerBoards::<Test>::iter_keys().count(),
			2,
			"Bob and Charlie waiting on finishing the game"
		);
		// A new board with the same players
		let new_board_id = board_id + 1;
		assert_noop!(
			AjunaBoard::new_game(
				Origin::signed(ALICE),
				new_board_id,
				BTreeSet::from([BOB, CHARLIE])
			),
			Error::<Test>::PlayerAlreadyInGame
		);
		// Finish the game
		assert_ok!(AjunaBoard::finish_game(Origin::signed(ALICE), board_id));
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			new_board_id,
			BTreeSet::from([BOB, CHARLIE])
		));
		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameCreated {
				board_id: new_board_id,
				players: vec![BOB, CHARLIE],
			}),
			"Board with Bob and Charlie created"
		);
	});
}

#[test]
fn game_should_properly_expire() {
	new_test_ext().execute_with(|| {
		let board_id = 1;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id,
			BTreeSet::from([BOB, CHARLIE])
		));

		// We 'advance' to block number 20
		System::set_block_number(20);

		// We force the call to 'on_idle' to trigger the validation of state
		AjunaBoard::on_idle(mock::System::block_number(), 10_000);

		// Here we check how Bob has automatically won because of inactivity
		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameFinished { board_id, winner: BOB }),
			"Bob should have won"
		);
		assert_eq!(
			BoardWinners::<Test>::get(board_id).unwrap(),
			BOB,
			"Board stored to state with winner as Bob"
		);
		assert_noop!(AjunaBoard::play_turn(Origin::signed(BOB), 1u32), Error::<Test>::InvalidTurn);
	});
}
