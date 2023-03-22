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
//! * `upgrade_storage` - Upgrade the capacity to hold avatars.
//! * `set_organizer` - Set the game organizer.
//! * `set_treasurer` - Set the treasurer.
//! * `set_season` - Add a new season.
//! * `update_global_config` - Update the configuration.
//! * `set_free_mints` - Set a number of free mints to a player.
//!
//! ### Public Functions
//!
//! * `do_forge` - Forge avatar.
//! * `do_mint` - Mint avatar.
//! * `ensure_season` - Given a season id and a season, validate them.

#![feature(map_first_last, variant_count)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod migration;
pub mod types;
pub mod weights;

use crate::{types::*, weights::WeightInfo};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::AllowDeath, Randomness, WithdrawReasons},
	PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use pallet_ajuna_nft_transfer::traits::NftHandler;
use sp_runtime::{
	traits::{AccountIdConversion, Hash, Saturating, TrailingZeroInput, UniqueSaturatedInto, Zero},
	ArithmeticError,
};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type SeasonOf<T> = Season<BlockNumberFor<T>>;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type BoundedAvatarIdsOf<T> = BoundedVec<AvatarIdOf<T>, MaxAvatarsPerPlayer>;
	pub(crate) type GlobalConfigOf<T> = GlobalConfig<BalanceOf<T>, BlockNumberFor<T>>;

	pub(crate) type CollectionIdOf<T> =
		<<T as Config>::NftHandler as NftHandler<AccountIdOf<T>, Avatar>>::CollectionId;
	pub(crate) type AssetIdOf<T> =
		<<T as Config>::NftHandler as NftHandler<AccountIdOf<T>, Avatar>>::AssetId;

	pub(crate) const MAX_PERCENTAGE: u8 = 100;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(migration::STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type NftHandler: NftHandler<Self::AccountId, Avatar>;

		#[pallet::constant]
		type NftCollectionId: Get<CollectionIdOf<Self>>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn organizer)]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasurer)]
	pub type Treasurer<T: Config> = StorageMap<_, Identity, SeasonId, T::AccountId, OptionQuery>;

	/// Contains the identifier of the current season.
	#[pallet::storage]
	#[pallet::getter(fn current_season_id)]
	pub type CurrentSeasonId<T: Config> = StorageValue<_, SeasonId, ValueQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn global_configs)]
	pub type GlobalConfigs<T: Config> = StorageValue<_, GlobalConfigOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn avatars)]
	pub type Avatars<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, (T::AccountId, Avatar)>;

	#[pallet::storage]
	#[pallet::getter(fn owners)]
	pub type Owners<T: Config> =
		StorageMap<_, Identity, T::AccountId, BoundedAvatarIdsOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locked_avatars)]
	pub type LockedAvatars<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, AssetIdOf<T>>;

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

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: sp_std::marker::PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			CurrentSeasonId::<T>::put(1);
			GlobalConfigs::<T>::put(GlobalConfig {
				mint: MintConfig {
					open: true,
					fees: MintFees {
						one: 550_000_000_000_u64.unique_saturated_into(), // 0.55 BAJU
						three: 500_000_000_000_u64.unique_saturated_into(), // 0.5 BAJU
						six: 450_000_000_000_u64.unique_saturated_into(), // 0.45 BAJU
					},
					cooldown: 5_u8.into(),
					free_mint_fee_multiplier: 1,
				},
				forge: ForgeConfig { open: true },
				transfer: TransferConfig {
					open: true,
					free_mint_transfer_fee: 1,
					min_free_mint_transfer: 1,
					avatar_transfer_fee: 1_000_000_000_000_u64.unique_saturated_into(), // 1 BAJU
				},
				trade: TradeConfig {
					open: true,
					min_fee: 1_000_000_000_u64.unique_saturated_into(), // 0.01 BAJU
					percent_fee: 1,                                     // 1% of sales price
				},
				account: AccountConfig {
					storage_upgrade_fee: 1_000_000_000_000_u64.unique_saturated_into(), // 1 BAJU
				},
			});
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An organizer has been set.
		OrganizerSet { organizer: T::AccountId },
		/// A treasurer has been set for a season.
		TreasurerSet { season_id: SeasonId, treasurer: T::AccountId },
		/// A season's treasury has been claimed by a treasurer.
		TreasuryClaimed { season_id: SeasonId, treasurer: T::AccountId, amount: BalanceOf<T> },
		/// The season configuration for {season_id} has been updated.
		UpdatedSeason { season_id: SeasonId, season: SeasonOf<T> },
		/// Global configuration updated.
		UpdatedGlobalConfig(GlobalConfigOf<T>),
		/// Avatars minted.
		AvatarsMinted { avatar_ids: Vec<AvatarIdOf<T>> },
		/// Avatar forged.
		AvatarForged { avatar_id: AvatarIdOf<T>, upgraded_components: u8 },
		/// Avatar transferred.
		AvatarTransferred { from: T::AccountId, to: T::AccountId, avatar_id: AvatarIdOf<T> },
		/// A season has started.
		SeasonStarted(SeasonId),
		/// A season has finished.
		SeasonFinished(SeasonId),
		/// Free mints transferred between accounts.
		FreeMintsTransferred { from: T::AccountId, to: T::AccountId, how_many: MintCount },
		/// Free mints set for target account.
		FreeMintsSet { target: T::AccountId, how_many: MintCount },
		/// Avatar has price set for trade.
		AvatarPriceSet { avatar_id: AvatarIdOf<T>, price: BalanceOf<T> },
		/// Avatar has price removed for trade.
		AvatarPriceUnset { avatar_id: AvatarIdOf<T> },
		/// Avatar has been traded.
		AvatarTraded { avatar_id: AvatarIdOf<T>, from: T::AccountId, to: T::AccountId },
		/// Avatar locked.
		AvatarLocked { avatar_id: AvatarIdOf<T>, asset_id: AssetIdOf<T> },
		/// Avatar unlocked.
		AvatarUnlocked { avatar_id: AvatarIdOf<T> },
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
		/// The treasurer doesn't exist.
		UnknownTreasurer,
		/// The season ID of a season to create is not sequential.
		NonSequentialSeasonId,
		/// Rarity percentages don't add up to 100
		IncorrectRarityPercentages,
		/// Max tier is achievable through forging only. Therefore the number of rarity percentages
		/// must be less than that of tiers for a season.
		TooManyRarityPercentages,
		/// Some rarity tier are duplicated.
		DuplicatedRarityTier,
		/// Attempt to set fees lower than the existential deposit amount.
		TooLowFees,
		/// Minting is not available at the moment.
		MintClosed,
		/// Forging is not available at the moment.
		ForgeClosed,
		/// Transfer is not available at the moment.
		TransferClosed,
		/// Trading is not available at the moment.
		TradeClosed,
		/// Attempt to mint or forge outside of an active season.
		SeasonClosed,
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
		/// Attempt to transfer, issue or withdraw free mints lower than the minimum allowed.
		TooLowFreeMints,
		/// Less than minimum allowed sacrifices are used for forging.
		TooFewSacrifices,
		/// More than maximum allowed sacrifices are used for forging.
		TooManySacrifices,
		/// Leader is being sacrificed.
		LeaderSacrificed,
		/// An avatar listed for trade is used to forge.
		AvatarInTrade,
		/// The avatar is currently locked and cannot be used.
		AvatarLocked,
		/// The avatar is currently unlocked and cannot be locked again.
		AvatarUnlocked,
		/// Tried to forge avatars from different seasons.
		IncorrectAvatarSeason,
		/// Tried transferring to his or her own account.
		CannotTransferToSelf,
		/// Tried claiming treasury during a season.
		CannotClaimDuringSeason,
		/// Tried claiming treasury which is zero.
		CannotClaimZero,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let current_season_id = Self::current_season_id();
			let mut weight = T::DbWeight::get().reads(1);

			if let Some(current_season) = Self::seasons(current_season_id) {
				weight.saturating_accrue(T::DbWeight::get().reads(1));

				if now <= current_season.end {
					Self::start_season(&mut weight, now, current_season_id, &current_season);
				} else {
					Self::finish_season(&mut weight, now, current_season_id);
				}
			}

			weight
		}
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
			Self::do_mint(&player, &mint_option)
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
			Self::do_forge(&player, &leader, &sacrifices)
		}

		#[pallet::call_index(2)]
		#[pallet::weight({
			let n = MaxAvatarsPerPlayer::get();
			T::WeightInfo::transfer_avatar_normal(n)
				.max(T::WeightInfo::transfer_avatar_organizer(n))
		})]
		pub fn transfer_avatar(
			origin: OriginFor<T>,
			to: T::AccountId,
			avatar_id: AvatarIdOf<T>,
		) -> DispatchResult {
			let GlobalConfig { transfer, .. } = Self::global_configs();
			let from = match Self::ensure_organizer(origin.clone()) {
				Ok(organizer) => organizer,
				_ => {
					ensure!(transfer.open, Error::<T>::TransferClosed);
					ensure_signed(origin)?
				},
			};
			ensure!(from != to, Error::<T>::CannotTransferToSelf);
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);

			let avatar = Self::ensure_ownership(&from, &avatar_id)?;
			let fee = transfer.avatar_transfer_fee;
			T::Currency::withdraw(&from, fee, WithdrawReasons::FEE, AllowDeath)?;
			Self::deposit_into_treasury(&avatar.season_id, fee);

			Self::do_transfer_avatar(&from, &to, &avatar_id)?;
			Self::deposit_event(Event::AvatarTransferred { from, to, avatar_id });
			Ok(())
		}

		/// Transfer free mints to a given account.
		///
		/// Emits `FreeMintsTransferred` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::transfer_free_mints())]
		pub fn transfer_free_mints(
			origin: OriginFor<T>,
			to: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(from != to, Error::<T>::CannotTransferToSelf);

			let GlobalConfig { transfer, .. } = Self::global_configs();
			ensure!(how_many >= transfer.min_free_mint_transfer, Error::<T>::TooLowFreeMints);
			let sender_free_mints = Self::accounts(&from)
				.free_mints
				.checked_sub(
					how_many
						.checked_add(transfer.free_mint_transfer_fee)
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
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::set_price())]
		pub fn set_price(
			origin: OriginFor<T>,
			avatar_id: AvatarIdOf<T>,
			#[pallet::compact] price: BalanceOf<T>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(Self::global_configs().trade.open, Error::<T>::TradeClosed);
			Self::ensure_ownership(&seller, &avatar_id)?;
			Self::ensure_unlocked(&avatar_id)?;
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
		#[pallet::call_index(5)]
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
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::buy(MaxAvatarsPerPlayer::get()))]
		pub fn buy(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let GlobalConfig { trade, .. } = Self::global_configs();
			ensure!(trade.open, Error::<T>::TradeClosed);

			let (seller, price) = Self::ensure_for_trade(&avatar_id)?;
			ensure!(buyer != seller, Error::<T>::AlreadyOwned);
			T::Currency::transfer(&buyer, &seller, price, AllowDeath)?;

			let trade_fee = trade.min_fee.max(
				price.saturating_mul(trade.percent_fee.unique_saturated_into()) /
					MAX_PERCENTAGE.unique_saturated_into(),
			);
			T::Currency::withdraw(&buyer, trade_fee, WithdrawReasons::FEE, AllowDeath)?;

			let avatar = Self::ensure_ownership(&seller, &avatar_id)?;
			Self::deposit_into_treasury(&avatar.season_id, trade_fee);

			Self::do_transfer_avatar(&seller, &buyer, &avatar_id)?;
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
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::upgrade_storage())]
		pub fn upgrade_storage(origin: OriginFor<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let storage_tier = Self::accounts(&player).storage_tier;
			ensure!(storage_tier != StorageTier::Max, Error::<T>::MaxStorageTierReached);

			let upgrade_fee = Self::global_configs().account.storage_upgrade_fee;
			T::Currency::withdraw(&player, upgrade_fee, WithdrawReasons::FEE, AllowDeath)?;

			let season_id = Self::current_season_id();
			Self::deposit_into_treasury(&season_id, upgrade_fee);

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
		#[pallet::call_index(8)]
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
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::set_treasurer())]
		pub fn set_treasurer(
			origin: OriginFor<T>,
			season_id: SeasonId,
			treasurer: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			Treasurer::<T>::insert(season_id, &treasurer);
			Self::deposit_event(Event::TreasurerSet { season_id, treasurer });
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::claim_treasury())]
		pub fn claim_treasury(origin: OriginFor<T>, season_id: SeasonId) -> DispatchResult {
			let maybe_treasurer = ensure_signed(origin)?;
			let treasurer = Self::treasurer(season_id).ok_or(Error::<T>::UnknownTreasurer)?;
			ensure!(maybe_treasurer == treasurer, DispatchError::BadOrigin);

			let (current_season_id, season) = Self::current_season_with_id()?;
			ensure!(
				season_id < current_season_id ||
					(season_id == current_season_id &&
						<frame_system::Pallet<T>>::block_number() > season.end),
				Error::<T>::CannotClaimDuringSeason
			);

			let amount = Treasury::<T>::take(season_id);
			ensure!(!amount.is_zero(), Error::<T>::CannotClaimZero);

			T::Currency::transfer(&Self::account_id(), &treasurer, amount, AllowDeath)?;
			Self::deposit_event(Event::TreasuryClaimed { season_id, treasurer, amount });
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
		#[pallet::call_index(11)]
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
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::update_global_config())]
		pub fn update_global_config(
			origin: OriginFor<T>,
			new_global_config: GlobalConfigOf<T>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			ensure!(
				[
					new_global_config.mint.fees.one,
					new_global_config.mint.fees.three,
					new_global_config.mint.fees.six,
					new_global_config.transfer.avatar_transfer_fee,
					new_global_config.trade.min_fee,
					new_global_config.account.storage_upgrade_fee
				]
				.iter()
				.all(|x| x > &T::Currency::minimum_balance()),
				Error::<T>::TooLowFees
			);
			GlobalConfigs::<T>::put(&new_global_config);
			Self::deposit_event(Event::UpdatedGlobalConfig(new_global_config));
			Ok(())
		}

		/// Set free mints.
		///
		/// It can only be called by an organizer account.
		///
		/// Emits `FreeMintSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::set_free_mints())]
		pub fn set_free_mints(
			origin: OriginFor<T>,
			target: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			Accounts::<T>::mutate(&target, |account| account.free_mints = how_many);
			Self::deposit_event(Event::FreeMintsSet { target, how_many });
			Ok(())
		}

		#[pallet::call_index(14)]
		#[pallet::weight(10_000)]
		pub fn lock_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let avatar = Self::ensure_ownership(&account, &avatar_id)?;
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);
			Self::ensure_unlocked(&avatar_id)?;

			let asset_id = T::NftHandler::store_as_nft(account, T::NftCollectionId::get(), avatar)?;
			LockedAvatars::<T>::insert(avatar_id, &asset_id);
			Self::deposit_event(Event::AvatarLocked { avatar_id, asset_id });
			Ok(())
		}

		#[pallet::call_index(15)]
		#[pallet::weight(10_000)]
		pub fn unlock_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let _ = Self::ensure_ownership(&account, &avatar_id)?;

			let asset_id = Self::locked_avatars(avatar_id).ok_or(Error::<T>::AvatarUnlocked)?;
			let _ = T::NftHandler::recover_from_nft(account, T::NftCollectionId::get(), asset_id)?;

			LockedAvatars::<T>::remove(avatar_id);
			Self::deposit_event(Event::AvatarUnlocked { avatar_id });
			Ok(())
		}

		#[pallet::call_index(16)]
		#[pallet::weight(T::WeightInfo::fix_variation())]
		pub fn fix_variation(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let mut avatar = Self::ensure_ownership(&account, &avatar_id)?;

			// Update the variation of the 3nd component to be the same as that of the 2nd by
			// copying the rightmost 4 bits of dna[1] to the dna[2]
			avatar.dna[2] = (avatar.dna[2] & 0b1111_0000) | (avatar.dna[1] & 0b0000_1111);

			Avatars::<T>::insert(avatar_id, (account, avatar));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the treasury.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub(crate) fn deposit_into_treasury(season_id: &SeasonId, amount: BalanceOf<T>) {
			Treasury::<T>::mutate(season_id, |bal| bal.saturating_accrue(amount));
			T::Currency::deposit_creating(&Self::account_id(), amount);
		}

		/// Check that the origin is an organizer account.
		pub(crate) fn ensure_organizer(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Self::organizer().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(maybe_organizer)
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
		pub(crate) fn do_mint(player: &T::AccountId, mint_option: &MintOption) -> DispatchResult {
			let GlobalConfig { mint, .. } = Self::global_configs();
			ensure!(mint.open, Error::<T>::MintClosed);
			let free_mints = Self::ensure_for_mint(player, &mint_option.mint_type)?;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let last_block = Self::accounts(player).stats.mint.last;
			if !last_block.is_zero() {
				ensure!(current_block >= last_block + mint.cooldown, Error::<T>::MintCooldown);
			}

			let season_id = Self::current_season_id();
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
					Self::deposit_into_treasury(&season_id, fee);
				},
				MintType::Free => {
					let fee = (mint_option.count as MintCount)
						.saturating_mul(mint.free_mint_fee_multiplier);
					Accounts::<T>::try_mutate(player, |account| -> DispatchResult {
						account.free_mints =
							free_mints.checked_sub(fee).ok_or(Error::<T>::InsufficientFreeMints)?;
						Ok(())
					})?;
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
		) -> DispatchResult {
			let GlobalConfig { forge, .. } = Self::global_configs();
			ensure!(forge.open, Error::<T>::ForgeClosed);

			let (season_id, season) = Self::current_season_with_id()?;
			let (mut leader, sacrifice_ids, sacrifices) =
				Self::ensure_for_forge(player, leader_id, sacrifice_ids, &season_id, &season)?;
			let prev_leader_tier = leader.min_tier();

			let max_tier = season.tiers.iter().max().ok_or(Error::<T>::UnknownTier)?.clone() as u8;
			let (mut unique_matched_indexes, matches) =
				leader.compare_all::<T>(&sacrifices, season.max_variations, max_tier)?;

			let random_hash = Self::random_hash(b"forging avatar", player);
			let random_hash = random_hash.as_ref();
			let mut upgraded_components = 0;

			let current_block = <frame_system::Pallet<T>>::block_number();
			let prob = leader.forge_probability::<T>(&season, &current_block, matches);

			let rolls = sacrifices.len();
			for hash in random_hash.iter().take(rolls) {
				let roll = hash % MAX_PERCENTAGE;
				if roll <= prob {
					if let Some(first_matched_index) = unique_matched_indexes.pop_first() {
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

			let after_leader_tier = leader.min_tier();
			if prev_leader_tier != max_tier && after_leader_tier == max_tier {
				CurrentSeasonStatus::<T>::mutate(|status| {
					status.max_tier_avatars.saturating_inc();
					if status.max_tier_avatars == season.max_tier_forges {
						status.early_ended = true;
					}
				});
			}

			Avatars::<T>::insert(leader_id, (player, leader));
			sacrifice_ids.iter().for_each(Avatars::<T>::remove);
			let remaining_avatar_ids: BoundedAvatarIdsOf<T> = Owners::<T>::take(player)
				.into_iter()
				.filter(|avatar_id| !sacrifice_ids.contains(avatar_id))
				.collect::<Vec<_>>()
				.try_into()
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

		fn do_transfer_avatar(
			from: &T::AccountId,
			to: &T::AccountId,
			avatar_id: &AvatarIdOf<T>,
		) -> DispatchResult {
			let mut from_avatar_ids = Self::owners(from);
			from_avatar_ids.retain(|existing_avatar_id| existing_avatar_id != avatar_id);

			let mut to_avatar_ids = Self::owners(to);
			to_avatar_ids
				.try_push(*avatar_id)
				.map_err(|_| Error::<T>::MaxOwnershipReached)?;
			ensure!(
				to_avatar_ids.len() <= Self::accounts(to).storage_tier as usize,
				Error::<T>::MaxOwnershipReached
			);

			Owners::<T>::mutate(from, |avatar_ids| *avatar_ids = from_avatar_ids);
			Owners::<T>::mutate(to, |avatar_ids| *avatar_ids = to_avatar_ids);
			Avatars::<T>::try_mutate(avatar_id, |maybe_avatar| -> DispatchResult {
				let (from_owner, _) = maybe_avatar.as_mut().ok_or(Error::<T>::UnknownAvatar)?;
				*from_owner = to.clone();
				Ok(())
			})
		}

		fn current_season_with_id() -> Result<(SeasonId, SeasonOf<T>), DispatchError> {
			let mut season_id = Self::current_season_id();
			let season = match Self::seasons(season_id) {
				Some(season) if Self::current_season_status().is_in_season() => season,
				_ => {
					if season_id > 1 {
						season_id.saturating_dec();
					}
					Self::seasons(season_id).ok_or(Error::<T>::UnknownSeason)?
				},
			};
			Ok((season_id, season))
		}

		fn ensure_ownership(
			player: &T::AccountId,
			avatar_id: &AvatarIdOf<T>,
		) -> Result<Avatar, DispatchError> {
			let (owner, avatar) = Self::avatars(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			ensure!(player == &owner, Error::<T>::Ownership);
			Ok(avatar)
		}

		pub(crate) fn ensure_for_mint(
			player: &T::AccountId,
			mint_type: &MintType,
		) -> Result<MintCount, DispatchError> {
			let SeasonStatus { active, early, early_ended, .. } = Self::current_season_status();
			let free_mints = Self::accounts(player).free_mints;
			let is_whitelisted = free_mints > Zero::zero();
			let is_free_mint = mint_type == &MintType::Free;
			ensure!(!early_ended || is_free_mint, Error::<T>::PrematureSeasonEnd);
			ensure!(
				active || early && is_whitelisted || early && is_free_mint,
				Error::<T>::SeasonClosed
			);
			Ok(free_mints)
		}

		fn ensure_for_forge(
			player: &T::AccountId,
			leader_id: &AvatarIdOf<T>,
			sacrifice_ids: &[AvatarIdOf<T>],
			season_id: &SeasonId,
			season: &SeasonOf<T>,
		) -> Result<(Avatar, BTreeSet<AvatarIdOf<T>>, Vec<Avatar>), DispatchError> {
			let sacrifice_count = sacrifice_ids.len() as u8;
			ensure!(sacrifice_count >= season.min_sacrifices, Error::<T>::TooFewSacrifices);
			ensure!(sacrifice_count <= season.max_sacrifices, Error::<T>::TooManySacrifices);
			ensure!(!sacrifice_ids.contains(leader_id), Error::<T>::LeaderSacrificed);
			ensure!(
				sacrifice_ids.iter().all(|id| Self::ensure_for_trade(id).is_err()),
				Error::<T>::AvatarInTrade
			);
			ensure!(Self::ensure_for_trade(leader_id).is_err(), Error::<T>::AvatarInTrade);
			Self::ensure_unlocked(leader_id)?;

			let deduplicated_sacrifice_ids = sacrifice_ids.iter().copied().collect::<BTreeSet<_>>();
			let sacrifices = deduplicated_sacrifice_ids
				.iter()
				.map(|id| {
					let avatar = Self::ensure_ownership(player, id)?;
					ensure!(avatar.season_id == *season_id, Error::<T>::IncorrectAvatarSeason);
					Self::ensure_unlocked(id)?;
					Ok(avatar)
				})
				.collect::<Result<Vec<Avatar>, DispatchError>>()?;

			let leader = Self::ensure_ownership(player, leader_id)?;
			ensure!(leader.season_id == *season_id, Error::<T>::IncorrectAvatarSeason);

			Ok((leader, deduplicated_sacrifice_ids, sacrifices))
		}

		fn ensure_for_trade(
			avatar_id: &AvatarIdOf<T>,
		) -> Result<(T::AccountId, BalanceOf<T>), DispatchError> {
			let price = Self::trade(avatar_id).ok_or(Error::<T>::UnknownAvatarForSale)?;
			let (seller, _) = Self::avatars(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			Ok((seller, price))
		}

		fn ensure_unlocked(avatar_id: &AvatarIdOf<T>) -> Result<(), DispatchError> {
			ensure!(!LockedAvatars::<T>::contains_key(avatar_id), Error::<T>::AvatarLocked);
			Ok(())
		}

		fn start_season(
			weight: &mut Weight,
			block_number: T::BlockNumber,
			season_id: SeasonId,
			season: &SeasonOf<T>,
		) {
			let is_current_season_active = Self::current_season_status().active;
			weight.saturating_accrue(T::DbWeight::get().reads(1));

			if !is_current_season_active {
				CurrentSeasonStatus::<T>::mutate(|status| {
					status.early = season.is_early(block_number);
					status.active = season.is_active(block_number);

					if season.is_active(block_number) {
						Self::deposit_event(Event::SeasonStarted(season_id));
					}
				});

				weight.saturating_accrue(T::DbWeight::get().writes(1));
			}
		}

		fn finish_season(weight: &mut Weight, block_number: T::BlockNumber, season_id: SeasonId) {
			let next_season_id = season_id.saturating_add(1);

			CurrentSeasonStatus::<T>::mutate(|status| {
				status.early = false;
				status.active = false;
				status.early_ended = false;
				status.max_tier_avatars = Zero::zero();
			});
			CurrentSeasonId::<T>::put(next_season_id);
			Self::deposit_event(Event::SeasonFinished(season_id));
			weight.saturating_accrue(T::DbWeight::get().writes(1));

			if let Some(next_season) = Self::seasons(next_season_id) {
				Self::start_season(weight, block_number, next_season_id, &next_season);
			}
		}
	}
}
