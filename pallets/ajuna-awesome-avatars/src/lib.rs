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

use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub mod ajuna_awesome_avatar {

	use super::{Decode, Encode, RuntimeDebug};
	use codec::MaxEncodedLen;
	use scale_info::TypeInfo;

	#[derive(Encode, Decode, Clone, Default, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Season<BlockNumber> {
		pub early_access_start: BlockNumber,
		pub start: BlockNumber,
		pub end: BlockNumber,
		pub max_mints: u16,
		pub max_mythical_mints: u16,
	}

	impl<BlockNumber: PartialOrd> Season<BlockNumber> {
		pub fn new(
			early_access_start: BlockNumber,
			start: BlockNumber,
			end: BlockNumber,
			max_mints: u16,
			max_mythical_mints: u16,
		) -> Self {
			Self { early_access_start, start, end, max_mints, max_mythical_mints }
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use crate::ajuna_awesome_avatar::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::ArithmeticError;

	// type SeasonOf<T> = Season<<T as frame_system::Config>::BlockNumber>;
	type SeasonId = u16;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	/// Season number. Storage value to keep track of the id.
	#[pallet::storage]
	#[pallet::getter(fn next_season_id)]
	pub type NextSeasonId<T: Config> = StorageValue<_, SeasonId, ValueQuery>;

	/// Season id currently active.
	#[pallet::storage]
	#[pallet::getter(fn active_season_id)]
	pub type ActiveSeason<T: Config> = StorageValue<_, SeasonId, ValueQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, Season<BlockNumberFor<T>>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewSeasonCreated(Season<BlockNumberFor<T>>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The season starts before the previous season has ended.
		EarlyAccessStartsTooEarly,
		/// The season season start later than its early access
		EarlyAccessStartsTooLate,
		/// The season start date is newer than its end date.
		SeasonStartsTooLate,
		/// The season ends after the new season has started.
		SeasonEndsTooLate,
		/// The season doesn't exist.
		UnknownSeason,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn new_season(
			origin: OriginFor<T>,
			new_season: Season<BlockNumberFor<T>>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			if new_season.start < new_season.early_access_start {
				return Err(Error::<T>::EarlyAccessStartsTooLate.into())
			}

			if new_season.start > new_season.end {
				return Err(Error::<T>::SeasonStartsTooLate.into())
			}

			let season_id = Self::next_season_id();

			if season_id > 0 {
				let maybe_season = Seasons::<T>::get(season_id - 1);

				if let Some(season) = maybe_season {
					if season.end > new_season.early_access_start {
						return Err(Error::<T>::EarlyAccessStartsTooEarly.into())
					}
				}
			}

			// save season
			Seasons::<T>::insert(season_id, new_season.clone());

			// increase next season id
			match season_id.checked_add(1) {
				Some(number) => NextSeasonId::<T>::put(number),
				None => return Err(ArithmeticError::Overflow.into()),
			};

			Self::deposit_event(Event::NewSeasonCreated(new_season));

			Ok(())
		}
	}
}
