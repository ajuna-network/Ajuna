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

//! # Ajuna Awesome Avatars
//!
//! The Awesome Ajuna Avatars is a collective game based on the Heroes of Ajuna.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Error`]
//! - [`Event`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! Some dispatchable functions can be called only from the organizer.
//!
//! * `mint` - Create a new AAA.
//! * `forge` - Sacrifice a batch of avatars in order to improve a leader.
//! * `transfer_free_mints` - Send free mints to another player.
//! * `set_price` - Assign a price to an avatar.
//! * `remove_price` - Remove the price of an avatar.
//! * `buy` - Buy an avatar.
//! * `upgrade_storage` -
//! * `set_organizer` - Set the game organizer.
//! * `set_treasurer` - Set the treasurer.
//! * `set_season` - Add a new season.
//! * `update_global_config` - Update the configuration.
//! * `issue_free_mints` - Give a number of free mints to a player.
//!
//! ### Public Functions
//!
//! * `do_forge` - Forge avatar.
//! * `do_mint` - Mint avatar.
//! * `ensure_season` - Given a season id and a season, validate them.

#![feature(map_first_last)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::{types::*, weights::WeightInfo};
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::AllowDeath, Randomness, WithdrawReasons},
	};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::{
		traits::{Hash, Saturating, TrailingZeroInput, UniqueSaturatedInto, Zero},
		ArithmeticError,
	};
	use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

	pub(crate) type SeasonOf<T> = Season<<T as frame_system::Config>::BlockNumber>;
	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type BoundedAvatarIdsOf<T> = BoundedVec<AvatarIdOf<T>, MaxAvatarsPerPlayer>;
	pub(crate) type GlobalConfigOf<T> =
		GlobalConfig<BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

	pub(crate) const MAX_PERCENTAGE: u8 = 100;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn organizer)]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasurer)]
	pub type Treasurer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultSeasonId() -> SeasonId {
		1
	}

	/// Contains the identifier of the current season.
	#[pallet::storage]
	#[pallet::getter(fn current_season_id)]
	pub type CurrentSeasonId<T: Config> = StorageValue<_, SeasonId, ValueQuery, DefaultSeasonId>;

	#[pallet::storage]
	#[pallet::getter(fn current_season_status)]
	pub type CurrentSeasonStatus<T: Config> = StorageValue<_, SeasonStatus, ValueQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	#[pallet::getter(fn seasons)]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, SeasonOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasury)]
	pub type Treasury<T: Config> = StorageMap<_, Identity, SeasonId, BalanceOf<T>, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultGlobalConfig<T: Config>() -> GlobalConfigOf<T> {
		GlobalConfig {
			mint: MintConfig {
				open: true,
				fees: MintFees {
					one: 550_000_000_000_u64.unique_saturated_into(), // 0.55 BAJU
					three: 500_000_000_000_u64.unique_saturated_into(), // 0.5 BAJU
					six: 450_000_000_000_u64.unique_saturated_into(), // 0.45 BAJU
				},
				cooldown: 5_u8.into(),
				free_mint_fee_multiplier: 1,
				free_mint_transfer_fee: 1,
				min_free_mint_transfer: 1,
			},
			forge: ForgeConfig { open: true },
			trade: TradeConfig {
				open: true,
				buy_fee: 1_000_000_000_u64.unique_saturated_into(), // 0.01 BAJU
			},
			account: AccountConfig {
				storage_upgrade_fee: 1_000_000_000_000_u64.unique_saturated_into(), // 1 BAJU
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
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> =
		StorageMap<_, Identity, T::AccountId, AccountInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn season_stats)]
	pub type SeasonStats<T: Config> =
		StorageDoubleMap<_, Identity, SeasonId, Identity, T::AccountId, SeasonInfo, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn trade)]
	pub type Trade<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, BalanceOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An organizer has been set.
		OrganizerSet { organizer: T::AccountId },
		/// A treasurer has been set.
		TreasurerSet { treasurer: T::AccountId },
		/// The season configuration for {season_id} has been updated.
		UpdatedSeason { season_id: SeasonId, season: SeasonOf<T> },
		/// Global configuration updated.
		UpdatedGlobalConfig(GlobalConfigOf<T>),
		/// Avatars minted.
		AvatarsMinted { avatar_ids: Vec<AvatarIdOf<T>> },
		/// Avatar forged.
		AvatarForged { avatar_id: AvatarIdOf<T>, upgraded_components: u8 },
		/// A season has started.
		SeasonStarted(SeasonId),
		/// A season has finished.
		SeasonFinished(SeasonId),
		/// Free mints transferred between accounts.
		FreeMintsTransferred { from: T::AccountId, to: T::AccountId, how_many: MintCount },
		/// Free mints issued to account.
		FreeMintsIssued { to: T::AccountId, how_many: MintCount },
		/// Avatar has price set for trade.
		AvatarPriceSet { avatar_id: AvatarIdOf<T>, price: BalanceOf<T> },
		/// Avatar has price removed for trade.
		AvatarPriceUnset { avatar_id: AvatarIdOf<T> },
		/// Avatar has been traded.
		AvatarTraded { avatar_id: AvatarIdOf<T>, from: T::AccountId, to: T::AccountId },
		/// Storage tier has been upgraded.
		StorageTierUpgraded,
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
		/// The season's per period and periods configuration overflows.
		PeriodConfigOverflow,
		/// The season's periods configuration is indivisible by max variation.
		PeriodsIndivisible,
		/// The season doesn't exist.
		UnknownSeason,
		/// The avatar doesn't exist.
		UnknownAvatar,
		/// The avatar for sale doesn't exist.
		UnknownAvatarForSale,
		/// The tier doesn't exist.
		UnknownTier,
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
		/// Forging is not available at the moment.
		ForgeClosed,
		/// Trading is not available at the moment.
		TradeClosed,
		/// Attempt to mint or forge outside of an active season.
		OutOfSeason,
		/// Attempt to mint when the season has ended prematurely.
		PrematureSeasonEnd,
		/// Max ownership reached.
		MaxOwnershipReached,
		/// Max storage tier reached.
		MaxStorageTierReached,
		/// Avatar belongs to someone else.
		Ownership,
		/// Attempt to buy his or her own avatar.
		AlreadyOwned,
		/// Incorrect DNA.
		IncorrectDna,
		/// Incorrect Avatar ID.
		IncorrectAvatarId,
		/// Incorrect season ID.
		IncorrectSeasonId,
		/// The player must wait cooldown period.
		MintCooldown,
		/// The season's max components value is less than the minimum allowed (1).
		MaxComponentsTooLow,
		/// The season's max components value is more than the maximum allowed (random byte: 32).
		MaxComponentsTooHigh,
		/// The season's max variations value is less than the minimum allowed (1).
		MaxVariationsTooLow,
		/// The season's max variations value is more than the maximum allowed (15).
		MaxVariationsTooHigh,
		/// The player has not enough free mints available.
		InsufficientFreeMints,
		/// Attempt to transfer free mints lower than the minimum allowed.
		TooLowFreeMintTransfer,
		/// Less than minimum allowed sacrifices are used for forging.
		TooFewSacrifices,
		/// More than maximum allowed sacrifices are used for forging.
		TooManySacrifices,
		/// Leader is being sacrificed.
		LeaderSacrificed,
		/// An avatar listed for trade is used to forge.
		AvatarInTrade,
		/// Tried to forge avatars from different seasons.
		IncorrectAvatarSeason,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue a new avatar.
		///
		/// Emits `AvatarsMinted` event when successful.
		///
		/// Weight: `O(n)` where:
		/// - `n = max avatars per player`
		#[pallet::call_index(0)]
		#[pallet::weight({
			let n = MaxAvatarsPerPlayer::get();
			T::WeightInfo::mint_normal(n)
				.max(T::WeightInfo::mint_free(n))
		})]
		pub fn mint(origin: OriginFor<T>, mint_option: MintOption) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let is_early_access = mint_option.mint_type == MintType::Free;
			let (mut season_id, deactivated) = Self::toggle_season(is_early_access)?;
			if deactivated {
				season_id.saturating_inc();
			}
			Self::do_mint(&player, &mint_option, season_id)
		}

		/// Forge an avatar.
		///
		/// This action can enhance the skills of an avatar by consuming a batch of avatars.
		/// The minimum and maximum number of avatars that can be utilized for forging is
		/// defined in the season configuration.
		///
		/// Emits `AvatarForged` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::forge(MaxAvatarsPerPlayer::get()))]
		pub fn forge(
			origin: OriginFor<T>,
			leader: AvatarIdOf<T>,
			sacrifices: Vec<AvatarIdOf<T>>,
		) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let (season_id, _deactivated) = Self::toggle_season(false)?;
			Self::do_forge(&player, &leader, &sacrifices, season_id)
		}

		/// Transfer free mints to a given account.
		///
		/// Emits `FreeMintsTransferred` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::transfer_free_mints())]
		pub fn transfer_free_mints(
			origin: OriginFor<T>,
			to: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let GlobalConfig { mint, .. } = Self::global_configs();
			ensure!(how_many >= mint.min_free_mint_transfer, Error::<T>::TooLowFreeMintTransfer);
			let sender_free_mints = Self::accounts(&from)
				.free_mints
				.checked_sub(
					how_many
						.checked_add(mint.free_mint_transfer_fee)
						.ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(Error::<T>::InsufficientFreeMints)?;
			let dest_free_mints = Self::accounts(&to)
				.free_mints
				.checked_add(how_many)
				.ok_or(ArithmeticError::Overflow)?;

			Accounts::<T>::mutate(&from, |account| account.free_mints = sender_free_mints);
			Accounts::<T>::mutate(&to, |account| account.free_mints = dest_free_mints);

			Self::deposit_event(Event::FreeMintsTransferred { from, to, how_many });
			Ok(())
		}

		/// Set the price of a given avatar.
		///
		/// Only allowed while trade period is open.
		///
		/// Emits `AvatarPriceSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::set_price())]
		pub fn set_price(
			origin: OriginFor<T>,
			avatar_id: AvatarIdOf<T>,
			#[pallet::compact] price: BalanceOf<T>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(Self::global_configs().trade.open, Error::<T>::TradeClosed);
			let _ = Self::ensure_ownership(&seller, &avatar_id)?;
			Trade::<T>::insert(avatar_id, price);
			Self::deposit_event(Event::AvatarPriceSet { avatar_id, price });
			Ok(())
		}

		/// Remove the price of a given avatar.
		///
		/// Only allowed while trade period is open.
		///
		/// Emits `AvatarPriceUnset` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_price())]
		pub fn remove_price(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(Self::global_configs().trade.open, Error::<T>::TradeClosed);
			Self::ensure_for_trade(&avatar_id)?;
			Self::ensure_ownership(&seller, &avatar_id)?;
			Trade::<T>::remove(avatar_id);
			Self::deposit_event(Event::AvatarPriceUnset { avatar_id });
			Ok(())
		}

		/// Buy the given avatar.
		///
		/// It consumes tokens for the trade operation. The avatar will be owned by the
		/// player after the transaction.
		///
		/// Only allowed while trade period is open.
		///
		/// Emits `AvatarTraded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::buy(MaxAvatarsPerPlayer::get()))]
		pub fn buy(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let GlobalConfig { trade, .. } = Self::global_configs();
			ensure!(trade.open, Error::<T>::TradeClosed);
			let (seller, price) = Self::ensure_for_trade(&avatar_id)?;
			ensure!(buyer != seller, Error::<T>::AlreadyOwned);

			T::Currency::transfer(&buyer, &seller, price, AllowDeath)?;

			let current_season_id = Self::current_season_id();
			let season_id = match Self::seasons(current_season_id) {
				Some(_) => current_season_id,
				None => current_season_id.saturating_sub(1),
			};
			T::Currency::withdraw(&buyer, trade.buy_fee, WithdrawReasons::FEE, AllowDeath)?;
			Treasury::<T>::mutate(season_id, |bal| bal.saturating_accrue(trade.buy_fee));

			let mut buyer_avatar_ids = Self::owners(&buyer);
			buyer_avatar_ids
				.try_push(avatar_id)
				.map_err(|_| Error::<T>::MaxOwnershipReached)?;

			ensure!(
				buyer_avatar_ids.len() <= Self::accounts(&buyer).storage_tier as usize,
				Error::<T>::MaxOwnershipReached
			);

			let mut seller_avatar_ids = Self::owners(&seller);
			seller_avatar_ids.retain(|x| x != &avatar_id);

			Owners::<T>::mutate(&buyer, |avatar_ids| *avatar_ids = buyer_avatar_ids);
			Owners::<T>::mutate(&seller, |avatar_ids| *avatar_ids = seller_avatar_ids);
			Avatars::<T>::mutate(avatar_id, |maybe_avatar| {
				if let Some((owner, _)) = maybe_avatar {
					*owner = buyer.clone();
				}
			});
			Trade::<T>::remove(avatar_id);

			Accounts::<T>::mutate(&buyer, |account| account.stats.trade.bought.saturating_inc());
			Accounts::<T>::mutate(&seller, |account| account.stats.trade.sold.saturating_inc());

			Self::deposit_event(Event::AvatarTraded { avatar_id, from: seller, to: buyer });
			Ok(())
		}

		/// Upgrade storage.
		///
		/// Emits `StorageTierUpgraded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::upgrade_storage())]
		pub fn upgrade_storage(origin: OriginFor<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let storage_tier = Self::accounts(&player).storage_tier;
			ensure!(storage_tier != StorageTier::Four, Error::<T>::MaxStorageTierReached);

			let upgrade_fee = Self::global_configs().account.storage_upgrade_fee;
			T::Currency::withdraw(&player, upgrade_fee, WithdrawReasons::FEE, AllowDeath)?;

			let season_id = Self::current_season_id();
			Treasury::<T>::mutate(season_id, |bal| bal.saturating_accrue(upgrade_fee));

			Accounts::<T>::mutate(&player, |account| account.storage_tier = storage_tier.upgrade());
			Self::deposit_event(Event::StorageTierUpgraded);
			Ok(())
		}

		/// Set game organizer.
		///
		/// The organizer account is like an admin, allowed to perform certain operations
		/// related with the game configuration like `set_season`, `ensure_free_mint` and
		/// `update_global_config`.
		///
		/// It can only be set by a root account.
		///
		/// Emits `OrganizerSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::set_organizer())]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		/// Set treasurer.
		///
		/// This is an additional treasury.
		///
		/// It can only be set by a root account.
		///
		/// Emits `TreasurerSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::set_treasurer())]
		pub fn set_treasurer(origin: OriginFor<T>, treasurer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Treasurer::<T>::put(&treasurer);
			Self::deposit_event(Event::TreasurerSet { treasurer });
			Ok(())
		}

		/// Set season.
		///
		/// Creates a new season. The new season can overlap with the already existing.
		///
		/// It can only be set by an organizer account.
		///
		/// Emits `UpdatedSeason` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::set_season())]
		pub fn set_season(
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

		/// Update global configuration.
		///
		/// It can only be called by an organizer account.
		///
		/// Emits `UpdatedGlobalConfig` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::update_global_config())]
		pub fn update_global_config(
			origin: OriginFor<T>,
			new_global_config: GlobalConfigOf<T>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			GlobalConfigs::<T>::put(&new_global_config);
			Self::deposit_event(Event::UpdatedGlobalConfig(new_global_config));
			Ok(())
		}

		/// Issue free mints.
		///
		/// It can only be called by an organizer account.
		///
		/// Emits `FreeMintsIssued` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::issue_free_mints())]
		pub fn issue_free_mints(
			origin: OriginFor<T>,
			to: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			let new_free_mints = Self::accounts(&to)
				.free_mints
				.checked_add(how_many)
				.ok_or(ArithmeticError::Overflow)?;
			Accounts::<T>::mutate(&to, |account| account.free_mints = new_free_mints);
			Self::deposit_event(Event::FreeMintsIssued { to, how_many });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check that the origin is an organizer account.
		pub(crate) fn ensure_organizer(origin: OriginFor<T>) -> DispatchResult {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Self::organizer().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(())
		}

		/// Validates a new season.
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
				let random_prob = hash[index] % MAX_PERCENTAGE;
				let probs =
					if batched_mint { &season.batch_mint_probs } else { &season.single_mint_probs };
				let mut cumulative_sum = 0;
				let mut random_tier = season.tiers[0].clone() as u8;
				for i in 0..probs.len() {
					let new_cumulative_sum = cumulative_sum + probs[i];
					if random_prob >= cumulative_sum && random_prob < new_cumulative_sum {
						random_tier = season.tiers[i].clone() as u8;
						break
					}
					cumulative_sum = new_cumulative_sum;
				}
				random_tier
			};
			let random_variation = hash[index + 1] % season.max_variations;
			(random_tier, random_variation)
		}

		fn random_dna(
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

		/// Mint a new avatar.
		pub(crate) fn do_mint(
			player: &T::AccountId,
			mint_option: &MintOption,
			season_id: SeasonId,
		) -> DispatchResult {
			let GlobalConfig { mint, .. } = Self::global_configs();
			ensure!(mint.open, Error::<T>::MintClosed);
			ensure!(!Self::current_season_status().early_ended, Error::<T>::PrematureSeasonEnd);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let last_block = Self::accounts(player).stats.mint.last;
			if !last_block.is_zero() {
				ensure!(current_block >= last_block + mint.cooldown, Error::<T>::MintCooldown);
			}

			let season = Self::seasons(season_id).ok_or(Error::<T>::UnknownSeason)?;
			let is_batched = mint_option.count.is_batched();
			let generated_avatar_ids = (0..mint_option.count as usize)
				.map(|_| {
					let avatar_id = Self::random_hash(b"create_avatar", player);
					let dna = Self::random_dna(&avatar_id, &season, is_batched)?;
					let souls = (dna.iter().map(|x| *x as SoulCount).sum::<SoulCount>() % 100) + 1;
					let avatar = Avatar { season_id, dna, souls };
					Avatars::<T>::insert(avatar_id, (&player, avatar));
					Owners::<T>::try_append(&player, avatar_id)
						.map_err(|_| Error::<T>::MaxOwnershipReached)?;
					Ok(avatar_id)
				})
				.collect::<Result<Vec<AvatarIdOf<T>>, DispatchError>>()?;

			ensure!(
				Self::owners(player).len() <= Self::accounts(player).storage_tier as usize,
				Error::<T>::MaxOwnershipReached
			);

			match mint_option.mint_type {
				MintType::Normal => {
					let fee = mint.fees.fee_for(&mint_option.count);
					T::Currency::withdraw(player, fee, WithdrawReasons::FEE, AllowDeath)?;
					Treasury::<T>::mutate(season_id, |bal| bal.saturating_accrue(fee));
				},
				MintType::Free => {
					let fee = (mint_option.count as MintCount)
						.saturating_mul(mint.free_mint_fee_multiplier);
					let free_mints = Self::accounts(player)
						.free_mints
						.checked_sub(fee)
						.ok_or(Error::<T>::InsufficientFreeMints)?;
					Accounts::<T>::mutate(player, |account| account.free_mints = free_mints);
				},
			};

			Accounts::<T>::try_mutate(player, |AccountInfo { stats, .. }| -> DispatchResult {
				if stats.mint.first.is_zero() {
					stats.mint.first = current_block;
				}
				stats.mint.last = current_block;
				stats
					.mint
					.seasons_participated
					.try_insert(season_id)
					.map_err(|_| Error::<T>::IncorrectSeasonId)?;
				Ok(())
			})?;
			SeasonStats::<T>::mutate(season_id, player, |info| {
				info.minted.saturating_accrue(generated_avatar_ids.len() as Stat);
			});

			Self::deposit_event(Event::AvatarsMinted { avatar_ids: generated_avatar_ids });
			Ok(())
		}

		/// Enhance an avatar using a batch of avatars.
		pub(crate) fn do_forge(
			player: &T::AccountId,
			leader_id: &AvatarIdOf<T>,
			sacrifice_ids: &[AvatarIdOf<T>],
			season_id: SeasonId,
		) -> DispatchResult {
			let GlobalConfig { forge, .. } = Self::global_configs();
			ensure!(forge.open, Error::<T>::ForgeClosed);

			let season = Self::seasons(season_id).ok_or(Error::<T>::UnknownSeason)?;
			ensure!(
				sacrifice_ids.len() as u8 >= season.min_sacrifices,
				Error::<T>::TooFewSacrifices
			);
			ensure!(
				sacrifice_ids.len() as u8 <= season.max_sacrifices,
				Error::<T>::TooManySacrifices
			);
			let max_tier = season.tiers.iter().max().ok_or(Error::<T>::UnknownTier)?.clone() as u8;

			ensure!(Self::ensure_for_trade(leader_id).is_err(), Error::<T>::AvatarInTrade);
			ensure!(
				sacrifice_ids.iter().all(|id| Self::ensure_for_trade(id).is_err()),
				Error::<T>::AvatarInTrade
			);
			ensure!(!sacrifice_ids.contains(leader_id), Error::<T>::LeaderSacrificed);

			let mut leader = Self::ensure_ownership(player, leader_id)?;
			let sacrifice_ids = sacrifice_ids.iter().copied().collect::<BTreeSet<_>>();
			let sacrifices = sacrifice_ids
				.iter()
				.map(|id| Self::ensure_ownership(player, id))
				.collect::<Result<Vec<Avatar>, DispatchError>>()?;

			ensure!(leader.season_id == season_id, Error::<T>::IncorrectAvatarSeason);
			ensure!(
				sacrifices.iter().all(|avatar| avatar.season_id == season_id),
				Error::<T>::IncorrectAvatarSeason
			);

			let (mut unique_matched_indexes, matches) =
				leader.compare_all::<T>(&sacrifices, season.max_variations, max_tier)?;

			let random_hash = Self::random_hash(b"forging avatar", player);
			let random_hash = random_hash.as_ref();
			let mut upgraded_components = 0;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let prob = leader.forge_probability::<T>(&season, &current_block, matches);

			let rolls = sacrifices.len();
			for hash in random_hash.iter().take(rolls) {
				if let Some(first_matched_index) = unique_matched_indexes.pop_first() {
					let roll = hash % MAX_PERCENTAGE;
					if roll <= prob {
						let nucleotide = leader.dna[first_matched_index];
						let current_tier_index = season
							.tiers
							.iter()
							.position(|tier| tier.clone() as u8 == nucleotide >> 4)
							.ok_or(Error::<T>::UnknownTier)?;

						let already_maxed_out = current_tier_index == (season.tiers.len() - 1);
						if !already_maxed_out {
							let next_tier = season.tiers[current_tier_index + 1].clone() as u8;
							let upgraded_nucleotide = (next_tier << 4) | (nucleotide & 0b0000_1111);
							leader.dna[first_matched_index] = upgraded_nucleotide;
							upgraded_components += 1;
						}
					}
				}
			}

			if leader.min_tier::<T>()? == max_tier {
				CurrentSeasonStatus::<T>::mutate(|status| {
					status.max_tier_avatars.saturating_inc();
					if status.max_tier_avatars == season.max_tier_forges {
						status.early_ended = true;
					}
				});
			}

			Avatars::<T>::insert(leader_id, (player, leader));
			sacrifice_ids.iter().for_each(Avatars::<T>::remove);
			let remaining_avatar_ids = Owners::<T>::take(player)
				.into_iter()
				.filter(|avatar_id| !sacrifice_ids.contains(avatar_id))
				.collect::<Vec<_>>();
			let remaining_avatar_ids = BoundedAvatarIdsOf::<T>::try_from(remaining_avatar_ids)
				.map_err(|_| Error::<T>::IncorrectAvatarId)?;
			Owners::<T>::insert(player, remaining_avatar_ids);

			Accounts::<T>::try_mutate(player, |AccountInfo { stats, .. }| -> DispatchResult {
				if stats.forge.first.is_zero() {
					stats.forge.first = current_block;
				}
				stats.forge.last = current_block;
				stats
					.forge
					.seasons_participated
					.try_insert(season_id)
					.map_err(|_| Error::<T>::IncorrectSeasonId)?;
				Ok(())
			})?;
			SeasonStats::<T>::mutate(season_id, player, |info| info.forged.saturating_inc());

			Self::deposit_event(Event::AvatarForged { avatar_id: *leader_id, upgraded_components });
			Ok(())
		}

		fn ensure_ownership(
			player: &T::AccountId,
			avatar_id: &AvatarIdOf<T>,
		) -> Result<Avatar, DispatchError> {
			let (owner, avatar) = Self::avatars(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			ensure!(player == &owner, Error::<T>::Ownership);
			Ok(avatar)
		}

		fn ensure_for_trade(
			avatar_id: &AvatarIdOf<T>,
		) -> Result<(T::AccountId, BalanceOf<T>), DispatchError> {
			let price = Self::trade(avatar_id).ok_or(Error::<T>::UnknownAvatarForSale)?;
			let (seller, _) = Self::avatars(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			Ok((seller, price))
		}

		fn toggle_season(early_access: bool) -> Result<(SeasonId, bool), DispatchError> {
			let current_season_id = Self::current_season_id();
			let mut season_deactivated = false;
			if let Some(season) = Self::seasons(current_season_id) {
				let now = <frame_system::Pallet<T>>::block_number();
				let is_current_season_active = Self::current_season_status().active;

				// activate early season
				if !is_current_season_active && season.is_early(now) {
					CurrentSeasonStatus::<T>::mutate(|status| status.early = true);
				}

				// activate season
				if !is_current_season_active && season.is_active(now) {
					Self::start_season(current_season_id);
				}

				// deactivate season (and active if condition met)
				if now > season.end {
					Self::finish_season(current_season_id);
					season_deactivated = true;
					let next_season_id = current_season_id.saturating_add(1);
					if let Some(next_season) = Self::seasons(next_season_id) {
						if next_season.is_active(now) {
							Self::start_season(next_season_id);
						}
					}
				}
			}

			// Failed extrinsics roll back storage changes as they're atomic, meaning it's
			// impossible to deactivate a season and check for out of season inside the same
			// extrinsic (since deactivation will be rolled back). Hence this condition is required
			// to allow for one extra mint / forge to happen when a season is deactivated.
			if !season_deactivated {
				let SeasonStatus { early, active, .. } = Self::current_season_status();
				ensure!(
					if early_access { early || active } else { active },
					Error::<T>::OutOfSeason
				);
			}

			Ok((current_season_id, season_deactivated))
		}

		fn start_season(season_id: SeasonId) {
			CurrentSeasonStatus::<T>::mutate(|status| {
				status.early = false;
				status.active = true;
				status.early_ended = false;
				status.max_tier_avatars = Zero::zero();
			});
			Self::deposit_event(Event::SeasonStarted(season_id));
		}

		fn finish_season(season_id: SeasonId) {
			CurrentSeasonStatus::<T>::mutate(|status| {
				status.early = false;
				status.active = false;
				status.early_ended = false;
				status.max_tier_avatars = Zero::zero();
			});
			CurrentSeasonId::<T>::put(season_id.saturating_add(1));
			Self::deposit_event(Event::SeasonFinished(season_id));
		}
	}
}
