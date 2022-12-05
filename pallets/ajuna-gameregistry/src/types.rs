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

#[derive(Encode, Decode, Default, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Game<AccountId> {
	pub tee_id: Option<AccountId>,
	pub players: Vec<AccountId>,
	pub winner: Option<AccountId>,
}

impl<AccountId> Game<AccountId> {
	pub fn new(players: Vec<AccountId>) -> Self {
		Game { tee_id: None, winner: None, players }
	}
}

#[derive(Encode, Decode, RuntimeDebugNoBound, TypeInfo, Clone, Default, Eq, PartialEq)]
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

#[derive(Encode, Decode, RuntimeDebugNoBound, TypeInfo, Clone, Eq, PartialEq)]
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
	type RunnerId: Identifier;
	/// Create a runner with identifier and initial state
	fn create<G: GetIdentifier<Self::RunnerId>>(initial_state: State) -> Option<Self::RunnerId>;
	/// Accept a runner has been scheduled to run, with an optional new state
	fn accept(identifier: &Self::RunnerId, new_state: Option<State>) -> DispatchResult;
	/// Runner has finished executing, with an optional final state
	fn finished(identifier: &Self::RunnerId, final_state: Option<State>) -> DispatchResult;
	/// Remove a runner
	fn remove(identifier: &Self::RunnerId) -> DispatchResult;
	/// Get state for runner identified by identifier
	fn get_state(identifier: &Self::RunnerId) -> Option<RunnerState>;
}

/// Marker trait used to define a common basis for all potential identifiers used in the pallets
pub trait Identifier:
	Member + Parameter + MaxEncodedLen + AtLeast32BitUnsigned + Default + Copy
{
}

impl Identifier for u32 {}
impl Identifier for u64 {}

/// Provide a unique identifier
pub trait GetIdentifier<T: Identifier> {
	fn get_identifier() -> T;
}
