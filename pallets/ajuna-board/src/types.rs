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

pub(crate) type PlayerOf<T> = <<T as Config>::Game as TurnBasedGame>::Player;
pub(crate) type BoundedPlayersOf<T> =
	BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::Players>;
pub(crate) type BoardGameOf<T> = BoardGame<
	<T as Config>::BoardId,
	<T as Config>::GameState,
	BoundedPlayersOf<T>,
	BlockNumberFor<T>,
>;

/// The state of the board game
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BoardGame<BoardId, State, Players, Start> {
	board_id: BoardId,
	/// Players in the game
	pub(crate) players: Players,
	/// The current state of the game
	pub state: State,
	/// When the game started
	pub started: Start,
}

impl<BoardId, State, Players, Start> BoardGame<BoardId, State, Players, Start> {
	/// Create a BoardGame
	pub(crate) fn new(board_id: BoardId, players: Players, state: State, started: Start) -> Self {
		Self { board_id, players, state, started }
	}
}

#[derive(Debug, PartialEq)]
pub enum Finished<Player> {
	No,
	Winner(Player),
}

pub trait TurnBasedGame {
	/// Represents a turn in the game
	type Turn;
	/// Represents a player in the game
	type Player: Clone;
	/// The state of the game
	type State: Codec;
	/// Initialise turn based game with players returning the initial state
	fn init(players: &[Self::Player], seed: Option<u32>) -> Option<Self::State>;
	/// Get the player that played its turn last
	fn get_last_player(state: &Self::State) -> Self::Player;
	/// Get the player that should play its turn next
	fn get_next_player(state: &Self::State) -> Self::Player;
	/// Play a turn with player on the current state returning the new state
	fn play_turn(player: Self::Player, state: Self::State, turn: Self::Turn)
		-> Option<Self::State>;
	/// Forces the termination of a game with a designated winner, useful when games
	/// get stalled for some reason.
	fn abort(state: Self::State, winner: Self::Player) -> Self::State;
	/// Check if the game has finished with winner
	fn is_finished(state: &Self::State) -> Finished<Self::Player>;
	/// Get seed if any
	fn seed(state: &Self::State) -> Option<u32>;
}
