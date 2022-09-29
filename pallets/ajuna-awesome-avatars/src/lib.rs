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
		traits::{Currency, ExistenceRequirement, Randomness, WithdrawReasons},
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
	pub(crate) const FREE_MINT_TRANSFER_FEE: MintCount = 1;

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
	pub fn DefaultSeasonId() -> SeasonId {
		1
	}

	#[pallet::storage]
	#[pallet::getter(fn current_season_id)]
	pub type CurrentSeasonId<T: Config> = StorageValue<_, SeasonId, ValueQuery, DefaultSeasonId>;

	#[pallet::storage]
	#[pallet::getter(fn is_season_active)]
	pub type IsSeasonActive<T: Config> = StorageValue<_, bool, ValueQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	#[pallet::getter(fn seasons)]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, SeasonOf<T>, OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultGlobalConfig<T: Config>() -> GlobalConfigOf<T> {
		GlobalConfig {
			mint: MintConfig {
				open: false,
				fees: MintFees {
					one: (1_000_000_000_000_u64 * 55 / 100).unique_saturated_into(),
					three: (1_000_000_000_000_u64 * 50 / 100).unique_saturated_into(),
					six: (1_000_000_000_000_u64 * 45 / 100).unique_saturated_into(),
				},
				cooldown: 5_u8.into(),
			},
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
		/// The season configuration for {season_id} has been updated.
		UpdatedSeason { season_id: SeasonId, season: SeasonOf<T> },
		/// Global configuration updated.
		UpdatedGlobalConfig(GlobalConfigOf<T>),
		/// Avatars minted.
		AvatarsMinted { avatar_ids: Vec<AvatarIdOf<T>> },
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
		/// The season ID of a season to create is not sequential.
		NonSequentialSeasonId,
		/// Rarity percentages don't add up to 100
		IncorrectRarityPercentages,
		/// Max tier is achievable through forging only. Therefore the number of rarity percentages
		/// must be less than that of tiers for a season.
		TooManyRarityPercentages,
		/// Some rarity tier are duplicated.
		DuplicatedRarityTier,
		/// Minting is not available at the moment.
		MintClosed,
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
		pub fn mint(origin: OriginFor<T>, mint_option: MintOption) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Self::toggle_season();
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
				.checked_sub(
					how_many
						.checked_add(FREE_MINT_TRANSFER_FEE)
						.ok_or(ArithmeticError::Overflow)?,
				)
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
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn upsert_season(
			origin: OriginFor<T>,
			season_id: SeasonId,
			season: SeasonOf<T>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			let season = Self::ensure_season(&season_id, season)?;
			Seasons::<T>::insert(season_id, &season);
			Self::deposit_event(Event::UpdatedSeason { season_id, season });
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
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn ensure_organizer(origin: OriginFor<T>) -> DispatchResult {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(())
		}

		pub(crate) fn ensure_season(
			season_id: &SeasonId,
			mut season: SeasonOf<T>,
		) -> Result<SeasonOf<T>, DispatchError> {
			season.validate::<T>()?;

			let prev_season_id = season_id.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			if prev_season_id > 0 {
				let prev_season =
					Self::seasons(prev_season_id).ok_or(Error::<T>::NonSequentialSeasonId)?;
				ensure!(prev_season.end < season.early_start, Error::<T>::EarlyStartTooEarly);
			}
			if let Some(next_season) = Self::seasons(next_season_id) {
				ensure!(season.end < next_season.early_start, Error::<T>::SeasonEndTooLate);
			}
			Ok(season)
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
				let random_p = hash[index] % MAX_PERCENTAGE;
				let p = if batched_mint { &season.p_batch_mint } else { &season.p_single_mint };
				let mut cumulative_sum = 0;
				let mut random_tier = season.tiers[0].clone() as u8;
				for i in 0..p.len() {
					let new_cumulative_sum = cumulative_sum + p[i];
					if random_p >= cumulative_sum && random_p < new_cumulative_sum {
						random_tier = season.tiers[i].clone() as u8;
						break
					}
					cumulative_sum = new_cumulative_sum;
				}
				random_tier
			};
			let random_variation = hash[index] % season.max_variations;
			(random_tier, random_variation)
		}

		pub(crate) fn random_dna(
			hash: &T::Hash,
			season: &SeasonOf<T>,
			batched_mint: bool,
		) -> Result<Dna, DispatchError> {
			let dna = (0..season.max_components)
				.map(|i| {
					let (random_tier, random_variation) =
						Self::random_component(season, hash, i as usize * 2, batched_mint);
					((random_tier << 4) | random_variation) as u8
				})
				.collect::<Vec<_>>();
			Dna::try_from(dna).map_err(|_| Error::<T>::IncorrectDna.into())
		}

		pub(crate) fn do_mint(player: &T::AccountId, mint_option: &MintOption) -> DispatchResult {
			let GlobalConfig { mint, .. } = Self::global_configs();
			ensure!(mint.open, Error::<T>::MintClosed);

			let current_block = <frame_system::Pallet<T>>::block_number();
			if let Some(last_block) = Self::last_minted_block_numbers(&player) {
				ensure!(current_block > last_block + mint.cooldown, Error::<T>::MintCooldown);
			}

			let MintOption { mint_type, count } = mint_option;
			match mint_type {
				MintType::Normal => {
					let fee = mint.fees.fee_for(*count);
					ensure!(
						T::Currency::free_balance(player) >= fee,
						Error::<T>::InsufficientFunds
					);
				},
				MintType::Free => ensure!(
					Self::free_mints(player) >= *count as MintCount,
					Error::<T>::InsufficientFreeMints
				),
			};

			let how_many = *count as usize;
			let max_ownership = (MAX_AVATARS_PER_PLAYER as usize)
				.checked_sub(how_many)
				.ok_or(ArithmeticError::Underflow)?;
			ensure!(Self::owners(player).len() <= max_ownership, Error::<T>::MaxOwnershipReached);

			ensure!(Self::is_season_active(), Error::<T>::OutOfSeason);
			let season_id = Self::current_season_id();
			let season = Self::seasons(season_id).ok_or(Error::<T>::UnknownSeason)?;

			let generated_avatars = (0..how_many)
				.map(|_| {
					let avatar_id = Self::random_hash(b"create_avatar", player);
					let dna = Self::random_dna(&avatar_id, &season, how_many > 1)?;
					let souls = (dna.iter().map(|x| *x as SoulCount).sum::<SoulCount>() % 100) + 1;
					let avatar = Avatar { season_id, dna, souls };
					Ok((avatar_id, avatar))
				})
				.collect::<Result<Vec<(AvatarIdOf<T>, Avatar)>, DispatchError>>()?;

			Owners::<T>::try_mutate(&player, |avatar_ids| -> DispatchResult {
				let generated_avatars_ids = generated_avatars
					.into_iter()
					.map(|(avatar_id, avatar)| {
						Avatars::<T>::insert(avatar_id, (&player, avatar));
						avatar_id
					})
					.collect::<Vec<_>>();
				ensure!(
					avatar_ids.try_extend(generated_avatars_ids.clone().into_iter()).is_ok(),
					Error::<T>::MaxOwnershipReached
				);
				LastMintedBlockNumbers::<T>::insert(&player, current_block);

				match mint_type {
					MintType::Normal => {
						T::Currency::withdraw(
							player,
							mint.fees.fee_for(*count),
							WithdrawReasons::FEE,
							ExistenceRequirement::KeepAlive,
						)?;
					},
					MintType::Free => {
						let _ = FreeMints::<T>::try_mutate(
							player,
							|dest_player_free_mints| -> DispatchResult {
								*dest_player_free_mints = dest_player_free_mints
									.checked_sub(*count as MintCount)
									.ok_or(ArithmeticError::Overflow)?;
								Ok(())
							},
						);
					},
				};

				Self::deposit_event(Event::AvatarsMinted { avatar_ids: generated_avatars_ids });
				Ok(())
			})
		}

		fn toggle_season() {
			let mut current_season_id = Self::current_season_id();
			if let Some(season) = Self::seasons(&current_season_id) {
				let now = <frame_system::Pallet<T>>::block_number();
				let is_active = now >= season.early_start && now <= season.end;
				let is_currently_active = Self::is_season_active();

				// activate season
				if !is_currently_active && is_active {
					Self::activate_season(current_season_id);
				}

				// deactivate season (and active if condition met)
				if now > season.end {
					Self::deactivate_season();
					current_season_id.saturating_inc();
					if let Some(next_season) = Self::seasons(current_season_id) {
						if now >= next_season.early_start {
							Self::activate_season(current_season_id);
						}
					}
				}
			}
		}

		fn activate_season(season_id: SeasonId) {
			IsSeasonActive::<T>::put(true);
			Self::deposit_event(Event::SeasonStarted(season_id));
		}

		fn deactivate_season() {
			IsSeasonActive::<T>::put(false);
			CurrentSeasonId::<T>::mutate(|season_id| {
				Self::deposit_event(Event::SeasonFinished(*season_id));
				season_id.saturating_inc()
			});
		}
	}
}
