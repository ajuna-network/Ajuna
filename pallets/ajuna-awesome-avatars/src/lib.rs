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
	use crate::types::RarityTier::Mythical;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, Hooks, Randomness, WithdrawReasons},
	};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::{
		traits::{Hash, Saturating, TrailingZeroInput, UniqueSaturatedInto},
		ArithmeticError,
	};
	use sp_std::vec::Vec;

	pub(crate) type SeasonOf<T> = Season<<T as frame_system::Config>::BlockNumber>;
	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type BoundedAvatarIdsOf<T> =
		BoundedVec<AvatarIdOf<T>, ConstU32<MAX_AVATARS_PER_PLAYER>>;
	pub(crate) type GlobalConfigOf<T> =
		GlobalConfig<BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

	pub(crate) const MAX_AVATARS_PER_PLAYER: u32 = 1_000;
	pub(crate) const MAX_PERCENTAGE: u8 = 100;
	pub(crate) const MAX_RANDOM_BYTES: u8 = 32;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::storage]
	#[pallet::getter(fn organizer)]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultNextSeasonId() -> SeasonId {
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

	#[pallet::type_value]
	pub fn DefaultNextActiveSeasonId() -> SeasonId {
		1
	}
	#[pallet::storage]
	#[pallet::getter(fn active_season_rare_mints)]
	pub type ActiveSeasonRareMints<T: Config> = StorageValue<_, MintCount, ValueQuery>;

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

	#[pallet::type_value]
	pub fn DefaultGlobalConfig<T: Config>() -> GlobalConfigOf<T> {
		GlobalConfig {
			mint_available: false,
			mint_fees: MintFees {
				one: (1_000_000_000_000_u64 * 55 / 100).unique_saturated_into(),
				three: (1_000_000_000_000_u64 * 50 / 100).unique_saturated_into(),
				six: (1_000_000_000_000_u64 * 45 / 100).unique_saturated_into(),
			},
			mint_cooldown: 5_u8.into(),
		}
	}
	#[pallet::storage]
	#[pallet::getter(fn global_configs)]
	pub type GlobalConfigs<T: Config> =
		StorageValue<_, GlobalConfigOf<T>, ValueQuery, DefaultGlobalConfig<T>>;

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

	#[pallet::storage]
	#[pallet::getter(fn free_mints)]
	pub type FreeMints<T: Config> = StorageMap<_, Identity, T::AccountId, MintCount, ValueQuery>;

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
		/// Global configuration updated.
		UpdatedGlobalConfig(GlobalConfigOf<T>),
		/// Avatars minted.
		AvatarsMinted { avatar_ids: Vec<AvatarIdOf<T>> },
		/// Rare avatars minted.
		RareAvatarsMinted { count: MintCount },
		/// A season has started.
		SeasonStarted(SeasonId),
		/// A season has finished.
		SeasonFinished(SeasonId),
		/// Free mints transfered between accounts.
		FreeMintsTransferred { from: T::AccountId, to: T::AccountId, how_many: MintCount },
		/// Free mints issued to account.
		FreeMintsIssued { to: T::AccountId, how_many: MintCount },
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
		/// The player must wait cooldown period.
		MintCooldown,
		/// The player has not enough funds.
		InsufficientFunds,
		/// The season's variants + components exceed the maximum number of random bytes allowed
		/// (32)
		ExceededMaxRandomBytes,
		/// The player has not free mints available.
		InsufficientFreeMints,
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
		pub fn mint(origin: OriginFor<T>, mint_option: MintOption) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Self::do_mint(&player, &mint_option)
		}

		#[pallet::weight(10_000)]
		pub fn transfer_free_mints(
			origin: OriginFor<T>,
			dest: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let sender_free_mints = FreeMints::<T>::get(&sender)
				.checked_sub(how_many.checked_add(1).ok_or(ArithmeticError::Overflow)?)
				.ok_or(Error::<T>::InsufficientFreeMints)?;
			let dest_free_mints = FreeMints::<T>::get(&dest)
				.checked_add(how_many)
				.ok_or(ArithmeticError::Overflow)?;

			FreeMints::<T>::insert(&sender, sender_free_mints);
			FreeMints::<T>::insert(&dest, dest_free_mints);

			Self::deposit_event(Event::FreeMintsTransferred { from: sender, to: dest, how_many });
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn issue_free_mints(
			origin: OriginFor<T>,
			dest: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			let dest_free_mints = FreeMints::<T>::get(&dest)
				.checked_add(how_many)
				.ok_or(ArithmeticError::Overflow)?;
			FreeMints::<T>::insert(&dest, dest_free_mints);
			Self::deposit_event(Event::FreeMintsIssued { to: dest, how_many });
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn update_global_config(
			origin: OriginFor<T>,
			new_global_config: GlobalConfigOf<T>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			GlobalConfigs::<T>::put(&new_global_config);
			Self::deposit_event(Event::UpdatedGlobalConfig(new_global_config));
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
			ensure!(
				season.max_variations + season.max_components <= MAX_RANDOM_BYTES,
				Error::<T>::ExceededMaxRandomBytes
			);

			Self::validate_rarity_tiers(&mut season.rarity_tiers)?;
			Self::validate_rarity_tiers(&mut season.rarity_tiers_batch_mint)?;

			Ok(season)
		}

		fn validate_rarity_tiers(rarity_tiers: &mut RarityTiers) -> DispatchResult {
			rarity_tiers.sort_by(|a, b| b.1.cmp(&a.1));
			let (tiers, chances) = {
				let (mut tiers, chances): (Vec<_>, Vec<_>) =
					rarity_tiers.clone().into_iter().unzip();
				tiers.sort();
				tiers.dedup();
				(tiers, chances)
			};
			ensure!(
				chances.iter().sum::<RarityPercent>() == MAX_PERCENTAGE,
				Error::<T>::IncorrectRarityPercentages
			);
			ensure!(tiers.len() == chances.len(), Error::<T>::DuplicatedRarityTier);

			Ok(())
		}

		#[inline]
		fn random_hash(phrase: &[u8], who: &T::AccountId) -> T::Hash {
			let (seed, _) = T::Randomness::random(phrase);
			let seed = T::Hash::decode(&mut TrailingZeroInput::new(seed.as_ref()))
				.expect("input is padded with zeroes; qed");
			let nonce = frame_system::Pallet::<T>::account_nonce(who);
			frame_system::Pallet::<T>::inc_account_nonce(who);
			(seed, &who, nonce.encode()).using_encoded(T::Hashing::hash)
		}

		#[inline]
		fn random_component(
			season: &SeasonOf<T>,
			hash: &T::Hash,
			index: usize,
			batched_mint: bool,
		) -> (u8, u8) {
			let hash = hash.as_ref();
			let random_tier = {
				let random_percent = hash[index] % MAX_PERCENTAGE;
				let rarity_tiers = if batched_mint {
					&season.rarity_tiers_batch_mint
				} else {
					&season.rarity_tiers
				};
				let mut cumulative_sum = 0;
				let mut random_tier = rarity_tiers[0].0.clone() as u8;
				for (tier, chance) in rarity_tiers.iter() {
					let new_cumulative_sum = cumulative_sum + chance;
					if random_percent >= cumulative_sum && random_percent < new_cumulative_sum {
						random_tier = tier.clone() as u8;
						break
					}
					cumulative_sum = new_cumulative_sum;
				}
				random_tier
			};
			let random_variation = hash[index] % season.max_variations;
			(random_tier, random_variation)
		}

		#[inline]
		fn calculate_soul_points_from_dna(dna: &Dna, season: &SeasonOf<T>) -> SoulPoints {
			// We need to calculate the maximum possible value for a single DNA strand based on
			// the formula in Self::random_dna
			let max_dna_value = (((Mythical as u8) << 4) | season.max_variations) as u32;
			let value_scale = 100_u32;
			// We scale the total sum of the DNA strands to value_scale, we set the maximum
			// possible value of the sum to the max value of a single strand multiplied
			// by the amount of components the strand has
			(dna.iter().map(|i| *i as u32).sum::<u32>() * value_scale) /
				(max_dna_value * dna.len() as u32)
		}

		pub(crate) fn random_dna(
			hash: &T::Hash,
			season: &SeasonOf<T>,
			batched_mint: bool,
		) -> Result<(Dna, bool), DispatchError> {
			let dna = (0..season.max_components)
				.map(|i| {
					let (random_tier, random_variation) =
						Self::random_component(season, hash, i as usize * 2, batched_mint);
					((random_tier << 4) | random_variation) as u8
				})
				.collect::<Vec<_>>();
			let is_rare = dna.iter().all(|each| (each >> 4) >= RarityTier::Legendary as u8);
			Dna::try_from(dna)
				.map(|x| (x, is_rare))
				.map_err(|_| Error::<T>::IncorrectDna.into())
		}

		pub(crate) fn do_mint(player: &T::AccountId, mint_option: &MintOption) -> DispatchResult {
			let season_configs = Self::global_configs();
			ensure!(season_configs.mint_available, Error::<T>::MintUnavailable);

			let current_block = <frame_system::Pallet<T>>::block_number();
			if let Some(last_block) = Self::last_minted_block_numbers(&player) {
				let cooldown = season_configs.mint_cooldown;
				ensure!(current_block > last_block + cooldown, Error::<T>::MintCooldown);
			}

			match mint_option.mint_type {
				MintType::Normal => {
					let fee = season_configs.mint_fees.fee_for(mint_option.count);
					ensure!(
						T::Currency::free_balance(player) >= fee,
						Error::<T>::InsufficientFunds
					);
				},
				MintType::Free => ensure!(
					Self::free_mints(player) >= mint_option.count as MintCount,
					Error::<T>::InsufficientFreeMints
				),
			};

			let how_many = mint_option.count as usize;
			let max_ownership = (MAX_AVATARS_PER_PLAYER as usize)
				.checked_sub(how_many)
				.ok_or(ArithmeticError::Underflow)?;
			ensure!(Self::owners(player).len() <= max_ownership, Error::<T>::MaxOwnershipReached);

			let active_season_id = match Self::active_season_id() {
				Some(season_id) => season_id,
				None => {
					let last_season_id = Self::next_active_season_id()
						.checked_sub(1)
						.ok_or(ArithmeticError::Underflow)?;
					let last_season =
						Self::seasons(last_season_id).ok_or(Error::<T>::OutOfSeason)?;
					ensure!(
						Self::is_season_premature_closed(
							last_season,
							<frame_system::Pallet<T>>::block_number()
						),
						Error::<T>::OutOfSeason
					);
					last_season_id
				},
			};

			let active_season = Self::seasons(active_season_id).ok_or(Error::<T>::UnknownSeason)?;

			let generated_avatars = (0..how_many)
				.map(|_| {
					let avatar_id = Self::random_hash(b"create_avatar", player);
					let (dna, is_rare) =
						Self::random_dna(&avatar_id, &active_season, how_many > 1)?;
					let soul_points = Self::calculate_soul_points_from_dna(&dna, &active_season);
					let avatar = Avatar { season: active_season_id, dna, souls: soul_points };
					Ok((avatar_id, avatar, is_rare))
				})
				.collect::<Result<Vec<(AvatarIdOf<T>, Avatar, bool)>, DispatchError>>()?;

			let mut rare_avatars = 0;
			Owners::<T>::try_mutate(&player, |avatar_ids| -> DispatchResult {
				let generated_avatars_ids = generated_avatars
					.into_iter()
					.map(|(avatar_id, avatar, is_rare)| {
						Avatars::<T>::insert(avatar_id, (&player, avatar));
						if is_rare {
							ActiveSeasonRareMints::<T>::mutate(|count| count.saturating_inc());
							rare_avatars.saturating_inc();
						}
						avatar_id
					})
					.collect::<Vec<_>>();
				ensure!(
					avatar_ids.try_extend(generated_avatars_ids.clone().into_iter()).is_ok(),
					Error::<T>::MaxOwnershipReached
				);
				LastMintedBlockNumbers::<T>::insert(&player, current_block);

				match mint_option.mint_type {
					MintType::Normal => {
						T::Currency::withdraw(
							player,
							season_configs.mint_fees.fee_for(mint_option.count),
							WithdrawReasons::FEE,
							ExistenceRequirement::KeepAlive,
						)?;
					},
					MintType::Free => {
						let _ = FreeMints::<T>::try_mutate(
							player,
							|dest_player_free_mints| -> DispatchResult {
								*dest_player_free_mints = dest_player_free_mints
									.checked_sub(mint_option.count as MintCount)
									.ok_or(ArithmeticError::Overflow)?;
								Ok(())
							},
						);
					},
				};

				Self::deposit_event(Event::AvatarsMinted { avatar_ids: generated_avatars_ids });
				if rare_avatars > 0 {
					Self::deposit_event(Event::RareAvatarsMinted { count: rare_avatars });
				}
				Ok(())
			})
		}

		fn is_season_premature_closed(season: SeasonOf<T>, now: T::BlockNumber) -> bool {
			let current_high_tier_minted = Self::active_season_rare_mints();
			(season.early_start <= now && now <= season.end) &&
				(current_high_tier_minted < season.max_rare_mints)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let season_id = Self::active_season_id().unwrap_or_else(Self::next_active_season_id);
			let mut db_weight = T::DbWeight::get().reads(2);

			if let Some(season) = Self::seasons(season_id) {
				db_weight += T::DbWeight::get().reads(2);
				if Self::is_season_premature_closed(season, now) {
					ActiveSeasonId::<T>::put(season_id);
					NextActiveSeasonId::<T>::put(season_id.saturating_add(1));
					Self::deposit_event(Event::SeasonStarted(season_id));
					db_weight += T::DbWeight::get().writes(3);
				} else {
					ActiveSeasonRareMints::<T>::kill();
					if let Some(season_id) = ActiveSeasonId::<T>::take() {
						Self::deposit_event(Event::SeasonFinished(season_id));
					}
					db_weight += T::DbWeight::get().writes(3);
				}
			}
			db_weight
		}
	}
}
