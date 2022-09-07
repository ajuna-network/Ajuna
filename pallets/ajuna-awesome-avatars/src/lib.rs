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

pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::types::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Hooks, WithdrawReasons},
	};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::{
		traits::{Hash, UniqueSaturatedInto},
		ArithmeticError,
	};
	use sp_std::vec::Vec;

	pub(crate) type SeasonOf<T> = Season<<T as frame_system::Config>::BlockNumber>;
	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type BoundedAvatarIdsOf<T> =
		BoundedVec<AvatarIdOf<T>, ConstU32<MAX_AVATARS_PER_PLAYER>>;
	pub(crate) type MintFeesFor<T> = (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>);

	pub(crate) const MAX_AVATARS_PER_PLAYER: u32 = 1_000;
	pub(crate) const MAX_PERCENTAGE: u8 = 100;

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
	pub fn DefaultMintFee<T: Config>() -> MintFeesFor<T> {
		(
			(1_000_000_000_000_u64 * 55 / 100).unique_saturated_into(),
			(1_000_000_000_000_u64 * 50 / 100).unique_saturated_into(),
			(1_000_000_000_000_u64 * 45 / 100).unique_saturated_into(),
		)
	}

	#[pallet::storage]
	#[pallet::getter(fn mint_fees)]
	pub type MintFee<T: Config> = StorageValue<_, MintFeesFor<T>, ValueQuery, DefaultMintFee<T>>;

	#[pallet::type_value]
	pub fn DefaultMintCooldown<T: Config>() -> T::BlockNumber {
		5_u8.into()
	}

	#[pallet::storage]
	#[pallet::getter(fn mint_cooldown)]
	pub type MintCooldown<T: Config> =
		StorageValue<_, T::BlockNumber, ValueQuery, DefaultMintCooldown<T>>;

	#[pallet::storage]
	#[pallet::getter(fn avatars)]
	pub type Avatars<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, (T::AccountId, Avatar)>;

	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type Owners<T: Config> =
		StorageMap<_, Identity, T::AccountId, BoundedAvatarIdsOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_minted_block_numbers)]
	pub type LastMintedBlockNumbers<T: Config> =
		StorageMap<_, Identity, T::AccountId, T::BlockNumber, OptionQuery>;

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
		UpdatedMintFee { fee: MintFeesFor<T> },
		/// Mint cooldown updated.
		UpdatedMintCooldown { cooldown: T::BlockNumber },
		/// Avatar minted.
		AvatarMinted { avatar_ids: BoundedAvatarIdsOf<T> },
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
		/// The combination of all tiers rarity percentages doesn't add up to 100
		IncorrectRarityPercentages,
		/// Some rarity tier are duplicated.
		DuplicatedRarityTier,
		/// Minting is not available at the moment.
		MintUnavailable,
		/// No season active currently.
		OutOfSeason,
		/// Max ownership reached.
		MaxOwnershipReached,
		/// Incorrect DNA.
		IncorrectDna,
		/// Minting the batch would overflow the max ownership.
		BatchSizeTooBig,
		/// The player must wait cooldown period.
		MintCooldown,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			Organizer::<T>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn new_season(origin: OriginFor<T>, new_season: SeasonOf<T>) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			let new_season = Self::ensure_season(new_season)?;

			let season_id = Self::next_season_id();
			let prev_season_id = season_id.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			if let Some(prev_season) = Self::seasons(prev_season_id) {
				ensure!(prev_season.end < new_season.early_start, Error::<T>::EarlyStartTooEarly);
			}

			Seasons::<T>::insert(season_id, &new_season);
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
			let season = Self::ensure_season(season)?;

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

			SeasonsMetadata::<T>::insert(season_id, &metadata);

			Self::deposit_event(Event::UpdatedSeasonMetadata {
				season_id,
				season_metadata: metadata,
			});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn update_mint_fees(origin: OriginFor<T>, new_fees: MintFeesFor<T>) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			MintFee::<T>::set(new_fees);
			Self::deposit_event(Event::UpdatedMintFee { fee: new_fees });

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

		#[pallet::weight(10_000)]
		pub fn mint(origin: OriginFor<T>, how_many: MintCount) -> DispatchResult {
			let player = ensure_signed(origin)?;
			ensure!(Self::mint_available(), Error::<T>::MintUnavailable);
			let num_of_existing_avatars = Self::owners(&player).len();
			ensure!(
				num_of_existing_avatars < MAX_AVATARS_PER_PLAYER as usize,
				Error::<T>::MaxOwnershipReached
			);
			ensure!(
				(num_of_existing_avatars + how_many as usize) < MAX_AVATARS_PER_PLAYER as usize,
				Error::<T>::BatchSizeTooBig
			);

			let current_block = <frame_system::Pallet<T>>::block_number();
			if let Some(last_block) = Self::last_minted_block_numbers(&player) {
				let cooldown = Self::mint_cooldown();
				ensure!(current_block > last_block + cooldown, Error::<T>::MintCooldown);
			}

			Self::do_mint(&player, how_many)?;

			LastMintedBlockNumbers::<T>::insert(&player, current_block);

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

		pub(crate) fn ensure_season(mut season: SeasonOf<T>) -> Result<SeasonOf<T>, DispatchError> {
			ensure!(season.early_start < season.start, Error::<T>::EarlyStartTooLate);
			ensure!(season.start < season.end, Error::<T>::SeasonStartTooLate);

			season.rarity_tiers.sort_by(|a, b| b.1.cmp(&a.1));
			let (tiers, chances) = {
				let (mut tiers, chances): (Vec<_>, Vec<_>) =
					season.rarity_tiers.clone().into_iter().unzip();
				tiers.sort();
				tiers.dedup();
				(tiers, chances)
			};
			ensure!(
				chances.iter().sum::<RarityPercent>() == MAX_PERCENTAGE,
				Error::<T>::IncorrectRarityPercentages
			);
			ensure!(tiers.len() == chances.len(), Error::<T>::DuplicatedRarityTier);

			Ok(season)
		}

		fn random_number(who: &T::AccountId, until: u8) -> u8 {
			let nonce = frame_system::Pallet::<T>::account_nonce(who);
			let block_number = frame_system::Pallet::<T>::block_number();
			let random_hash = (nonce, who, block_number).using_encoded(sp_io::hashing::twox_128);
			frame_system::Pallet::<T>::inc_account_nonce(who);
			random_hash[0] % until
		}

		fn random_component(who: &T::AccountId, season: &SeasonOf<T>) -> (u8, u8) {
			let random_tier = {
				let random_percent = Self::random_number(who, MAX_PERCENTAGE);
				let mut cumulative_sum = 0;
				let mut random_tier = season.rarity_tiers[0].0.clone() as u8;
				for (tier, chance) in season.rarity_tiers.iter() {
					let new_cumulative_sum = cumulative_sum + chance;
					if random_percent >= cumulative_sum && random_percent < new_cumulative_sum {
						random_tier = tier.clone() as u8;
						break
					}
					cumulative_sum = new_cumulative_sum;
				}
				random_tier
			};
			let random_variation = Self::random_number(who, season.max_variations);
			(random_tier, random_variation)
		}

		pub(crate) fn random_dna(
			who: &T::AccountId,
			season: &SeasonOf<T>,
		) -> Result<Dna, DispatchError> {
			let dna = (0..season.max_components)
				.map(|_| {
					let (random_tier, random_variation) = Self::random_component(who, season);
					((random_tier << 4) | random_variation) as u8
				})
				.collect::<Vec<_>>();
			Dna::try_from(dna).map_err(|_| Error::<T>::IncorrectDna.into())
		}

		pub(crate) fn do_mint(
			player: &T::AccountId,
			how_many: MintCount,
		) -> Result<(), DispatchError> {
			let active_season_id = Self::active_season_id().ok_or(Error::<T>::OutOfSeason)?;
			let active_season = Self::seasons(active_season_id).ok_or(Error::<T>::UnknownSeason)?;

			let mut generated_avatars = Vec::new();
			let mut generated_avatars_ids = Vec::new();

			for _ in 0..how_many as usize {
				let dna = Self::random_dna(player, &active_season)?;
				let avatar = Avatar { season: active_season_id, dna };
				let avatar_id = T::Hashing::hash_of(&avatar);

				generated_avatars.push(avatar.clone());
				generated_avatars_ids.push(avatar_id);
				Avatars::<T>::insert(avatar_id, (player, avatar));
			}

			let mut player_avatars = Owners::<T>::get(&player);
			for avatar in &generated_avatars_ids {
				ensure!(player_avatars.try_push(*avatar).is_ok(), Error::<T>::MaxOwnershipReached);
			}
			Owners::<T>::insert(&player, player_avatars);

			let fee = match how_many {
				MintCount::One => Self::mint_fees().0,
				MintCount::Three => Self::mint_fees().1,
				MintCount::Six => Self::mint_fees().2,
			};

			T::Currency::withdraw(
				player,
				fee,
				WithdrawReasons::FEE,
				ExistenceRequirement::KeepAlive,
			)?;

			Self::deposit_event(Event::AvatarMinted {
				avatar_ids: BoundedVec::try_from(generated_avatars_ids).unwrap(),
			});

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let season_id = Self::active_season_id().unwrap_or_else(Self::next_active_season_id);
			let mut db_weight = T::DbWeight::get().reads(2);

			if let Some(season) = Self::seasons(season_id) {
				db_weight += T::DbWeight::get().reads(1);
				if season.early_start <= now && now <= season.end {
					ActiveSeasonId::<T>::put(season_id);
					NextActiveSeasonId::<T>::put(season_id.saturating_add(1));
					db_weight += T::DbWeight::get().writes(2);
				} else {
					ActiveSeasonId::<T>::kill();
					db_weight += T::DbWeight::get().writes(1);
				}
			}
			db_weight
		}
	}
}
