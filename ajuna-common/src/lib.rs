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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod mocks;

use codec::{Codec, Decode, Encode, Input, MaxEncodedLen};
use frame_support::{dispatch::DispatchResult, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
/// Type to identify a bracket
pub type Bracket = u32;
/// Type to counter items in a bracket
pub type BracketCounter = u32;
/// A matchmaker trait which groups players as accounts in brackets
pub trait MatchMaker {
	/// The identifier for player
	type Player;
	/// Enqueue account in bracket
	fn enqueue(account_id: Self::Player, bracket: Bracket) -> bool;
	/// Clear queue for bracket
	fn clear_queue(bracket: Bracket);
	/// Is account queued already?
	fn is_queued(account_id: &Self::Player) -> bool;
	/// Those that are queued in bracket
	fn queued_players(bracket: Bracket) -> Vec<Self::Player>;
	/// Try to get a match using bracket specifying groups size.  A number of players upto
	/// `number_required` will be returned based on availability in the bracket.
	/// The players would be removed from the bracket and queue
	fn try_match(bracket: Bracket, number_required: u8) -> Option<Vec<Self::Player>>;
}

/// Provide a unique identifier
pub trait GetIdentifier<T> {
	fn get_identifier() -> T;
}

#[derive(Clone, Eq, Default, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
pub struct State(Vec<u8>);

impl From<Vec<u8>> for State {
	fn from(v: Vec<u8>) -> Self {
		Self(v)
	}
}

impl From<State> for Vec<u8> {
	fn from(s: State) -> Self {
		s.0
	}
}

impl Input for State {
	fn remaining_len(&mut self) -> Result<Option<usize>, codec::Error> {
		Ok(Some(self.0.len()))
	}

	fn read(&mut self, into: &mut [u8]) -> Result<(), codec::Error> {
		into.clone_from_slice(self.0.drain(0..into.len()).as_slice());
		Ok(())
	}
}

const MAX_STATE_LENGTH: usize = 1024;
impl MaxEncodedLen for State {
	fn max_encoded_len() -> usize {
		MAX_STATE_LENGTH
	}
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub enum RunnerState {
	Queued(State),
	Accepted(State),
	Finished(State),
}

/// A Runner is something we want to run offchain
/// It is identified by a unique identifier which is used in its creation
/// The Runner passes through the states `RunnerState` in which an optional
/// internal state is stored
pub trait Runner {
	type Identifier;
	/// Create a runner with identifier and initial state
	fn create<G: GetIdentifier<Self::Identifier>>(initial_state: State)
		-> Option<Self::Identifier>;
	/// Accept a runner has been scheduled to run, with an optional new state
	fn accept(identifier: Self::Identifier, new_state: Option<State>) -> DispatchResult;
	/// Runner has finished executing, with an optional final state
	fn finished(identifier: Self::Identifier, final_state: Option<State>) -> DispatchResult;
	/// Remove a runner
	fn remove(identifier: Self::Identifier) -> DispatchResult;
	/// Get state for runner identified by identifier
	fn get_state(identifier: Self::Identifier) -> Option<RunnerState>;
}

pub const DEFAULT_BRACKET: Bracket = 0;
pub const DEFAULT_PLAYERS: u8 = 2;

pub enum Finished<Player> {
	No,
	Winner(Player),
	Draw,
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
	/// Play a turn with player on the current state returning the new state
	fn play_turn(player: Self::Player, state: Self::State, turn: Self::Turn)
		-> Option<Self::State>;
	/// Check if the game has finished with winner
	fn is_finished(state: &Self::State) -> Finished<Self::Player>;
	/// Get seed if any
	fn seed(state: &Self::State) -> Option<u32>;
}
