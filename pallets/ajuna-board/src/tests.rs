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
const ERIN: u32 = 5;

const BOARD_ID: u32 = 0;
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
fn queue_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(AjunaBoard::queue(RuntimeOrigin::signed(ALICE)));
		System::assert_last_event(RuntimeEvent::AjunaMatchmaker(
			pallet_ajuna_matchmaker::Event::Queued(ALICE),
		));
		assert_noop!(AjunaBoard::queue(RuntimeOrigin::signed(ALICE)), Error::<Test>::AlreadyQueued);
	});
}

#[test]
fn queue_creates_game_on_successful_match() {
	new_test_ext().execute_with(|| {
		// nothing persisted before matchmaking
		assert!(PlayerBoards::<Test>::get(ALICE).is_none());
		assert!(PlayerBoards::<Test>::get(BOB).is_none());
		assert!(BoardGames::<Test>::get(BOARD_ID).is_none());
		assert_eq!(NextBoardId::<Test>::get(), BOARD_ID);

		// queue twice to matchmake
		assert_ok!(AjunaBoard::queue(RuntimeOrigin::signed(ALICE)));
		assert_ok!(AjunaBoard::queue(RuntimeOrigin::signed(BOB)));

		let players = vec![ALICE, BOB];
		System::assert_has_event(RuntimeEvent::AjunaMatchmaker(
			pallet_ajuna_matchmaker::Event::Matched(players.clone()),
		));
		System::assert_last_event(RuntimeEvent::AjunaBoard(crate::Event::GameCreated {
			board_id: BOARD_ID,
			players,
		}));

		assert!(PlayerBoards::<Test>::get(ALICE).is_some());
		assert!(PlayerBoards::<Test>::get(BOB).is_some());
		assert!(BoardGames::<Test>::get(BOARD_ID).is_some());
		assert_eq!(NextBoardId::<Test>::get(), BOARD_ID + 1);
	});
}

#[test]
fn play_works() {
	new_test_ext().execute_with(|| {
		Seed::<Test>::put(TEST_SEED);
		assert_ok!(AjunaBoard::queue(RuntimeOrigin::signed(BOB)));
		assert_ok!(AjunaBoard::queue(RuntimeOrigin::signed(ERIN)));
		assert_noop!(
			AjunaBoard::play(RuntimeOrigin::signed(ALICE), Turn::DropBomb(TEST_COORD)),
			Error::<Test>::NotPlaying
		);

		// Bomb phase
		let drop_bomb = |coord: Coordinates| {
			let _ = AjunaBoard::play(RuntimeOrigin::signed(BOB), Turn::DropBomb(coord));
			let _ = AjunaBoard::play(RuntimeOrigin::signed(ERIN), Turn::DropBomb(coord));
		};
		drop_bomb(Coordinates::new(9, 9));
		drop_bomb(Coordinates::new(8, 8));
		drop_bomb(Coordinates::new(7, 7));

		// Play phase
		let drop_stone = || {
			let win = (Side::North, 0);
			let loss = (Side::North, 9);
			let _ = AjunaBoard::play(RuntimeOrigin::signed(BOB), Turn::DropStone(win));
			let _ = AjunaBoard::play(RuntimeOrigin::signed(ERIN), Turn::DropStone(loss));
		};
		drop_stone();
		drop_stone();
		drop_stone();
		drop_stone();

		// check if game has finished
		System::assert_last_event(RuntimeEvent::AjunaBoard(crate::Event::GameFinished {
			board_id: BOARD_ID,
			winner: BOB,
		}));
		assert_ne!(Seed::<Test>::get(), Some(TEST_SEED));
		assert!(PlayerBoards::<Test>::get(ALICE).is_none());
		assert!(PlayerBoards::<Test>::get(BOB).is_none());
		assert!(BoardGames::<Test>::get(BOARD_ID).is_some());

		// We clear the board
		assert_ok!(AjunaBoard::clear_board(RuntimeOrigin::root(), BOARD_ID));

		assert!(BoardGames::<Test>::get(BOARD_ID).is_none());
	})
}
