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
use crate::{
	dot4gravity::{Coordinates, Side, Turn},
	Pallet as AjunaBoard,
};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

const SEED: u32 = 0;

fn player<Player: Decode>(index: u32) -> Player {
	account("player", index, SEED)
}

fn players<Player: Decode + Ord>(how_many: u32) -> BTreeSet<Player> {
	let mut players = BTreeSet::new();
	for i in 0..how_many {
		players.insert(player::<Player>(i));
	}
	players
}

fn assert_last_event<T: Config>(event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

fn create_new_game<T: Config>(board_id: T::BoardId, players: BTreeSet<T::AccountId>) {
	let player_1 = players.clone().into_iter().next().unwrap();
	let _ = AjunaBoard::<T>::new_game(RawOrigin::Signed(player_1).into(), board_id, players);
}

fn create_and_play_until_win<T: Config>(board_id: T::BoardId, players: BTreeSet<T::AccountId>) {
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
	Seed::<T>::put(7357);
	create_new_game::<T>(board_id, players.clone());

	let mut players = players.into_iter();
	let player_1: T::Origin = RawOrigin::Signed(players.next().unwrap()).into();
	let player_2: T::Origin = RawOrigin::Signed(players.next().unwrap()).into();

	let each_player_drops_bomb = |coord: Coordinates| {
		let drop_bomb: T::PlayersTurn = Turn::DropBomb(coord).into();
		let _ = AjunaBoard::<T>::play_turn(player_1.clone(), drop_bomb.clone());
		let _ = AjunaBoard::<T>::play_turn(player_2.clone(), drop_bomb);
	};
	let each_player_drops_stone = |win_position: (Side, u8), lose_position: (Side, u8)| {
		let win: T::PlayersTurn = Turn::DropStone(win_position).into();
		let lose: T::PlayersTurn = Turn::DropStone(lose_position).into();
		let _ = AjunaBoard::<T>::play_turn(player_1.clone(), win);
		let _ = AjunaBoard::<T>::play_turn(player_2.clone(), lose);
	};

	// Bomb phase
	each_player_drops_bomb(Coordinates::new(9, 9));
	each_player_drops_bomb(Coordinates::new(8, 8));
	each_player_drops_bomb(Coordinates::new(7, 7));

	// Stone phase
	let win_position = (Side::North, 0);
	let lose_position = (Side::North, 9);
	each_player_drops_stone(win_position, lose_position);
	each_player_drops_stone(win_position, lose_position);
	each_player_drops_stone(win_position, lose_position);
}

benchmarks! {
	new_game {
		// NOTE: Only 2-player games are supported at the moment. When we need variable player
		// count, this weight should be parameterised over min & max number of players
		let caller = whitelisted_caller::<T::AccountId>();
		let board_id = T::BoardId::saturated_from(u128::MAX);
		let players = players::<T::AccountId>(T::MaxNumberOfPlayers::get());
	}: new_game(RawOrigin::Signed(caller), board_id, players.clone())
	verify {
		assert!(BoardStates::<T>::contains_key(board_id));
		for player in players.clone() {
			assert!(PlayerBoards::<T>::contains_key(player));
		}

		let players = players.into_iter().collect::<Vec<_>>();
		assert_last_event::<T>(Event::GameCreated { board_id, players }.into());
	}

	play_turn {
		let board_id = T::BoardId::saturated_from(u128::MAX);
		let players = players::<T::AccountId>(T::MaxNumberOfPlayers::get());
		create_new_game::<T>(board_id, players.clone());

		let player_1 = players.into_iter().next().unwrap();
		let turn = Turn::DropBomb(Coordinates::new(1, 2));
	}: play_turn(RawOrigin::Signed(player_1), turn.into())

	play_turn_until_finished {
		let board_id = T::BoardId::saturated_from(u128::MAX);
		let players = players::<T::AccountId>(T::MaxNumberOfPlayers::get());
		create_and_play_until_win::<T>(board_id, players.clone());

		let winner = players.into_iter().next().unwrap();
		let turn = Turn::DropStone((Side::North, 0));
	}: play_turn(RawOrigin::Signed(winner.clone()), turn.into())
	verify {
		assert_last_event::<T>(Event::GameFinished { board_id, winner }.into());
	}

	impl_benchmark_test_suite!(
		AjunaBoard,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	)
}
