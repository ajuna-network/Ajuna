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
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Hooks},
	};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::{
		traits::{Saturating, UniqueSaturatedInto},
		ArithmeticError,
	};

	pub(crate) type SeasonOf<T> = Season<<T as frame_system::Config>::BlockNumber>;
	pub(crate) type SeasonId = u16;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn organizer)]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultNextSeasonId() -> SeasonId {
		1
	}

	#[pallet::type_value]
	pub fn DefaultNextActiveSeasonId() -> SeasonId {
		1
	}

	/// Season number. Storage value to keep track of the id.
	#[pallet::storage]
	#[pallet::getter(fn next_season_id)]
	pub type NextSeasonId<T: Config> = StorageValue<_, SeasonId, ValueQuery, DefaultNextSeasonId>;

	/// Season id currently active.
	#[pallet::storage]
	#[pallet::getter(fn active_season_id)]
	pub type ActiveSeasonId<T: Config> = StorageValue<_, SeasonId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_active_season_id)]
	pub type NextActiveSeasonId<T: Config> =
		StorageValue<_, SeasonId, ValueQuery, DefaultNextActiveSeasonId>;

	#[pallet::storage]
	#[pallet::getter(fn seasons_metadata)]
	pub type SeasonsMetadata<T: Config> = StorageMap<_, Identity, SeasonId, SeasonMetadata>;

	/// Storage for the seasons.
	#[pallet::storage]
	#[pallet::getter(fn seasons)]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, SeasonOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn mint_available)]
	pub type MintAvailable<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultMintFee<T: Config>() -> BalanceOf<T> {
		(1_000_000_000_000_u64 * 55 / 100).unique_saturated_into()
	}

	#[pallet::storage]
	#[pallet::getter(fn mint_fee)]
	pub type MintFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery, DefaultMintFee<T>>;

	#[pallet::type_value]
	pub fn DefaultMintCooldown<T: Config>() -> T::BlockNumber {
		5_u8.into()
	}

	#[pallet::storage]
	#[pallet::getter(fn mint_cooldown)]
	pub type MintCooldown<T: Config> =
		StorageValue<_, T::BlockNumber, ValueQuery, DefaultMintCooldown<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new organizer has been set.
		OrganizerSet { organizer: T::AccountId },
		/// A new season has been created.
		NewSeasonCreated(SeasonOf<T>),
		/// An existing season has been updated.
		SeasonUpdated(SeasonOf<T>, SeasonId),
		/// The metadata for {season_id} has been updated
		UpdatedSeasonMetadata { season_id: SeasonId, season_metadata: SeasonMetadata },
		/// Mint availability updated.
		UpdatedMintAvailability { availability: bool },
		/// Mint fee updated.
		UpdatedMintFee { fee: BalanceOf<T> },
		/// Mint cooldown updated.
		UpdatedMintCooldown { cooldown: T::BlockNumber },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no account set as the organizer
		OrganizerNotSet,
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
		/// The combination of all tiers rarity chances doesn't add up to 100
		IncorrectRarityChances,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Organizer::<T>::put(organizer.clone());
			Self::deposit_event(Event::OrganizerSet { organizer });

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn new_season(origin: OriginFor<T>, new_season: SeasonOf<T>) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			ensure!(new_season.early_start < new_season.start, Error::<T>::EarlyStartTooLate);
			ensure!(new_season.start < new_season.end, Error::<T>::SeasonStartTooLate);
			ensure!(
				new_season.rarity_tiers.values().sum::<RarityChance>() == 100,
				Error::<T>::IncorrectRarityChances
			);

			let season_id = Self::next_season_id();
			let prev_season_id = season_id.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			if let Some(prev_season) = Self::seasons(prev_season_id) {
				ensure!(prev_season.end < new_season.early_start, Error::<T>::EarlyStartTooEarly);
			}

			Seasons::<T>::insert(season_id, new_season.clone());
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
			Self::ensure_organizer(origin)?;

			ensure!(season.early_start < season.start, Error::<T>::EarlyStartTooLate);
			ensure!(season.start < season.end, Error::<T>::SeasonStartTooLate);
			ensure!(
				season.rarity_tiers.values().sum::<RarityChance>() == 100,
				Error::<T>::IncorrectRarityChances
			);

			let prev_season_id = season_id.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			Seasons::<T>::try_mutate(season_id, |maybe_season| {
				if let Some(prev_season) = Self::seasons(prev_season_id) {
					ensure!(prev_season.end < season.early_start, Error::<T>::EarlyStartTooEarly);
				}
				if let Some(next_season) = Self::seasons(next_season_id) {
					ensure!(season.end < next_season.early_start, Error::<T>::SeasonEndTooLate);
				}
				let existing_season = maybe_season.as_mut().ok_or(Error::<T>::UnknownSeason)?;
				*existing_season = season.clone();
				Self::deposit_event(Event::SeasonUpdated(season, season_id));
				Ok(())
			})
		}

		#[pallet::weight(10_000)]
		pub fn update_season_metadata(
			origin: OriginFor<T>,
			season_id: SeasonId,
			metadata: SeasonMetadata,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			ensure!(Self::seasons(season_id).is_some(), Error::<T>::UnknownSeason);

			SeasonsMetadata::<T>::insert(season_id, metadata.clone());

			Self::deposit_event(Event::UpdatedSeasonMetadata {
				season_id,
				season_metadata: metadata,
			});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn update_mint_fee(origin: OriginFor<T>, new_fee: BalanceOf<T>) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			MintFee::<T>::set(new_fee);
			Self::deposit_event(Event::UpdatedMintFee { fee: new_fee });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn update_mint_cooldown(
			origin: OriginFor<T>,
			new_cooldown: T::BlockNumber,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			MintCooldown::<T>::set(new_cooldown);
			Self::deposit_event(Event::UpdatedMintCooldown { cooldown: new_cooldown });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn update_mint_available(origin: OriginFor<T>, availability: bool) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			MintAvailable::<T>::set(availability);
			Self::deposit_event(Event::UpdatedMintAvailability { availability });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn ensure_organizer(origin: OriginFor<T>) -> DispatchResult {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			let active_season_id = Self::active_season_id();
			let next_season_id = Self::next_active_season_id();
			let season_id = active_season_id.unwrap_or(next_season_id);
			let maybe_season = Self::seasons(season_id);

			if let Some(season) = maybe_season {
				if season.early_start <= block_number && block_number <= season.end {
					ActiveSeasonId::<T>::put(season_id);
					if block_number >= season.end {
						NextActiveSeasonId::<T>::mutate(|season_id| season_id.saturating_inc());
					}
				} else {
					ActiveSeasonId::<T>::kill();
				}
			}

			// Register the Weight used on_finalize.
			// 	- One storage read to get the block_weight.
			// 	- One storage read to get the Elasticity.
			// 	- One write to BaseFeePerGas.
			let db_weight = <T as frame_system::Config>::DbWeight::get();
			db_weight.reads(2).saturating_add(db_weight.write)
		}
	}
}
