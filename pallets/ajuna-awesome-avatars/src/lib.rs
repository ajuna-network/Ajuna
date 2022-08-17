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

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new organizer has been set
		OrganizerSet { organizer: T::AccountId },
		/// The previous organizer has been replaced
		OrganizerReplaced { new_organizer: T::AccountId, prev_organizer: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The specified account is not an organizer
		AccountIsNotOrganizer,
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// Account used to perform action doesn't have enough privileges
		InsufficientPrivileges,
	}

	#[pallet::storage]
	#[pallet::getter(fn organizer)]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			if let Err(_) = ensure_root(origin) {
				return Err(Error::<T>::InsufficientPrivileges.into())
			}

			let event = if let Some(prev_organizer) = Organizer::<T>::get() {
				Event::OrganizerReplaced { new_organizer: organizer.clone(), prev_organizer }
			} else {
				Event::OrganizerSet { organizer: organizer.clone() }
			};

			Organizer::<T>::put(organizer.clone());

			Self::deposit_event(event);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn ensure_organizer(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			match Organizer::<T>::get() {
				Some(organizer) => match organizer == account {
					true => Ok(()),
					false => Err(Error::<T>::AccountIsNotOrganizer.into()),
				},
				None => Err(Error::<T>::OrganizerNotSet.into()),
			}
		}
	}
}
