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
use ajuna_common::{GetIdentifier, Runner, RunnerState, State};
use frame_support::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::One;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use sp_runtime::traits::AtLeast32BitUnsigned;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type RunnerId: Member + Parameter + MaxEncodedLen + AtLeast32BitUnsigned + Default + Copy;
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
		/// An internal error has occurred
		InternalError,
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
			*nonce += One::one();
			*nonce
		})
	}
}

pub struct Running<T: Config>(PhantomData<T>);
impl<T: Config> Running<T> {
	pub fn update_state(
		identifier: T::RunnerId,
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
	type Identifier = T::RunnerId;
	fn create<G: GetIdentifier<T::RunnerId>>(initial_state: State) -> Option<Self::Identifier> {
		let identifier = G::get_identifier();
		if Runners::<T>::contains_key(identifier) {
			return None
		}

		Runners::<T>::insert(identifier, RunnerState::Queued(initial_state));
		Pallet::<T>::deposit_event(Event::StateQueued { runner_id: identifier });
		Some(identifier)
	}

	fn accept(identifier: Self::Identifier, new_state: Option<State>) -> DispatchResult {
		if let Some(RunnerState::Queued(original_state)) = Self::get_state(identifier) {
			match new_state {
				Some(new_state) => {
					Self::update_state(identifier, RunnerState::Accepted(new_state))
						.map_err(|_| Error::<T>::InternalError)?;

					Pallet::<T>::deposit_event(Event::StateAccepted { runner_id: identifier });
				},
				None => Self::update_state(identifier, RunnerState::Accepted(original_state))
					.map_err(|_| Error::<T>::InternalError)?,
			}
			Ok(())
		} else {
			Err(Error::<T>::InvalidState.into())
		}
	}

	fn finished(identifier: Self::Identifier, final_state: Option<State>) -> DispatchResult {
		if let Some(RunnerState::Accepted(original_state)) = Self::get_state(identifier) {
			match final_state {
				Some(final_state) => {
					Self::update_state(identifier, RunnerState::Finished(final_state))
						.map_err(|_| Error::<T>::InternalError)?;

					Pallet::<T>::deposit_event(Event::StateFinished { runner_id: identifier });
				},
				None => Self::update_state(identifier, RunnerState::Finished(original_state))
					.map_err(|_| Error::<T>::InternalError)?,
			}
			Ok(())
		} else {
			Err(Error::<T>::InvalidState.into())
		}
	}

	fn remove(identifier: Self::Identifier) -> DispatchResult {
		if !Runners::<T>::contains_key(identifier) {
			return Err(Error::<T>::UnknownRunner.into())
		}

		Runners::<T>::remove(identifier);
		Ok(())
	}

	fn get_state(identifier: Self::Identifier) -> Option<RunnerState> {
		Runners::<T>::get(identifier)
	}
}
