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

pub mod season;

#[frame_support::pallet]
pub mod pallet {
	use super::season::*;
	use frame_support::pallet_prelude::{DispatchResult, *};
	use frame_system::pallet_prelude::{OriginFor, *};
	use sp_runtime::ArithmeticError;

	type SeasonOf<T> = Season<<T as frame_system::Config>::BlockNumber>;
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
	#[pallet::getter(fn seasons)]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, SeasonOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewSeasonCreated(SeasonOf<T>),
		SeasonUpdated(SeasonOf<T>, SeasonId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The season starts before the previous season has ended.
		EarlyStartTooEarly,
		/// The season season start later than its early access
		EarlyStartTooLate,
		/// The season start date is newer than its end date.
		SeasonStartTooLate,
		/// The season ends after the new season has started.
		SeasonEndTooLate,
		/// The season doesn't exist.
		UnknownSeason,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn new_season(origin: OriginFor<T>, new_season: SeasonOf<T>) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(new_season.early_start < new_season.start, Error::<T>::EarlyStartTooLate);
			ensure!(new_season.start < new_season.end, Error::<T>::SeasonStartTooLate);

			let season_id = Self::next_season_id();
			if season_id > 0 {
				if let Some(prev_season) = Self::seasons(season_id - 1) {
					ensure!(
						prev_season.end < new_season.early_start,
						Error::<T>::EarlyStartTooEarly
					);
				}
			}

			Seasons::<T>::insert(season_id, new_season.clone());

			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
			NextSeasonId::<T>::put(next_season_id);

			Self::deposit_event(Event::NewSeasonCreated(new_season));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_season(
			origin: OriginFor<T>,
			season_id: SeasonId,
			season: SeasonOf<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			ensure!(season.early_start < season.start, Error::<T>::EarlyStartTooLate);
			ensure!(season.start < season.end, Error::<T>::SeasonStartTooLate);

			if Seasons::<T>::get(season_id).is_none() {
				return Err(Error::<T>::UnknownSeason.into())
			}

			let mut maybe_previous_season: Option<
				Season<<T as frame_system::Config>::BlockNumber>,
			> = None;

			if season_id > 0 {
				maybe_previous_season = Seasons::<T>::get(season_id - 1);
			}

			let maybe_next_season = Seasons::<T>::get(season_id + 1);

			enum UpdateError {
				OverlappedWithPreviousSeason,
				OverlappedWithNextSeason,
				NotFound,
			}

			let mutate_result = Seasons::<T>::try_mutate(season_id, |maybe_season| {
				if let Some(existing_season) = maybe_season {
					if let Some(previous_season) = maybe_previous_season {
						if Season::are_seasons_overlapped(&previous_season, &season) {
							return Err(UpdateError::OverlappedWithPreviousSeason)
						}
					}

					if let Some(next_season) = maybe_next_season {
						if Season::are_seasons_overlapped(&season, &next_season) {
							return Err(UpdateError::OverlappedWithNextSeason)
						}
					}

					existing_season.end = season.end;
					existing_season.start = season.start;
					existing_season.early_start = season.early_start;
					existing_season.max_mints = season.max_mints;
					existing_season.max_mythical_mints = season.max_mythical_mints;
					Ok(())
				} else {
					Err(UpdateError::NotFound)
				}
			});

			match mutate_result {
				Err(UpdateError::OverlappedWithPreviousSeason) =>
					return Err(Error::<T>::EarlyStartTooEarly.into()),
				Err(UpdateError::OverlappedWithNextSeason) =>
					return Err(Error::<T>::SeasonEndTooLate.into()),
				Err(UpdateError::NotFound) => return Err(Error::<T>::UnknownSeason.into()),
				Ok(_) => {},
			}

			Self::deposit_event(Event::SeasonUpdated(season, season_id));

			Ok(())
		}
	}
}
