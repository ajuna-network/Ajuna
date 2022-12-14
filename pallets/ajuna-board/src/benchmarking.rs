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
use frame_benchmarking::{account, benchmarks};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;

const SEED: u32 = 0;

fn players<Player: Decode + Ord>(how_many: u32) -> Vec<Player> {
	(0..how_many).into_iter().map(|i| account("player", i, SEED)).collect()
}

fn assert_last_event<T: Config>(event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

fn create_new_game<T: Config>(players: Vec<T::AccountId>) {
	assert_ok!(AjunaBoard::<T>::queue(RawOrigin::Signed(players[0].clone()).into()));
	assert_ok!(AjunaBoard::<T>::queue(RawOrigin::Signed(players[1].clone()).into()));
}

fn create_and_play_until_win<T: Config>(players: Vec<T::AccountId>) {
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
	create_new_game::<T>(players.clone());

	let mut players = players.into_iter();
	let player_1: T::RuntimeOrigin = RawOrigin::Signed(players.next().unwrap()).into();
	let player_2: T::RuntimeOrigin = RawOrigin::Signed(players.next().unwrap()).into();

	let each_player_drops_bomb = |coord: Coordinates| {
		let drop_bomb: T::PlayersTurn = Turn::DropBomb(coord).into();
		let _ = AjunaBoard::<T>::play(player_1.clone(), drop_bomb.clone());
		let _ = AjunaBoard::<T>::play(player_2.clone(), drop_bomb);
	};
	let each_player_drops_stone = |win_position: (Side, u8), lose_position: (Side, u8)| {
		let win: T::PlayersTurn = Turn::DropStone(win_position).into();
		let lose: T::PlayersTurn = Turn::DropStone(lose_position).into();
		let _ = AjunaBoard::<T>::play(player_1.clone(), win);
		let _ = AjunaBoard::<T>::play(player_2.clone(), lose);
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
	play {
		let players = players::<T::AccountId>(T::Players::get());
		create_new_game::<T>(players.clone());

		let player_1 = players.into_iter().next().unwrap();
		let turn = Turn::DropBomb(Coordinates::new(1, 2));
	}: play(RawOrigin::Signed(player_1), turn.into())

	play_turn_until_finished {
		let board_id = T::BoardId::saturated_from(0_u32);
		let players = players::<T::AccountId>(T::Players::get());
		create_and_play_until_win::<T>( players.clone());

		let winner = players.into_iter().next().unwrap();
		let turn = Turn::DropStone((Side::North, 0));
	}: play(RawOrigin::Signed(winner.clone()), turn.into())
	verify {
		assert_last_event::<T>(Event::GameFinished { board_id, winner }.into());
	}

	impl_benchmark_test_suite!(
		AjunaBoard,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	)
}
