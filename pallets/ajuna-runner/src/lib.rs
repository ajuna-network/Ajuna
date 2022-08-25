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
use ajuna_common::{GetIdentifier, Identifier, Runner, RunnerState, State};
use frame_support::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::Saturating;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type RunnerId: Identifier;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores a map of the runners with its configuration and state
	#[pallet::storage]
	#[pallet::getter(fn runners)]
	pub type Runners<T: Config> = StorageMap<_, Blake2_128, T::RunnerId, RunnerState, OptionQuery>;

	/// A nonce
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, T::RunnerId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Runner Queued
		StateQueued { runner_id: T::RunnerId },
		/// Runner Accepted
		StateAccepted { runner_id: T::RunnerId },
		/// Runner Finished
		StateFinished { runner_id: T::RunnerId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A errorneous state transition
		InvalidState,
		/// Indentifier used for runner is unknown
		UnknownRunner,
	}
}

pub struct AjunaIdentifier<T: Config>(PhantomData<T>);
impl<T: Config> GetIdentifier<T::RunnerId> for AjunaIdentifier<T> {
	fn get_identifier() -> T::RunnerId {
		Nonce::<T>::mutate(|nonce| {
			nonce.saturating_inc();
			*nonce
		})
	}
}

pub struct Running<T: Config>(PhantomData<T>);
impl<T: Config> Running<T> {
	pub fn update_state(
		identifier: &T::RunnerId,
		new_state: RunnerState,
	) -> Result<(), &'static str> {
		// Locate the runner and update state
		Runners::<T>::mutate(identifier, |maybe_runner| {
			if let Some(state) = maybe_runner {
				*state = new_state.clone();
				Ok(())
			} else {
				Err("mutating storage failed!")
			}
		})
	}
}

impl<T: Config> Runner for Running<T> {
	type RunnerId = T::RunnerId;
	fn create<G: GetIdentifier<T::RunnerId>>(initial_state: State) -> Option<Self::RunnerId> {
		let identifier = G::get_identifier();
		if Runners::<T>::contains_key(&identifier) {
			return None
		}

		Runners::<T>::insert(&identifier, RunnerState::Queued(initial_state));
		Pallet::<T>::deposit_event(Event::StateQueued { runner_id: identifier });
		Some(identifier)
	}

	// Review comment: `accept` and `finished` were re-written to accommodate the following:
	//   * Return `UnknownRunner` instead of `InvalidState` when the runner is not found,
	//   * Always emit `StateAccepted` and `StatedFinished` events, not only when the
	//     `new_state`/`final_state` is not-None,
	//   * Eliminated `InternalError` completely (it would never be raised).

	fn accept(identifier: &Self::RunnerId, new_state: Option<State>) -> DispatchResult {
		Runners::<T>::try_mutate(identifier, |state| {
			let state = match state {
				Some(state) => state,
				None => return Err(Error::<T>::UnknownRunner),
			};
			let original_state = match state {
				RunnerState::Queued(state) => state,
				_ => return Err(Error::<T>::InvalidState),
			};
			let new_state = new_state.unwrap_or(original_state.clone());

			*state = RunnerState::Accepted(new_state);
			Pallet::<T>::deposit_event(Event::StateAccepted { runner_id: *identifier });
			Ok(())
		})?;
		Ok(())
	}

	fn finished(identifier: &Self::RunnerId, final_state: Option<State>) -> DispatchResult {
		Runners::<T>::try_mutate(identifier, |state| {
			let state = match state {
				Some(state) => state,
				None => return Err(Error::<T>::UnknownRunner),
			};
			let original_state = match state {
				RunnerState::Accepted(state) => state,
				_ => return Err(Error::<T>::InvalidState),
			};
			let final_state = final_state.unwrap_or(original_state.clone());

			*state = RunnerState::Finished(final_state);
			Pallet::<T>::deposit_event(Event::StateFinished { runner_id: *identifier });
			Ok(())
		})?;
		Ok(())
	}

	fn remove(identifier: &Self::RunnerId) -> DispatchResult {
		if !Runners::<T>::contains_key(identifier) {
			return Err(Error::<T>::UnknownRunner.into())
		}

		Runners::<T>::remove(identifier);
		Ok(())
	}

	fn get_state(identifier: &Self::RunnerId) -> Option<RunnerState> {
		Runners::<T>::get(identifier)
	}
}
