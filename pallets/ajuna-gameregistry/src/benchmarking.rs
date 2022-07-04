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
use ajuna_common::{MatchMaker, Runner, RunnerState};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_core::H256;
use sp_runtime::traits::TrailingZeroInput;
use sp_std::vec;

const SEED: u32 = 0;

fn player<T: Config>(index: u32) -> T::AccountId {
	account("player", index, SEED)
}

fn enqueue<T: Config>(player: T::AccountId) {
	T::MatchMaker::enqueue(player, DEFAULT_BRACKET);
}

fn runner_create<T: Config>(players: Vec<T::AccountId>) -> T::GameId {
	let game = Game::new(players);
	T::Runner::create::<T::GetIdentifier>(game.encode().into()).unwrap()
}

fn runner_accept<T: Config>(game_id: &T::GameId) {
	let game_state = T::Runner::get_state(game_id).and_then(|runner_state| match runner_state {
		RunnerState::Queued(state) => Some(state),
		_ => None,
	});
	let _ = T::Runner::accept(game_id, game_state);
}

fn shard_id<ShardIdentifier: Decode>() -> ShardIdentifier {
	// NOTE: H256::random() isn't available for the wasm32-unknown-unknown target
	// See https://docs.rs/getrandom/#webassembly-support
	let hash = H256::from_slice(&[1u8; 32]);
	let mut input = TrailingZeroInput::new(hash.as_bytes());
	Decode::decode(&mut input).unwrap()
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

	drop_game {
		let players = vec![player::<T>(0), player::<T>(1)];
		let game_id = runner_create::<T>(players);
		let caller = whitelisted_caller::<T::AccountId>();
	}: drop_game(RawOrigin::Signed(caller.clone()), game_id)
	verify {
		assert!(T::Runner::get_state(&game_id).is_none());
	}

	ack_game {
		let max_ack_batch = T::MaxAcknowledgeBatch::get();
		let game_ids = (0..max_ack_batch)
			.into_iter()
			.map(
				|i| {
					let players = vec![player::<T>(i * 2), player::<T>(i * 2 + 1)];
					runner_create::<T>(players)
				}
			)
			.collect::<Vec<_>>();
		let caller = whitelisted_caller::<T::AccountId>();
		let shard_id = shard_id::<T::ShardIdentifier>();
	}: ack_game(RawOrigin::Signed(caller), game_ids.clone(), shard_id)
	verify {
		assert!(GameRegistry::<T>::queued().is_none());
		for game_id in game_ids {
			let state = T::Runner::get_state(&game_id);
			assert!(matches!(state, Some(RunnerState::Accepted(_))));
		}
	}

	finish_game {
		let players = vec![player::<T>(0), player::<T>(1)];
		let game_id = runner_create::<T>(players.clone());
		runner_accept::<T>(&game_id);

		let caller = whitelisted_caller::<T::AccountId>();
		let winner = players[1].clone();
		let shard_id = shard_id::<T::ShardIdentifier>();
	}: finish_game(RawOrigin::Signed(caller), game_id, winner, shard_id)
	verify {
		let state = T::Runner::get_state(&game_id);
		assert!(matches!(state, Some(RunnerState::Finished(_))));
	}

	impl_benchmark_test_suite!(
		GameRegistry,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	)
}
