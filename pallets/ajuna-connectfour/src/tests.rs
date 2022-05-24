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

use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

#[test]
fn test_game_creation() {
	new_test_ext().execute_with(|| {
		// Test player can not play against himself
		assert_noop!(ConnectFour::new_game(Origin::signed(1), 1, 1), Error::<Test>::NoFakePlay);

		// Test game creation between to different players
		assert_ok!(ConnectFour::new_game(Origin::signed(1), 1, 2));
		run_to_block(1);

		let board_id_1 = ConnectFour::player_board(1);
		let board_id_2 = ConnectFour::player_board(2);

		assert_eq!(board_id_1, board_id_2);

		assert_noop!(
			ConnectFour::new_game(Origin::signed(1), 1, 3),
			Error::<Test>::PlayerBoardExists
		);

		assert_noop!(
			ConnectFour::new_game(Origin::signed(3), 3, 2),
			Error::<Test>::PlayerBoardExists
		);

		let board = ConnectFour::boards(board_id_1).expect("board should exist");

		assert_eq!(board.last_turn, 0);
	});
}

#[test]
fn test_game_play() {
	new_test_ext().execute_with(|| {
		let mut current_block: u64 = 100;

		// start from block 100
		run_to_block(current_block);

		// Test game creation between to different players
		assert_ok!(ConnectFour::new_game(
			Origin::signed(PLAYER_1 as u64),
			PLAYER_1 as u64,
			PLAYER_2 as u64
		));
		let board_id = ConnectFour::player_board(PLAYER_1 as u64);
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert_eq!(board.last_turn, current_block);

		run_next_block();
		current_block += 1;

		assert_eq!(System::block_number(), current_block);

		if board.next_player == PLAYER_1 {
			assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 0));
			let board = ConnectFour::boards(board_id).expect("board should exist");
			assert!(board.board_state == BoardState::Running);
			assert!(board.next_player == PLAYER_2);
			assert_eq!(board.last_turn, current_block);

			run_next_block();
			current_block += 1;
		}

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert_eq!(board.last_turn, current_block);
		assert!(board.board_state == BoardState::Running);
		assert!(board.next_player == PLAYER_1);

		run_next_block();
		current_block += 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 2));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block += 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block += 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 3));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block += 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block += 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 4));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block += 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert!(board.board_state == BoardState::Finished(board.blue));
		assert_eq!(board.last_turn, current_block);
	});
}

#[test]
fn test_force_turn() {
	new_test_ext().execute_with(|| {
		let mut current_block: u64 = 100;

		// start from block 100
		run_to_block(current_block);

		// Test game creation between to different players
		assert_ok!(ConnectFour::new_game(
			Origin::signed(PLAYER_1 as u64),
			PLAYER_1 as u64,
			PLAYER_2 as u64
		));
		let board_id = ConnectFour::player_board(PLAYER_1 as u64);
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert_eq!(board.last_turn, current_block);

		run_next_block();
		current_block += 1;

		assert_eq!(System::block_number(), current_block);

		if board.next_player == PLAYER_1 {
			assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 0));
			let board = ConnectFour::boards(board_id).expect("board should exist");
			assert!(board.board_state == BoardState::Running);
			assert!(board.next_player == PLAYER_2);
			assert_eq!(board.last_turn, current_block);

			run_next_block();
			current_block += 1;
		}

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert_eq!(board.last_turn, current_block);
		assert!(board.board_state == BoardState::Running);
		assert!(board.next_player == PLAYER_1);

		run_to_block(current_block + 10);
		current_block += 10;

		// check if force turn ended the game
		let board = ConnectFour::boards(board_id).expect("board should exist");
		assert_eq!(board.last_turn, current_block);
		assert!(board.board_state == BoardState::Finished(board.blue));

		assert!(Boards::<Test>::contains_key(board_id));
		assert!(PlayerBoard::<Test>::contains_key(board.red));
		assert!(PlayerBoard::<Test>::contains_key(board.blue));
		assert!(BoardSchedules::<Test>::contains_key(board_id));

		run_to_block(current_block + 20);
		let _current_block = current_block + 20;

		// check if boards are cleaned up
		assert!(!Boards::<Test>::contains_key(board_id));
		assert!(!PlayerBoard::<Test>::contains_key(board.red));
		assert!(!PlayerBoard::<Test>::contains_key(board.blue));
		assert!(!BoardSchedules::<Test>::contains_key(board_id));
	});
}
