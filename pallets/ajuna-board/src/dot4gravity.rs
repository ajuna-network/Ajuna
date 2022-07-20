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

use ajuna_common::{Finished, TurnBasedGame};
use dot4gravity::Game as Dot4Gravity;
pub use dot4gravity::{Coordinates, GameState, Side};
use frame_support::pallet_prelude::*;
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
