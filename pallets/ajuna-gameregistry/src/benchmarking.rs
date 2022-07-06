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
use crate::Pallet as GameRegistry;
use ajuna_common::MatchMaker;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn player<T: Config>(index: u32) -> T::AccountId {
	account("player", index, SEED)
}

fn enqueue<T: Config>(player: T::AccountId) {
	T::MatchMaker::enqueue(player, DEFAULT_BRACKET);
}

benchmarks! {
	queue {
		let player = player::<T>(0);
		enqueue::<T>(player.clone());

		let caller = whitelisted_caller::<T::AccountId>();
	}: queue(RawOrigin::Signed(caller.clone()))
	verify {
		let player_game_id = GameRegistry::<T>::players(&player).unwrap();
		let caller_game_id = GameRegistry::<T>::players(&caller).unwrap();
		assert_eq!(player_game_id, caller_game_id);

		let queued = GameRegistry::<T>::queued().expect("the game to be queued");
		assert!(queued.contains(&player_game_id));
		assert!(queued.contains(&caller_game_id));
	}

	impl_benchmark_test_suite!(
		GameRegistry,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	)
}
