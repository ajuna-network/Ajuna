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
use crate::Pallet as AjunaBoard;
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

	impl_benchmark_test_suite!(
		AjunaBoard,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	)
}
