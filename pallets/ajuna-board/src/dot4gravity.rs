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

use crate::{Finished, TurnBasedGame};
use dot4gravity::Game as Dot4Gravity;
pub use dot4gravity::{Coordinates, GameState, Side};
use frame_support::{pallet_prelude::*, Parameter};
use sp_std::borrow::ToOwned;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub enum Turn {
	DropBomb(Coordinates),
	DropStone((Side, u8)),
}

pub struct Game<Account>(PhantomData<Account>);
impl<Account> TurnBasedGame for Game<Account>
where
	Account: Parameter,
{
	type Turn = Turn;
	type Player = Account;
	type State = GameState<Account>;

	fn init(players: &[Self::Player], seed: Option<u32>) -> Option<Self::State> {
		if let [player_1, player_2] = players {
			Some(Dot4Gravity::new_game(player_1.to_owned(), player_2.to_owned(), seed))
		} else {
			None
		}
	}

	fn get_last_player(state: &Self::State) -> Self::Player {
		state
			.last_move
			.clone()
			.map(|last_move_of| last_move_of.player)
			.unwrap_or_else(|| state.next_player.clone())
	}

	fn get_next_player(state: &Self::State) -> Self::Player {
		state.next_player.clone()
	}

	fn play_turn(
		player: Self::Player,
		state: Self::State,
		turn: Self::Turn,
	) -> Option<Self::State> {
		match turn {
			Turn::DropBomb(coords) => Dot4Gravity::drop_bomb(state, coords, player),
			Turn::DropStone((side, pos)) => Dot4Gravity::drop_stone(state, player, side, pos),
		}
		.ok()
	}

	fn abort(state: Self::State, winner: Self::Player) -> Self::State {
		let mut state = state;
		state.winner = Some(winner);
		state
	}

	fn is_finished(state: &Self::State) -> Finished<Self::Player> {
		match state.winner.clone() {
			Some(winner) => Finished::Winner(winner),
			None => Finished::No,
		}
	}

	fn seed(state: &Self::State) -> Option<u32> {
		Some(state.seed)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const THE_NUMBER: Guess = 42;
	const MAX_PLAYERS: usize = 2;
	const PLAYER_1: u32 = 1;
	const PLAYER_2: u32 = 2;

	type Guess = u32;
	type Account = u32;

	struct MockGame;

	#[derive(Encode, Decode, Copy, Clone)]
	struct MockGameState {
		pub players: [Account; MAX_PLAYERS],
		pub next_player: u8,
		pub solution: Guess,
		pub winner: Option<Account>,
	}

	impl TurnBasedGame for MockGame {
		type Turn = Guess;
		type Player = Account;
		type State = MockGameState;

		fn init(players: &[Self::Player], _seed: Option<u32>) -> Option<Self::State> {
			match players.to_vec().try_into() {
				Ok(players) => Some(MockGameState {
					players,
					next_player: 0,
					solution: THE_NUMBER,
					winner: None,
				}),
				_ => None,
			}
		}

		fn get_last_player(state: &Self::State) -> Self::Player {
			let next_player_index = (state.next_player as usize + 1) % state.players.len();
			state.players[next_player_index]
		}

		fn get_next_player(state: &Self::State) -> Self::Player {
			state.players[state.next_player as usize]
		}

		fn play_turn(
			player: Self::Player,
			state: Self::State,
			turn: Self::Turn,
		) -> Option<Self::State> {
			if state.winner.is_some() ||
				!state.players.contains(&player) ||
				state.players[state.next_player as usize] != player
			{
				return None
			}

			let mut state = state;
			state.next_player = (state.next_player + 1) % state.players.len() as u8;

			if state.solution == turn {
				state.winner = Some(player);
			}

			Some(state)
		}

		fn abort(state: Self::State, winner: Self::Player) -> Self::State {
			let mut state = state;
			state.winner = Some(winner);
			state
		}

		fn is_finished(state: &Self::State) -> Finished<Self::Player> {
			let winner = &state.winner;
			match winner {
				None => Finished::No,
				Some(winner) => Finished::Winner(*winner),
			}
		}

		fn seed(_state: &Self::State) -> Option<u32> {
			None
		}
	}

	#[test]
	fn guessing_works() {
		let state = MockGame::init(&[PLAYER_1, PLAYER_2], None).unwrap();
		assert_eq!(MockGame::get_next_player(&state), PLAYER_1);

		let state = MockGame::play_turn(PLAYER_1, state, 1).unwrap();
		assert_eq!(MockGame::get_last_player(&state), PLAYER_1);
		assert_eq!(MockGame::get_next_player(&state), PLAYER_2);

		let state = MockGame::play_turn(PLAYER_2, state, THE_NUMBER).unwrap();
		assert_eq!(MockGame::is_finished(&state), Finished::Winner(PLAYER_2));

		// new game
		let state = MockGame::init(&[PLAYER_1, PLAYER_2], None).unwrap();
		let state = MockGame::abort(state, PLAYER_1);
		assert_eq!(MockGame::is_finished(&state), Finished::Winner(PLAYER_1));
	}
}
