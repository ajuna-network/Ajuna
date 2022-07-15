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
const DELTHEA: u32 = 4;
const ERIN: u32 = 5;
const FLORINA: u32 = 6;
const HILDA: u32 = 7;

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

fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			AjunaBoard::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		AjunaBoard::on_initialize(System::block_number());
	}
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

		// We advance to block number 20
		run_to_block(20);

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

#[test]
fn game_expiry_should_properly_extend_after_play() {
	new_test_ext().execute_with(|| {
		let board_id = 1;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id,
			BTreeSet::from([BOB, CHARLIE])
		));

		let expiry_increase = MaxNumberOfIdleBlocks::get() as u64;
		let current_block = System::block_number();

		let game_expiry = BoardExpiries::<Test>::get(board_id);
		assert_eq!(
			game_expiry,
			current_block + expiry_increase,
			"New expiry should be {}",
			current_block + expiry_increase
		);

		// Game should expire on this block during on_idle
		run_to_block(game_expiry);

		// We play a turn extending it's expiry
		assert_ok!(AjunaBoard::play_turn(Origin::signed(BOB), 1_u32));

		// We run on_idle with the remaining weight
		AjunaBoard::on_idle(mock::System::block_number(), 10_000);

		// The latest event is still the game creation
		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameCreated {
				board_id,
				players: vec![BOB, CHARLIE]
			}),
			"Should be a GameCreated event for {}",
			board_id
		);

		// We validate that the new expiry is set to the correct value
		let new_game_expiry = BoardExpiries::<Test>::get(board_id);

		assert!(new_game_expiry > game_expiry, "New expiry should be greater");
		assert_eq!(
			new_game_expiry,
			game_expiry + expiry_increase,
			"New expiry should be {}",
			game_expiry + expiry_increase
		);
	});
}

#[test]
fn game_expiry_should_only_affect_max_number_of_games_to_expire() {
	new_test_ext().execute_with(|| {
		let board_id_1 = 1;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id_1,
			BTreeSet::from([BOB, CHARLIE])
		));

		let board_id_2 = 2;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id_2,
			BTreeSet::from([DELTHEA, ERIN])
		));

		let board_id_3 = 3;
		assert_ok!(AjunaBoard::new_game(
			Origin::signed(ALICE),
			board_id_3,
			BTreeSet::from([FLORINA, HILDA])
		));

		// We 'advance' to block number 20
		run_to_block(20);

		// We force the call to 'on_idle' to trigger the validation of state
		AjunaBoard::on_idle(mock::System::block_number(), 10_000);

		let (event_1, event_2) = last_two_events();

		// Here we check how Bob has automatically won because of inactivity
		assert_eq!(
			event_1,
			mock::Event::AjunaBoard(crate::Event::GameFinished {
				board_id: board_id_2,
				winner: DELTHEA
			}),
			"Delthea should have won"
		);
		assert_eq!(
			BoardWinners::<Test>::get(board_id_2).unwrap(),
			DELTHEA,
			"Board stored to state with winner as Delthea"
		);
		assert_noop!(
			AjunaBoard::play_turn(Origin::signed(DELTHEA), 1_u32),
			Error::<Test>::InvalidTurn
		);

		// Here we check how Bob has automatically won because of inactivity
		assert_eq!(
			event_2,
			mock::Event::AjunaBoard(crate::Event::GameFinished {
				board_id: board_id_1,
				winner: BOB
			}),
			"Bob should have won"
		);
		assert_eq!(
			BoardWinners::<Test>::get(board_id_1).unwrap(),
			BOB,
			"Board stored to state with winner as Bob"
		);
		assert_noop!(AjunaBoard::play_turn(Origin::signed(BOB), 1_u32), Error::<Test>::InvalidTurn);

		// The third game can still be played even though it should be expired by this block
		assert!(!BoardWinners::<Test>::contains_key(board_id_3), "Should contain {}", board_id_3);

		// We 'advance' to block number 21
		run_to_block(21);

		// We force the call to 'on_idle' to trigger the validation of state
		AjunaBoard::on_idle(mock::System::block_number(), 10_000);

		// Here we check how Bob has automatically won because of inactivity
		assert_eq!(
			last_event(),
			mock::Event::AjunaBoard(crate::Event::GameFinished {
				board_id: board_id_3,
				winner: FLORINA
			}),
			"Florina should have won"
		);
		assert_eq!(
			BoardWinners::<Test>::get(board_id_3).unwrap(),
			FLORINA,
			"Board stored to state with winner as Florina"
		);
		assert_noop!(
			AjunaBoard::play_turn(Origin::signed(FLORINA), 1_u32),
			Error::<Test>::InvalidTurn
		);
	});
}
