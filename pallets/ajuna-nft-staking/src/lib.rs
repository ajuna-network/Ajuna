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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub mod contracts;
pub mod weights;

use codec::HasCompact;
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::nonfungibles_v2::{Create, Destroy, Inspect, Mutate, Transfer},
		BalanceStatus, Currency, ExistenceRequirement, Get, ReservableCurrency,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Saturating};
use sp_std::prelude::*;

pub use contracts::*;
pub use pallet::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub(crate) type CollectionIdOf<T> = <T as Config>::CollectionId;
	pub(crate) type ItemIdOf<T> = <T as Config>::ItemId;
	pub(crate) type ContractAttributeKeyOf<T> = <T as Config>::ContractAttributeKey;
	pub(crate) type ContractAttributeValueOf<T> = <T as Config>::ContractAttributeValue;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

	pub const MAXIMUM_CLAUSES_PER_CONTRACT: u32 = 10;

	pub(crate) type ContractOf<T> = Contract<
		BalanceOf<T>,
		CollectionIdOf<T>,
		ItemIdOf<T>,
		BlockNumberOf<T>,
		ContractAttributeKeyOf<T>,
		ContractAttributeValueOf<T>,
		MAXIMUM_CLAUSES_PER_CONTRACT,
	>;
	pub(crate) type StakedAssetsVecOf<T> =
		StakedAssetsVec<CollectionIdOf<T>, ItemIdOf<T>, MAXIMUM_CLAUSES_PER_CONTRACT>;
	pub(crate) type NftAddressOf<T> = NftAddress<CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type RewardOf<T> = Reward<BalanceOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;

	#[derive(
		Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, Default, Eq, PartialEq,
	)]
	pub struct GlobalConfig {
		pub pallet_locked: bool,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The NFT-staking's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Identifier for the collection of an Nft.
		type CollectionId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		/// Type that holds the specific configurations for a collection.
		type CollectionConfig: Copy
			+ Clone
			+ Default
			+ PartialEq
			+ Encode
			+ Decode
			+ MaxEncodedLen
			+ TypeInfo;

		/// Identifier for the individual instances of an Nft.
		type ItemId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ AtLeast32BitUnsigned;

		/// Type that holds the specific configurations for an item.
		type ItemConfig: Copy
			+ Clone
			+ Default
			+ PartialEq
			+ Encode
			+ Decode
			+ MaxEncodedLen
			+ TypeInfo;

		type NftHelper: Inspect<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Create<Self::AccountId, Self::CollectionConfig>
			+ Mutate<Self::AccountId, Self::ItemConfig>
			+ Destroy<Self::AccountId>
			+ Transfer<Self::AccountId>;

		/// The origin which may interact with the staking logic.
		type StakingOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// The minimal amount of tokens that can be rewarded in a staking contract.
		#[pallet::constant]
		type MinimumStakingTokenReward: Get<BalanceOf<Self>>;

		/// The configuration for the contract Nft collection
		#[pallet::constant]
		type ContractCollectionItemConfig: Get<Self::ItemConfig>;

		/// Type of the contract attributes keys, used on contract condition evaluation
		type ContractAttributeKey: Member
			+ Encode
			+ Decode
			+ Ord
			+ PartialOrd
			+ MaxEncodedLen
			+ TypeInfo
			+ sp_std::fmt::Display;

		/// Type of the contract attributes values, used on contract condition evaluation
		type ContractAttributeValue: Member
			+ Encode
			+ Decode
			+ Ord
			+ PartialOrd
			+ MaxEncodedLen
			+ TypeInfo;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Creator<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type GlobalConfigs<T: Config> = StorageValue<_, GlobalConfig, ValueQuery>;

	#[pallet::storage]
	pub type ActiveContracts<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, ContractOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ContractOwners<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, AccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ContractDurations<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, T::BlockNumber, OptionQuery>;

	#[pallet::storage]
	pub type ContractStakedAssets<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, StakedAssetsVecOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ContractCollectionId<T: Config> = StorageValue<_, CollectionIdOf<T>>;

	#[pallet::storage]
	pub type NextContractId<T: Config> = StorageValue<_, ItemIdOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An creator has been set.
		CreatorSet { creator: AccountIdOf<T> },
		/// The collection holding the staking contracts has been set.
		ContractCollectionSet { collection_id: T::CollectionId },
		/// The pallet's global config has been set.
		SetGlobalConfig { new_config: GlobalConfig },
		/// A new staking contract has been created.
		Created { creator: AccountIdOf<T>, contract_id: ItemIdOf<T> },
		/// A staking contract has been accepted.
		Accepted { accepted_by: AccountIdOf<T>, contract_id: ItemIdOf<T> },
		/// A staking contract has been claimed.
		Claimed { claimed_by: AccountIdOf<T>, contract_id: ItemIdOf<T>, reward: RewardOf<T> },
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// There is no account set as the creator
		CreatorNotSet,
		/// The given collection doesn't exist.
		UnknownCollection,
		/// The given contract collection doesn't exist.
		UnknownContractCollection,
		/// The given item doesn't exist.
		UnknownItem,
		/// The given collection belongs to someone else.
		Ownership,
		/// The pallet is currently locked and cannot be interacted with.
		PalletLocked,
		/// The treasury doesn't have enough funds to pay the contract rewards.
		TreasuryLacksFunds,
		/// Account doesn't have enough the minimum amount of funds necessary to contribute.
		AccountLacksFunds,
		/// The account that tried to take a staking contract failed to fulfill its conditions.
		ContractConditionsNotFulfilled,
		/// The contract reward is not valid. Either an invalid Nft or not enough tokens.
		InvalidContractReward,
		/// The account that tried to take a staking contract didn't own one or more of the
		/// staked assets.
		StakedAssetNotOwned,
		/// The account that tried to create a contract didn't actually own it's reward.
		ContractRewardNotOwned,
		/// The account that tried to redeemed a contract didn't own it
		ContractNotOwned,
		/// The contract has been already taken by another account
		ContractTakenByOther,
		/// The contract has been already taken by the account
		ContractAlreadyTaken,
		/// The contract is still active, so it cannot be redeemed
		ContractStillActive,
		/// The contract to be redeemed cannot be found
		ContractNotFound,
	}

	//SBP-M3 review: Please add documentation in each extrinsic
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::set_creator())]
		#[pallet::call_index(0)]
		pub fn set_creator(origin: OriginFor<T>, creator: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Creator::<T>::put(&creator);
			Self::deposit_event(Event::CreatorSet { creator });
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_contract_collection_id())]
		#[pallet::call_index(1)]
		pub fn set_contract_collection_id(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
		) -> DispatchResult {
			let creator = Self::ensure_creator(origin)?;
			Self::ensure_collection_ownership(&creator, &collection_id)?;
			ContractCollectionId::<T>::put(collection_id);
			Self::deposit_event(Event::ContractCollectionSet { collection_id });
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_global_config())]
		#[pallet::call_index(2)]
		pub fn set_global_config(origin: OriginFor<T>, new_config: GlobalConfig) -> DispatchResult {
			let _ = Self::ensure_creator(origin)?;
			GlobalConfigs::<T>::put(new_config);
			Self::deposit_event(Event::SetGlobalConfig { new_config });
			Ok(())
		}

		#[pallet::weight(
			T::WeightInfo::create_token_reward()
				.max(T::WeightInfo::create_nft_reward())
		)]
		#[pallet::call_index(4)]
		pub fn create(origin: OriginFor<T>, contract: ContractOf<T>) -> DispatchResult {
			let creator = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::lock_reward(&creator, &contract.reward)?;
			let contract_id = Self::create_contract_nft(&contract)?;
			Self::deposit_event(Event::<T>::Created { creator, contract_id });
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::accept())]
		#[pallet::call_index(5)]
		pub fn accept(
			origin: OriginFor<T>,
			contract_id: ItemIdOf<T>,
			staked_assets: StakedAssetsVecOf<T>,
		) -> DispatchResult {
			Self::ensure_pallet_unlocked()?;

			let account = T::StakingOrigin::ensure_origin(origin)?;

			Self::try_if_contract_can_be_taken_by(&account, &contract_id)?;

			let contract: ContractOf<T> =
				ActiveContracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(
				contract.evaluate_for::<T::NftHelper, T::AccountId>(&staked_assets),
				Error::<T>::ContractConditionsNotFulfilled
			);

			Self::try_transferring_staked_assets_ownership(
				&staked_assets,
				&account,
				&Self::treasury_account_id(),
				false,
			)?;

			Self::try_taking_ownership_of_contract(
				&contract_id,
				account.clone(),
				&contract,
				staked_assets,
			)?;

			Self::deposit_event(Event::<T>::Accepted { accepted_by: account, contract_id });

			Ok(())
		}

		#[pallet::weight(
			T::WeightInfo::claim_token_reward()
				.max(T::WeightInfo::claim_nft_reward())
		)]
		#[pallet::call_index(6)]
		pub fn claim(origin: OriginFor<T>, contract_id: ItemIdOf<T>) -> DispatchResult {
			Self::ensure_pallet_unlocked()?;

			let account = T::StakingOrigin::ensure_origin(origin)?;

			Self::try_checking_if_contract_can_be_redeemed(&account, &contract_id)?;

			let staked_assets =
				ContractStakedAssets::<T>::get(contract_id).ok_or(Error::<T>::ContractNotFound)?;

			Self::try_transferring_staked_assets_ownership(
				&staked_assets,
				&Self::treasury_account_id(),
				&account,
				true,
			)?;

			let reward = ActiveContracts::<T>::get(contract_id)
				.ok_or(Error::<T>::ContractNotFound)?
				.reward;

			Self::try_handing_over_contract_reward_to(&account, &reward)?;
			Self::try_closing_redeemed_contract(&contract_id, &account)?;

			Self::deposit_event(Event::<T>::Claimed { claimed_by: account, contract_id, reward });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account identifier of the treasury pot.
		pub fn treasury_account_id() -> AccountIdOf<T> {
			T::PalletId::get().into_sub_account_truncating(b"treasury")
		}

		/// Return the amount of money available in the treasury reserves.
		pub fn treasury_pot_reserve() -> BalanceOf<T> {
			T::Currency::free_balance(&Self::treasury_account_id())
		}

		fn ensure_creator(origin: OriginFor<T>) -> Result<AccountIdOf<T>, DispatchError> {
			let maybe_creator = ensure_signed(origin)?;
			let existing_creator = Creator::<T>::get().ok_or(Error::<T>::CreatorNotSet)?;
			ensure!(maybe_creator == existing_creator, DispatchError::BadOrigin);
			Ok(maybe_creator)
		}

		fn ensure_pallet_unlocked() -> DispatchResult {
			ensure!(!GlobalConfigs::<T>::get().pallet_locked, Error::<T>::PalletLocked);
			Ok(())
		}

		fn ensure_collection_ownership(
			who: &T::AccountId,
			collection_id: &CollectionIdOf<T>,
		) -> DispatchResult {
			let owner = T::NftHelper::collection_owner(collection_id)
				.ok_or(Error::<T>::UnknownCollection)?;
			ensure!(who == &owner, Error::<T>::Ownership);
			Ok(())
		}

		fn ensure_item_ownership(
			who: &T::AccountId,
			collection_id: &CollectionIdOf<T>,
			item_id: &ItemIdOf<T>,
		) -> DispatchResult {
			let owner =
				T::NftHelper::owner(collection_id, item_id).ok_or(Error::<T>::UnknownItem)?;
			ensure!(who == &owner, Error::<T>::Ownership);
			Ok(())
		}

		fn lock_reward(from: &T::AccountId, reward: &RewardOf<T>) -> DispatchResult {
			let dest = Self::treasury_account_id();
			match reward {
				Reward::Tokens(amount) =>
					T::Currency::transfer(from, &dest, *amount, ExistenceRequirement::AllowDeath),
				Reward::Nft(address) => {
					let NftAddress(collection_id, item_id) = address;
					Self::ensure_item_ownership(from, collection_id, item_id)?;
					T::NftHelper::transfer(collection_id, item_id, &dest)
				},
			}?;
			Ok(())
		}

		fn create_contract_nft(contract: &ContractOf<T>) -> Result<ItemIdOf<T>, DispatchError> {
			let collection_id =
				ContractCollectionId::<T>::get().ok_or(Error::<T>::UnknownContractCollection)?;
			let contract_id = NextContractId::<T>::get();
			T::NftHelper::mint_into(
				&collection_id,
				&contract_id,
				&Self::treasury_account_id(),
				&T::ContractCollectionItemConfig::get(),
				true,
			)?;

			ActiveContracts::<T>::insert(contract_id, contract);
			NextContractId::<T>::mutate(|id| id.saturating_inc());
			Ok(contract_id)
		}

		fn try_if_contract_can_be_taken_by(
			account: &AccountIdOf<T>,
			contract_id: &ItemIdOf<T>,
		) -> DispatchResult {
			if let Some(ref contract_owner) = ContractOwners::<T>::get(contract_id) {
				if contract_owner == account {
					Err(Error::<T>::ContractAlreadyTaken.into())
				} else {
					Err(Error::<T>::ContractTakenByOther.into())
				}
			} else {
				Ok(())
			}
		}

		fn try_transferring_staked_assets_ownership(
			assets: &StakedAssetsVecOf<T>,
			from: &AccountIdOf<T>,
			to: &AccountIdOf<T>,
			skip_ownership_check: bool,
		) -> DispatchResult {
			for asset in assets.iter() {
				ensure!(
					skip_ownership_check ||
						(T::NftHelper::owner(&asset.0, &asset.1).as_ref() == Some(from)),
					Error::<T>::StakedAssetNotOwned
				);

				T::NftHelper::transfer(&asset.0, &asset.1, to)?;
			}

			Ok(())
		}

		fn try_taking_ownership_of_contract(
			contract_id: &ItemIdOf<T>,
			new_owner: AccountIdOf<T>,
			contract: &ContractOf<T>,
			staked_assets: StakedAssetsVecOf<T>,
		) -> DispatchResult {
			let collection_id = ContractCollectionId::<T>::get()
				.expect("Contract collection id should not be empty");
			T::NftHelper::transfer(&collection_id, contract_id, &new_owner)?;

			ContractOwners::<T>::insert(contract_id, new_owner);

			let runs_until =
				<frame_system::Pallet<T>>::block_number().saturating_add(contract.duration);
			ContractDurations::<T>::insert(contract_id, runs_until);
			ContractStakedAssets::<T>::insert(contract_id, staked_assets);

			Ok(())
		}

		fn try_handing_over_contract_reward_to(
			account_to_reward: &AccountIdOf<T>,
			contract_reward: &RewardOf<T>,
		) -> DispatchResult {
			match contract_reward {
				RewardOf::<T>::Tokens(amount) => {
					ensure!(
						Self::treasury_pot_reserve() >= *amount,
						Error::<T>::TreasuryLacksFunds
					);
					T::Currency::repatriate_reserved(
						&Self::treasury_account_id(),
						account_to_reward,
						*amount,
						BalanceStatus::Free,
					)?;
				},
				RewardOf::<T>::Nft(asset) => {
					T::NftHelper::transfer(&asset.0, &asset.1, account_to_reward)?;
				},
			}

			Ok(())
		}

		fn try_checking_if_contract_can_be_redeemed(
			contract_redeemer: &AccountIdOf<T>,
			contract_id: &ItemIdOf<T>,
		) -> DispatchResult {
			ensure!(
				ContractOwners::<T>::get(contract_id).as_ref() == Some(contract_redeemer),
				Error::<T>::ContractNotOwned
			);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let contract_expiry =
				ContractDurations::<T>::get(contract_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(current_block >= contract_expiry, Error::<T>::ContractStillActive);

			Ok(())
		}

		fn try_closing_redeemed_contract(
			contract_id: &ItemIdOf<T>,
			contract_redeemer: &AccountIdOf<T>,
		) -> DispatchResult {
			ContractStakedAssets::<T>::remove(contract_id);
			ContractDurations::<T>::remove(contract_id);
			ContractOwners::<T>::remove(contract_id);
			ActiveContracts::<T>::remove(contract_id);

			let contract_collection = ContractCollectionId::<T>::get()
				.expect("Contract collection id should not be empty");
			T::NftHelper::burn(&contract_collection, contract_id, Some(contract_redeemer))?;

			Ok(())
		}
	}
}
