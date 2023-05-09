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

#![feature(variant_count)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
	traits::{
		AccountIdConversion, CheckedAdd, CheckedSub, Hash, Saturating, TrailingZeroInput,
		UniqueSaturatedInto, Zero,
	},
	ArithmeticError,
};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type SeasonOf<T> = Season<BlockNumberFor<T>>;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
	pub(crate) type BoundedAvatarIdsOf<T> = BoundedVec<AvatarIdOf<T>, MaxAvatarsPerPlayer>;
	pub(crate) type GlobalConfigOf<T> = GlobalConfig<BalanceOf<T>, BlockNumberFor<T>>;
	pub(crate) type CollectionIdOf<T> = <<T as Config>::NftHandler as NftHandler<
		AccountIdOf<T>,
		AvatarIdOf<T>,
		Avatar,
	>>::CollectionId;

	pub(crate) const MAX_PERCENTAGE: u8 = 100;

	#[pallet::pallet]
	#[pallet::storage_version(migration::STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type NftHandler: NftHandler<Self::AccountId, Self::Hash, Avatar>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type Treasurer<T: Config> = StorageMap<_, Identity, SeasonId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type CurrentSeasonStatus<T: Config> = StorageValue<_, SeasonStatus, ValueQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, SeasonOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type Treasury<T: Config> = StorageMap<_, Identity, SeasonId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type GlobalConfigs<T: Config> = StorageValue<_, GlobalConfigOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type Avatars<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, (T::AccountId, Avatar)>;

	#[pallet::storage]
	pub type Owners<T: Config> =
		StorageMap<_, Identity, T::AccountId, BoundedAvatarIdsOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type LockedAvatars<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, ()>;

	#[pallet::storage]
	pub type CollectionId<T: Config> = StorageValue<_, CollectionIdOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type Accounts<T: Config> =
		StorageMap<_, Identity, T::AccountId, AccountInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	pub type SeasonStats<T: Config> =
		StorageDoubleMap<_, Identity, SeasonId, Identity, T::AccountId, SeasonInfo, ValueQuery>;

	#[pallet::storage]
	pub type Trade<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ServiceAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type Preparation<T: Config> = StorageMap<_, Identity, AvatarIdOf<T>, IpfsUrl, OptionQuery>;

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
			CurrentSeasonStatus::<T>::put(SeasonStatus {
				season_id: 1,
				early: Default::default(),
				active: Default::default(),
				early_ended: Default::default(),
				max_tier_avatars: Default::default(),
			});
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
				nft_transfer: NftTransferConfig {
					open: true,
					prepare_fee: 5_000_000_000_000_u64.unique_saturated_into(), // 5 BAJU
				},
			});
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An organizer has been set.
		OrganizerSet { organizer: T::AccountId },
		/// A service account has been set.
		ServiceAccountSet { service_account: T::AccountId },
		/// A collection ID has been set.
		CollectionIdSet { collection_id: CollectionIdOf<T> },
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
		AvatarsForged { avatar_ids: Vec<(AvatarIdOf<T>, UpgradedComponents)> },
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
		AvatarLocked { avatar_id: AvatarIdOf<T> },
		/// Avatar unlocked.
		AvatarUnlocked { avatar_id: AvatarIdOf<T> },
		/// Storage tier has been upgraded.
		StorageTierUpgraded,
		/// Avatar prepared.
		PreparedAvatar { avatar_id: AvatarIdOf<T> },
		/// Avatar unprepared.
		UnpreparedAvatar { avatar_id: AvatarIdOf<T> },
		/// IPFS URL prepared.
		PreparedIpfsUrl { url: IpfsUrl },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// There is no collection ID set for NFT handler.
		CollectionIdNotSet,
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
		/// The preparation doesn't exist.
		UnknownPreparation,
		/// The season ID of a season to create is not sequential.
		NonSequentialSeasonId,
		/// The sum of the given single mint probabilities overflows.
		SingleMintProbsOverflow,
		/// The sum of the given batch mint probabilities overflows.
		BatchMintProbsOverflow,
		/// Rarity percentages don't add up to 100
		IncorrectRarityPercentages,
		/// Max tier is achievable through forging only. Therefore the number of rarity percentages
		/// must be less than that of tiers for a season.
		TooManyRarityPercentages,
		/// The given base probability is too high. It must be less than 100.
		BaseProbTooHigh,
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
		/// NFT transfer is not available at the moment.
		NftTransferClosed,
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
		/// Incorrect data.
		IncorrectData,
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
		/// The player has not enough balance available.
		InsufficientBalance,
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
		/// Tried to forge avatars with different DNA versions.
		IncompatibleAvatarVersions,
		/// Tried transferring to his or her own account.
		CannotTransferToSelf,
		/// Tried claiming treasury during a season.
		CannotClaimDuringSeason,
		/// Tried claiming treasury which is zero.
		CannotClaimZero,
		/// The components tried to forge were not compatible.
		IncompatibleForgeComponents,
		/// The amount of sacrifices is not sufficient for forging.
		InsufficientSacrifices,
		/// The amount of sacrifices is too much for forging.
		ExcessiveSacrifices,
		/// Tried to prepare an already prepared avatar.
		AlreadyPrepared,
		/// Tried to prepare an IPFS URL for an avatar, that is not yet prepared.
		NotPrepared,
		/// No service account has been set.
		NoServiceAccount,
		/// Tried to prepare an IPFS URL for an avatar with an empty URL.
		EmptyIpfsUrl,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let current_season_id = CurrentSeasonStatus::<T>::get().season_id;
			let mut weight = T::DbWeight::get().reads(1);

			if let Some(current_season) = Seasons::<T>::get(current_season_id) {
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
			let GlobalConfig { transfer, .. } = GlobalConfigs::<T>::get();
			let from = match Self::ensure_organizer(origin.clone()) {
				Ok(organizer) => organizer,
				_ => {
					ensure!(transfer.open, Error::<T>::TransferClosed);
					ensure_signed(origin)?
				},
			};
			ensure!(from != to, Error::<T>::CannotTransferToSelf);
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);
			Self::ensure_unlocked(&avatar_id)?;
			Self::ensure_unprepared(&avatar_id)?;

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

			let GlobalConfig { transfer, .. } = GlobalConfigs::<T>::get();
			ensure!(how_many >= transfer.min_free_mint_transfer, Error::<T>::TooLowFreeMints);
			let sender_free_mints = Accounts::<T>::get(&from)
				.free_mints
				.checked_sub(
					how_many
						.checked_add(transfer.free_mint_transfer_fee)
						.ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(Error::<T>::InsufficientFreeMints)?;
			let dest_free_mints = Accounts::<T>::get(&to)
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
			ensure!(GlobalConfigs::<T>::get().trade.open, Error::<T>::TradeClosed);
			Self::ensure_ownership(&seller, &avatar_id)?;
			Self::ensure_unlocked(&avatar_id)?;
			Self::ensure_unprepared(&avatar_id)?;
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
			ensure!(GlobalConfigs::<T>::get().trade.open, Error::<T>::TradeClosed);
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
			let GlobalConfig { trade, .. } = GlobalConfigs::<T>::get();
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
			let storage_tier = Accounts::<T>::get(&player).storage_tier;
			ensure!(storage_tier != StorageTier::Max, Error::<T>::MaxStorageTierReached);

			let upgrade_fee = GlobalConfigs::<T>::get().account.storage_upgrade_fee;
			T::Currency::withdraw(&player, upgrade_fee, WithdrawReasons::FEE, AllowDeath)?;

			let season_id = CurrentSeasonStatus::<T>::get().season_id;
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

		/// Claim treasury of a season.
		///
		/// The origin of this call must be signed by a treasurer account associated with the given
		/// season ID. The treasurer of a season can claim the season's associated treasury once the
		/// season finishes.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::claim_treasury())]
		pub fn claim_treasury(origin: OriginFor<T>, season_id: SeasonId) -> DispatchResult {
			let maybe_treasurer = ensure_signed(origin)?;
			let treasurer = Treasurer::<T>::get(season_id).ok_or(Error::<T>::UnknownTreasurer)?;
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

			T::Currency::transfer(&Self::treasury_account_id(), &treasurer, amount, AllowDeath)?;
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

		/// Set the collection ID to associate avatars with.
		///
		/// Externally created collection ID for avatars must be set in the `CollectionId` storage
		/// to serve as a lookup for locking and unlocking avatars as NFTs.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(14)]
		#[pallet::weight(T::WeightInfo::set_collection_id())]
		pub fn set_collection_id(
			origin: OriginFor<T>,
			collection_id: CollectionIdOf<T>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			CollectionId::<T>::put(&collection_id);
			Self::deposit_event(Event::CollectionIdSet { collection_id });
			Ok(())
		}

		/// Locks an avatar to be tokenized as an NFT.
		///
		/// The origin of this call must specify an avatar, owned by the origin, to prevent it from
		/// forging, trading and transferring it to other players. When successful, the ownership of
		/// the avatar is transferred from the player to the pallet's technical account.
		///
		/// Locking an avatar allows for new
		/// ways of interacting with it currently under development.
		///
		/// Weight: `O(n)` where:
		/// - `n = max avatars per player`
		#[pallet::call_index(15)]
		#[pallet::weight(T::WeightInfo::lock_avatar(MaxAvatarsPerPlayer::get()))]
		pub fn lock_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let avatar = Self::ensure_ownership(&player, &avatar_id)?;
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);
			ensure!(GlobalConfigs::<T>::get().nft_transfer.open, Error::<T>::NftTransferClosed);
			Self::ensure_unlocked(&avatar_id)?;
			ensure!(Preparation::<T>::contains_key(avatar_id), Error::<T>::NotPrepared);

			Self::do_transfer_avatar(&player, &Self::technical_account_id(), &avatar_id)?;

			let collection_id = CollectionId::<T>::get().ok_or(Error::<T>::CollectionIdNotSet)?;
			let url = Preparation::<T>::take(avatar_id).ok_or(Error::<T>::UnknownPreparation)?;
			T::NftHandler::store_as_nft(player, collection_id, avatar_id, avatar, url.to_vec())?;

			LockedAvatars::<T>::insert(avatar_id, ());
			Self::deposit_event(Event::AvatarLocked { avatar_id });
			Ok(())
		}

		/// Unlocks an avatar removing its NFT representation.
		///
		/// The origin of this call must specify an avatar, owned and locked by the origin, to allow
		/// forging, trading and transferring it to other players. When successful, the ownership of
		/// the avatar is transferred from the pallet's technical account back to the player and its
		/// existing NFT representation is destroyed.
		///
		/// Weight: `O(n)` where:
		/// - `n = max avatars per player`
		#[pallet::call_index(16)]
		#[pallet::weight(T::WeightInfo::unlock_avatar(MaxAvatarsPerPlayer::get()))]
		pub fn unlock_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let _ = Self::ensure_ownership(&Self::technical_account_id(), &avatar_id)?;
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);
			ensure!(GlobalConfigs::<T>::get().nft_transfer.open, Error::<T>::NftTransferClosed);
			ensure!(LockedAvatars::<T>::contains_key(avatar_id), Error::<T>::AvatarUnlocked);

			Self::do_transfer_avatar(&Self::technical_account_id(), &player, &avatar_id)?;
			let collection_id = CollectionId::<T>::get().ok_or(Error::<T>::CollectionIdNotSet)?;
			let _ = T::NftHandler::recover_from_nft(player, collection_id, avatar_id)?;

			LockedAvatars::<T>::remove(avatar_id);
			Self::deposit_event(Event::AvatarUnlocked { avatar_id });
			Ok(())
		}

		/// Fix the variation of an avatar's DNA affected by a bug.
		///
		/// A trivial bug was introduced to incorrectly represent the 3rd component's variation,
		/// which should be the same as that of the 2nd. Instead of fixing the DNAs via migration,
		/// we allow players freedom to choose to fix these affected DNAs since they might prefer
		/// the existing looks.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(17)]
		#[pallet::weight(T::WeightInfo::fix_variation())]
		pub fn fix_variation(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let mut avatar = Self::ensure_ownership(&account, &avatar_id)?;

			// Update the variation of the 3rd component to be the same as that of the 2nd by
			// copying the rightmost 4 bits of dna[1] to the dna[2]
			avatar.dna[2] = (avatar.dna[2] & 0b1111_0000) | (avatar.dna[1] & 0b0000_1111);

			Avatars::<T>::insert(avatar_id, (account, avatar));

			Ok(())
		}

		/// Set a service account.
		///
		/// The origin of this call must be root. A service account has sufficient privilege to call
		/// the `prepare_ipfs` extrinsic.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(18)]
		#[pallet::weight(T::WeightInfo::set_service_account())]
		pub fn set_service_account(
			origin: OriginFor<T>,
			service_account: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			ServiceAccount::<T>::put(&service_account);
			Self::deposit_event(Event::ServiceAccountSet { service_account });
			Ok(())
		}

		/// Prepare an avatar to be uploaded to IPFS.
		///
		/// The origin of this call must specify an avatar, owned by the origin, to display the
		/// intention of uploading it to an IPFS storage. When successful, the `PreparedAvatar`
		/// event is emitted to be picked up by our external service that interacts with the IPFS.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(19)]
		#[pallet::weight(T::WeightInfo::prepare_avatar())]
		pub fn prepare_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let _ = Self::ensure_ownership(&player, &avatar_id)?;
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);
			ensure!(GlobalConfigs::<T>::get().nft_transfer.open, Error::<T>::NftTransferClosed);
			Self::ensure_unlocked(&avatar_id)?;
			Self::ensure_unprepared(&avatar_id)?;

			let service_account = ServiceAccount::<T>::get().ok_or(Error::<T>::NoServiceAccount)?;
			let prepare_fee = GlobalConfigs::<T>::get().nft_transfer.prepare_fee;
			T::Currency::transfer(&player, &service_account, prepare_fee, AllowDeath)?;

			Preparation::<T>::insert(avatar_id, IpfsUrl::default());
			Self::deposit_event(Event::PreparedAvatar { avatar_id });
			Ok(())
		}

		/// Unprepare an avatar to be detached from IPFS.
		///
		/// The origin of this call must specify an avatar, owned by the origin, that is undergoing
		/// the IPFS upload process.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(20)]
		#[pallet::weight(T::WeightInfo::unprepare_avatar())]
		pub fn unprepare_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let _ = Self::ensure_ownership(&player, &avatar_id)?;
			ensure!(GlobalConfigs::<T>::get().nft_transfer.open, Error::<T>::NftTransferClosed);
			ensure!(Preparation::<T>::contains_key(avatar_id), Error::<T>::NotPrepared);

			Preparation::<T>::remove(avatar_id);
			Self::deposit_event(Event::UnpreparedAvatar { avatar_id });
			Ok(())
		}

		/// Prepare IPFS for an avatar.
		///
		/// The origin of this call must be signed by the service account to upload the given avatar
		/// to an IPFS storage and stores its CID. A third-party service subscribes for the
		/// `PreparedAvatar` events which triggers preparing assets, their upload to IPFS and
		/// storing their CIDs.
		//
		/// Weight: `O(1)`
		#[pallet::call_index(21)]
		#[pallet::weight(T::WeightInfo::prepare_ipfs())]
		pub fn prepare_ipfs(
			origin: OriginFor<T>,
			avatar_id: AvatarIdOf<T>,
			url: IpfsUrl,
		) -> DispatchResult {
			let _ = Self::ensure_service_account(origin)?;
			ensure!(GlobalConfigs::<T>::get().nft_transfer.open, Error::<T>::NftTransferClosed);
			ensure!(Preparation::<T>::contains_key(avatar_id), Error::<T>::NotPrepared);
			ensure!(!url.is_empty(), Error::<T>::EmptyIpfsUrl);
			Preparation::<T>::insert(avatar_id, &url);
			Self::deposit_event(Event::PreparedIpfsUrl { url });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the treasury.
		pub fn treasury_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		/// The account ID of the treasury.
		pub fn technical_account_id() -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(b"technical")
		}

		pub(crate) fn deposit_into_treasury(season_id: &SeasonId, amount: BalanceOf<T>) {
			Treasury::<T>::mutate(season_id, |bal| bal.saturating_accrue(amount));
			T::Currency::deposit_creating(&Self::treasury_account_id(), amount);
		}

		/// Check that the origin is an organizer account.
		pub(crate) fn ensure_organizer(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(maybe_organizer)
		}

		pub(crate) fn ensure_service_account(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_sa = ensure_signed(origin)?;
			let existing_sa = ServiceAccount::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_sa == existing_sa, DispatchError::BadOrigin);
			Ok(maybe_sa)
		}

		/// Validates a new season.
		pub(crate) fn ensure_season(
			season_id: &SeasonId,
			mut season: SeasonOf<T>,
		) -> Result<SeasonOf<T>, DispatchError> {
			season.validate::<T>()?;

			let prev_season_id = season_id.checked_sub(&1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(&1).ok_or(ArithmeticError::Overflow)?;

			if prev_season_id > 0 {
				let prev_season =
					Seasons::<T>::get(prev_season_id).ok_or(Error::<T>::NonSequentialSeasonId)?;
				ensure!(prev_season.end < season.early_start, Error::<T>::EarlyStartTooEarly);
			}
			if let Some(next_season) = Seasons::<T>::get(next_season_id) {
				ensure!(season.end < next_season.early_start, Error::<T>::SeasonEndTooLate);
			}
			Ok(season)
		}

		#[inline]
		pub(crate) fn random_hash(phrase: &[u8], who: &T::AccountId) -> T::Hash {
			let (seed, _) = T::Randomness::random(phrase);
			let seed = T::Hash::decode(&mut TrailingZeroInput::new(seed.as_ref()))
				.expect("input is padded with zeroes; qed");
			let nonce = frame_system::Pallet::<T>::account_nonce(who);
			frame_system::Pallet::<T>::inc_account_nonce(who);
			(seed, &who, nonce.encode()).using_encoded(T::Hashing::hash)
		}

		/// Mint a new avatar.
		pub(crate) fn do_mint(player: &T::AccountId, mint_option: &MintOption) -> DispatchResult {
			Self::ensure_for_mint(player, mint_option)?;

			let season_id = CurrentSeasonStatus::<T>::get().season_id;
			let season = Seasons::<T>::get(season_id).ok_or(Error::<T>::UnknownSeason)?;
			let mint_output =
				mint_option.mint_version.with_minter(|minter: Box<dyn Minter<T>>| {
					minter.mint_avatar_set(player, &season_id, &season, mint_option)
				})?;

			// Add generated avatars from minter to storage
			let generated_avatar_ids = mint_output
				.into_iter()
				.map(|(minted_avatar_id, minted_avatar)| {
					Self::try_add_avatar_to(player, minted_avatar_id, minted_avatar)?;
					Ok(minted_avatar_id)
				})
				.collect::<Result<Vec<AvatarIdOf<T>>, DispatchError>>()?;

			let GlobalConfig { mint, .. } = GlobalConfigs::<T>::get();
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
						account.free_mints = account
							.free_mints
							.checked_sub(fee)
							.ok_or(Error::<T>::InsufficientFreeMints)?;
						Ok(())
					})?;
				},
			};

			Accounts::<T>::try_mutate(player, |AccountInfo { stats, .. }| -> DispatchResult {
				let current_block = <frame_system::Pallet<T>>::block_number();
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
			let GlobalConfig { forge, .. } = GlobalConfigs::<T>::get();
			ensure!(forge.open, Error::<T>::ForgeClosed);

			let (season_id, season) = Self::current_season_with_id()?;
			let (leader, sacrifice_ids, sacrifices) =
				Self::ensure_for_forge(player, leader_id, sacrifice_ids, &season_id, &season)?;

			let forger: Box<dyn Forger<T>> = leader.version.get_forger();
			let input_leader = (*leader_id, leader);
			let input_sacrifices =
				sacrifice_ids.into_iter().zip(sacrifices).collect::<Vec<ForgeItem<T>>>();
			let (output_leader, output_other) = forger.forge_with(
				player,
				season_id,
				&season,
				input_leader.clone(),
				input_sacrifices,
			)?;

			Self::process_leader_forge_output(player, &season, input_leader, output_leader)?;
			Self::process_other_forge_outputs(player, &season, output_other)?;
			Self::update_forging_statistics_for_player(player, season_id)?;
			Ok(())
		}

		fn do_transfer_avatar(
			from: &T::AccountId,
			to: &T::AccountId,
			avatar_id: &AvatarIdOf<T>,
		) -> DispatchResult {
			let mut from_avatar_ids = Owners::<T>::get(from);
			from_avatar_ids.retain(|existing_avatar_id| existing_avatar_id != avatar_id);

			let mut to_avatar_ids = Owners::<T>::get(to);
			to_avatar_ids
				.try_push(*avatar_id)
				.map_err(|_| Error::<T>::MaxOwnershipReached)?;
			ensure!(
				to_avatar_ids.len() <= Accounts::<T>::get(to).storage_tier as usize,
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
			let mut current_status = CurrentSeasonStatus::<T>::get();
			let season = match Seasons::<T>::get(current_status.season_id) {
				Some(season) if current_status.is_in_season() => season,
				_ => {
					if current_status.season_id > 1 {
						current_status.season_id.saturating_dec();
					}
					Seasons::<T>::get(current_status.season_id).ok_or(Error::<T>::UnknownSeason)?
				},
			};
			Ok((current_status.season_id, season))
		}

		fn ensure_ownership(
			player: &T::AccountId,
			avatar_id: &AvatarIdOf<T>,
		) -> Result<Avatar, DispatchError> {
			let (owner, avatar) = Avatars::<T>::get(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			ensure!(player == &owner, Error::<T>::Ownership);
			Ok(avatar)
		}

		pub(crate) fn ensure_for_mint(
			player: &T::AccountId,
			mint_option: &MintOption,
		) -> DispatchResult {
			let GlobalConfig { mint, .. } = GlobalConfigs::<T>::get();
			ensure!(mint.open, Error::<T>::MintClosed);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let last_block = Accounts::<T>::get(player).stats.mint.last;
			if !last_block.is_zero() {
				ensure!(current_block >= last_block + mint.cooldown, Error::<T>::MintCooldown);
			}

			let SeasonStatus { active, early, early_ended, .. } = CurrentSeasonStatus::<T>::get();
			let free_mints = Accounts::<T>::get(player).free_mints;
			let is_whitelisted = free_mints > Zero::zero();
			let is_free_mint = mint_option.mint_type == MintType::Free;
			ensure!(!early_ended || is_free_mint, Error::<T>::PrematureSeasonEnd);
			ensure!(active || early && (is_whitelisted || is_free_mint), Error::<T>::SeasonClosed);

			match mint_option.mint_type {
				MintType::Normal => {
					let fee = mint.fees.fee_for(&mint_option.count);
					T::Currency::free_balance(player)
						.checked_sub(&fee)
						.ok_or(Error::<T>::InsufficientBalance)?;
				},
				MintType::Free => {
					let fee = (mint_option.count as MintCount)
						.saturating_mul(mint.free_mint_fee_multiplier);
					free_mints.checked_sub(fee).ok_or(Error::<T>::InsufficientFreeMints)?;
				},
			};

			let new_count = Owners::<T>::get(player).len() + mint_option.count as usize;
			let max_count = Accounts::<T>::get(player).storage_tier as usize;
			ensure!(new_count <= max_count, Error::<T>::MaxOwnershipReached);
			Ok(())
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
			Self::ensure_unprepared(leader_id)?;

			let leader = Self::ensure_ownership(player, leader_id)?;
			ensure!(leader.season_id == *season_id, Error::<T>::IncorrectAvatarSeason);

			let deduplicated_sacrifice_ids = sacrifice_ids.iter().copied().collect::<BTreeSet<_>>();
			let sacrifices = deduplicated_sacrifice_ids
				.iter()
				.map(|id| {
					let avatar = Self::ensure_ownership(player, id)?;
					ensure!(avatar.season_id == *season_id, Error::<T>::IncorrectAvatarSeason);
					ensure!(
						avatar.version == leader.version,
						Error::<T>::IncompatibleAvatarVersions
					);
					Self::ensure_unlocked(id)?;
					Self::ensure_unprepared(id)?;
					Ok(avatar)
				})
				.collect::<Result<Vec<Avatar>, DispatchError>>()?;

			Ok((leader, deduplicated_sacrifice_ids, sacrifices))
		}

		#[inline]
		fn process_leader_forge_output(
			player: &AccountIdOf<T>,
			season: &SeasonOf<T>,
			input_leader: ForgeItem<T>,
			output_leader: LeaderForgeOutput<T>,
		) -> DispatchResult {
			match output_leader {
				LeaderForgeOutput::Forged((leader_id, leader), upgraded_components) => {
					let prev_leader_tier = input_leader.1.min_tier();
					let after_leader_tier = leader.min_tier();
					let max_tier = season.max_tier() as u8;

					if prev_leader_tier != max_tier && after_leader_tier == max_tier {
						CurrentSeasonStatus::<T>::mutate(|status| {
							status.max_tier_avatars.saturating_inc();
							if status.max_tier_avatars == season.max_tier_forges {
								status.early_ended = true;
							}
						});
					}

					Avatars::<T>::insert(leader_id, (player, leader));

					// TODO: May change in the future
					Self::deposit_event(Event::AvatarsForged {
						avatar_ids: vec![(leader_id, upgraded_components)],
					});
				},
				LeaderForgeOutput::Consumed(leader_id) =>
					Self::remove_avatar_from(player, &leader_id),
			}

			Ok(())
		}

		#[inline]
		fn process_other_forge_outputs(
			player: &AccountIdOf<T>,
			_season: &SeasonOf<T>,
			other_outputs: Vec<ForgeOutput<T>>,
		) -> DispatchResult {
			let mut minted_avatars: Vec<AvatarIdOf<T>> = Vec::with_capacity(0);
			let mut forged_avatars: Vec<(AvatarIdOf<T>, UpgradedComponents)> =
				Vec::with_capacity(0);

			for output in other_outputs {
				match output {
					ForgeOutput::Forged((avatar_id, avatar), upgraded_components) => {
						Avatars::<T>::insert(avatar_id, (player, avatar));
						forged_avatars.push((avatar_id, upgraded_components));
					},
					ForgeOutput::Minted(avatar) => {
						let avatar_id = Self::random_hash(b"create_avatar", player);
						Self::try_add_avatar_to(player, avatar_id, avatar)?;
						minted_avatars.push(avatar_id);
					},
					ForgeOutput::Consumed(avatar_id) =>
						Self::remove_avatar_from(player, &avatar_id),
				}
			}

			// TODO: May be removed in the future
			if !minted_avatars.is_empty() {
				Self::deposit_event(Event::AvatarsMinted { avatar_ids: minted_avatars });
			}

			// TODO: May change in the future
			if !forged_avatars.is_empty() {
				Self::deposit_event(Event::AvatarsForged { avatar_ids: forged_avatars });
			}

			Ok(())
		}

		#[inline]
		fn update_forging_statistics_for_player(
			player: &AccountIdOf<T>,
			season_id: SeasonId,
		) -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();

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

			SeasonStats::<T>::mutate(season_id, player, |info| {
				info.forged.saturating_inc();
			});

			Ok(())
		}

		#[inline]
		fn try_add_avatar_to(
			player: &AccountIdOf<T>,
			avatar_id: AvatarIdOf<T>,
			avatar: Avatar,
		) -> DispatchResult {
			Avatars::<T>::insert(avatar_id, (player, avatar));
			Owners::<T>::try_append(&player, avatar_id)
				.map_err(|_| Error::<T>::MaxOwnershipReached)?;
			Ok(())
		}

		#[inline]
		fn remove_avatar_from(player: &AccountIdOf<T>, avatar_id: &AvatarIdOf<T>) {
			Avatars::<T>::remove(avatar_id);
			Owners::<T>::mutate(player, |avatars| {
				avatars.retain(|id| id != avatar_id);
			});
		}

		fn ensure_for_trade(
			avatar_id: &AvatarIdOf<T>,
		) -> Result<(T::AccountId, BalanceOf<T>), DispatchError> {
			let price = Trade::<T>::get(avatar_id).ok_or(Error::<T>::UnknownAvatarForSale)?;
			let (seller, _) = Avatars::<T>::get(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			Ok((seller, price))
		}

		fn ensure_unlocked(avatar_id: &AvatarIdOf<T>) -> DispatchResult {
			ensure!(!LockedAvatars::<T>::contains_key(avatar_id), Error::<T>::AvatarLocked);
			Ok(())
		}

		fn ensure_unprepared(avatar_id: &AvatarIdOf<T>) -> DispatchResult {
			ensure!(!Preparation::<T>::contains_key(avatar_id), Error::<T>::AlreadyPrepared);
			Ok(())
		}

		fn start_season(
			weight: &mut Weight,
			block_number: T::BlockNumber,
			season_id: SeasonId,
			season: &SeasonOf<T>,
		) {
			let is_current_season_active = CurrentSeasonStatus::<T>::get().active;
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
				status.season_id = next_season_id;
				status.early = false;
				status.active = false;
				status.early_ended = false;
				status.max_tier_avatars = Zero::zero();
			});
			Self::deposit_event(Event::SeasonFinished(season_id));
			weight.saturating_accrue(T::DbWeight::get().writes(1));

			if let Some(next_season) = Seasons::<T>::get(next_season_id) {
				Self::start_season(weight, block_number, next_season_id, &next_season);
			}
		}
	}
}
