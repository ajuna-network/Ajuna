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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod contracts;
pub mod weights;

use sp_runtime::traits::{AccountIdConversion, Saturating};
use sp_std::prelude::*;

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Get, ReservableCurrency},
	PalletId,
};

pub use contracts::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::{weights::WeightInfo, *};
	use codec::HasCompact;
	use frame_support::traits::{
		tokens::nonfungibles_v2::{Create, Destroy, Inspect, Mutate, Transfer},
		BalanceStatus, ExistenceRequirement,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, One};

	pub(crate) type CollectionIdOf<T> = <T as Config>::CollectionId;
	pub(crate) type ItemIdOf<T> = <T as Config>::ItemId;
	pub(crate) type ContractItemIdOf<T> = ItemIdOf<T>;
	pub(crate) type ContractAttributeKeyOf<T> = <T as Config>::ContractAttributeKey;
	pub(crate) type ContractAttributeValueOf<T> = <T as Config>::ContractAttributeValue;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

	pub const MAXIMUM_CLAUSES_PER_CONTRACT: u32 = 10;

	pub(crate) type StakingContractOf<T> = StakingContract<
		BalanceOf<T>,
		CollectionIdOf<T>,
		ItemIdOf<T>,
		AccountIdOf<T>,
		BlockNumberOf<T>,
		ContractAttributeKeyOf<T>,
		ContractAttributeValueOf<T>,
		MAXIMUM_CLAUSES_PER_CONTRACT,
	>;
	pub(crate) type StakedAssetsVecOf<T> =
		StakedAssetsVec<CollectionIdOf<T>, ItemIdOf<T>, MAXIMUM_CLAUSES_PER_CONTRACT>;
	pub(crate) type NFTAddressOf<T> = NFTAddress<CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type StakingRewardOf<T> =
		StakingReward<BalanceOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Identifier for the collection of an NFT.
		type CollectionId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ AtLeast32BitUnsigned
			+ sp_std::fmt::Display
			+ sp_std::cmp::PartialOrd
			+ sp_std::cmp::Ord;

		/// Type that holds the specific configurations for a collection.
		type CollectionConfig: Copy
			+ Clone
			+ Default
			+ PartialEq
			+ Encode
			+ Decode
			+ MaxEncodedLen
			+ TypeInfo;

		/// Identifier for the individual instances of an NFT.
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

		type NFTHelper: Inspect<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Create<Self::AccountId, Self::CollectionConfig>
			+ Mutate<Self::AccountId, Self::ItemConfig>
			+ Destroy<Self::AccountId>
			+ Transfer<Self::AccountId>;

		/// The origin which may interact with the staking logic.
		type StakingOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type TreasuryPalletId: Get<PalletId>;

		/// The minimal amount of tokens that can be rewarded in a staking contract.
		#[pallet::constant]
		type MinimumStakingTokenReward: Get<BalanceOf<Self>>;

		/// The configuration for the contract NFT collection
		#[pallet::constant]
		type ContractCollectionConfig: Get<Self::CollectionConfig>;

		/// The configuration for the contract NFT collection
		#[pallet::constant]
		type ContractCollectionItemConfig: Get<Self::ItemConfig>;

		/// Type of the contract attributes keys, used on contract condition evaluation
		#[cfg(not(feature = "frame-benchmarking"))]
		type ContractAttributeKey: Member
			+ Encode
			+ Decode
			+ Ord
			+ PartialOrd
			+ MaxEncodedLen
			+ TypeInfo
			+ sp_std::fmt::Display;

		/// Type of the contract attributes keys, used on contract condition evaluation
		#[cfg(feature = "frame-benchmarking")]
		type ContractAttributeKey: Member
			+ Encode
			+ Decode
			+ Ord
			+ PartialOrd
			+ MaxEncodedLen
			+ TypeInfo
			+ sp_std::fmt::Display
			+ From<u32>;

		/// Type of the contract attributes values, used on contract condition evaluation
		#[cfg(not(feature = "frame-benchmarking"))]
		type ContractAttributeValue: Member
			+ Encode
			+ Decode
			+ Ord
			+ PartialOrd
			+ MaxEncodedLen
			+ TypeInfo;

		/// Type of the contract attributes values, used on contract condition evaluation
		#[cfg(feature = "frame-benchmarking")]
		type ContractAttributeValue: Member
			+ Encode
			+ Decode
			+ Ord
			+ PartialOrd
			+ MaxEncodedLen
			+ TypeInfo
			+ From<u64>;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn active_contracts)]
	pub type ActiveContracts<T: Config> =
		StorageMap<_, Identity, ContractItemIdOf<T>, StakingContractOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn contract_owners)]
	pub type ContractOwners<T: Config> =
		StorageMap<_, Identity, ContractItemIdOf<T>, AccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn contract_durations)]
	pub type ContractDurations<T: Config> =
		StorageMap<_, Identity, ContractItemIdOf<T>, T::BlockNumber, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn contract_staked_assets)]
	pub type ContractStakedAssets<T: Config> =
		StorageMap<_, Identity, ContractItemIdOf<T>, StakedAssetsVecOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasury_account)]
	pub type TreasuryAccount<T: Config> = StorageValue<_, AccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ContractCollectionId<T: Config> =
		StorageValue<_, CollectionIdOf<T>, ResultQuery<Error<T>::ContractNotFound>>;

	#[pallet::type_value]
	pub fn DefaultContractId<T: Config>() -> ContractItemIdOf<T> {
		ContractItemIdOf::<T>::default()
	}

	#[pallet::storage]
	#[pallet::getter(fn next_contract_id)]
	pub type NextContractId<T: Config> =
		StorageValue<_, ContractItemIdOf<T>, ValueQuery, DefaultContractId<T>>;

	#[pallet::genesis_config]
	pub struct GenesisConfig;

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// Create Treasury account
			let account_id = <Pallet<T>>::treasury_account_id();
			let min = T::Currency::minimum_balance();
			if T::Currency::free_balance(&account_id) < min {
				let _ = T::Currency::make_free_balance_be(&account_id, min);
			}

			let collection_config = T::ContractCollectionConfig::get();
			let collection_id =
				T::NFTHelper::create_collection(&account_id, &account_id, &collection_config)
					.expect("Should have create contract collection");
			ContractCollectionId::<T>::put(collection_id);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new staking contract has been successfully created
		StakingContractCreated { creator: AccountIdOf<T>, contract: ContractItemIdOf<T> },
		/// A new staking contract has been successfully created
		StakingContractTaken { taken_by: AccountIdOf<T>, contract: ContractItemIdOf<T> },
		/// A new staking contract has been successfully created
		StakingContractRedeemed {
			redeemed_by: AccountIdOf<T>,
			contract: ContractItemIdOf<T>,
			reward: StakingRewardOf<T>,
		},
		/// The treasury has received additional funds
		TreasuryFunded { funding_account: AccountIdOf<T>, funds: BalanceOf<T> },
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The treasury doesn't have enough funds to pay the contract rewards.
		TreasuryLacksFunds,
		/// Account doesn't have enough the minimum amount of funds necessary to contribute.
		AccountLacksFunds,
		/// The account that tried to take a staking contract failed to fulfill its conditions.
		ContractConditionsNotFulfilled,
		/// The contract reward is not valid. Either an invalid NFT or not enough tokens.
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

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::fund_treasury())]
		#[pallet::call_index(0)]
		pub fn fund_treasury(origin: OriginFor<T>, fund_amount: BalanceOf<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;

			ensure!(
				T::Currency::free_balance(&account) > fund_amount,
				Error::<T>::AccountLacksFunds
			);

			let treasury_account = Self::treasury_account_id();

			T::Currency::transfer(
				&account,
				&treasury_account,
				fund_amount,
				ExistenceRequirement::KeepAlive,
			)?;

			T::Currency::reserve(&treasury_account, fund_amount)?;

			Self::deposit_event(Event::<T>::TreasuryFunded {
				funding_account: account,
				funds: fund_amount,
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::submit_staking_contract_nft_reward())]
		#[pallet::call_index(1)]
		pub fn submit_staking_contract(
			origin: OriginFor<T>,
			staking_contract: StakingContractOf<T>,
		) -> DispatchResult {
			let account = T::StakingOrigin::ensure_origin(origin)?;

			match staking_contract.get_reward() {
				StakingReward::Tokens(amount) => {
					Self::try_transfer_funds_from_account_to_treasury(&account, amount)?;
				},
				StakingReward::NFT(address) => {
					Self::try_taking_ownership_of_nft(&account, &address)?;
				},
			}

			let contract_id = Self::try_creating_contract_nft_from(staking_contract)?;

			Self::deposit_event(Event::<T>::StakingContractCreated {
				creator: account,
				contract: contract_id,
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::take_staking_contract())]
		#[pallet::call_index(2)]
		pub fn take_staking_contract(
			origin: OriginFor<T>,
			contract_id: ContractItemIdOf<T>,
			staked_assets: StakedAssetsVecOf<T>,
		) -> DispatchResult {
			let account = T::StakingOrigin::ensure_origin(origin)?;

			Self::try_if_contract_can_be_taken_by(&account, &contract_id)?;

			let contract: StakingContractOf<T> =
				Self::active_contracts(contract_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(
				contract.evaluate_for::<T::NFTHelper>(&staked_assets),
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

			Self::deposit_event(Event::<T>::StakingContractTaken {
				taken_by: account,
				contract: contract_id,
			});

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::redeem_staking_contract_nft_reward())]
		#[pallet::call_index(3)]
		pub fn redeem_staking_contract(
			origin: OriginFor<T>,
			contract_id: ContractItemIdOf<T>,
		) -> DispatchResult {
			let account = T::StakingOrigin::ensure_origin(origin)?;

			Self::try_checking_if_contract_can_be_redeemed(&account, &contract_id)?;

			let staked_assets =
				Self::contract_staked_assets(contract_id).ok_or(Error::<T>::ContractNotFound)?;

			Self::try_transferring_staked_assets_ownership(
				&staked_assets,
				&Self::treasury_account_id(),
				&account,
				true,
			)?;

			let contract_reward = Self::active_contracts(contract_id)
				.ok_or(Error::<T>::ContractNotFound)?
				.get_reward();

			Self::try_handing_over_contract_reward_to(&account, &contract_reward)?;
			Self::try_closing_redeemed_contract(&contract_id, &account)?;

			Self::deposit_event(Event::<T>::StakingContractRedeemed {
				redeemed_by: account,
				contract: contract_id,
				reward: contract_reward,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the treasury pot.
		pub fn treasury_account_id() -> AccountIdOf<T> {
			if let Some(account) = Self::treasury_account() {
				account
			} else {
				let account: AccountIdOf<T> = T::TreasuryPalletId::get().into_account_truncating();

				TreasuryAccount::<T>::put(account.clone());

				account
			}
		}

		/// Return the amount of money available in the treasury reserves.
		// The existential deposit is not part of the pot so treasury account never gets deleted.
		pub fn treasury_pot_reserve() -> BalanceOf<T> {
			T::Currency::reserved_balance(&Self::treasury_account_id())
		}

		#[inline]
		fn get_next_contract_id() -> ContractItemIdOf<T> {
			let contract_id: ContractItemIdOf<T> = Self::next_contract_id();

			if let Some(result) = contract_id.checked_add(&ContractItemIdOf::<T>::one()) {
				NextContractId::<T>::put(result);
			} else {
				NextContractId::<T>::put(ContractItemIdOf::<T>::default())
			}

			contract_id
		}

		/// Tries to create a new NFT using the information provided in the contract as
		/// supporting data.
		#[inline]
		fn try_creating_contract_nft_from(
			contract_details: StakingContractOf<T>,
		) -> Result<ContractItemIdOf<T>, DispatchError> {
			let owner = Self::treasury_account_id();
			let collection_id = ContractCollectionId::<T>::get()
				.expect("Contract collection id should not be empty");
			let contract_id = Self::get_next_contract_id();
			let contract_item_config = T::ContractCollectionItemConfig::get();

			T::NFTHelper::mint_into(
				&collection_id,
				&contract_id,
				&owner,
				&contract_item_config,
				true,
			)?;

			ActiveContracts::<T>::insert(contract_id, contract_details);

			Ok(contract_id)
		}

		#[inline]
		fn try_if_contract_can_be_taken_by(
			account: &AccountIdOf<T>,
			contract_id: &ContractItemIdOf<T>,
		) -> DispatchResult {
			if let Some(ref contract_owner) = Self::contract_owners(contract_id) {
				if contract_owner == account {
					Err(Error::<T>::ContractAlreadyTaken.into())
				} else {
					Err(Error::<T>::ContractTakenByOther.into())
				}
			} else {
				Ok(())
			}
		}

		#[inline]
		fn try_transferring_staked_assets_ownership(
			assets: &StakedAssetsVecOf<T>,
			from: &AccountIdOf<T>,
			to: &AccountIdOf<T>,
			skip_ownership_check: bool,
		) -> DispatchResult {
			for asset in assets.iter() {
				ensure!(
					skip_ownership_check ||
						(T::NFTHelper::owner(&asset.0, &asset.1).as_ref() == Some(from)),
					Error::<T>::StakedAssetNotOwned
				);

				T::NFTHelper::transfer(&asset.0, &asset.1, to)?;
			}

			Ok(())
		}

		#[inline]
		fn try_taking_ownership_of_contract(
			contract_id: &ContractItemIdOf<T>,
			new_owner: AccountIdOf<T>,
			contract: &StakingContractOf<T>,
			staked_assets: StakedAssetsVecOf<T>,
		) -> DispatchResult {
			let collection_id = ContractCollectionId::<T>::get()
				.expect("Contract collection id should not be empty");
			T::NFTHelper::transfer(&collection_id, contract_id, &new_owner)?;

			ContractOwners::<T>::insert(contract_id, new_owner);

			let runs_until =
				<frame_system::Pallet<T>>::block_number().saturating_add(contract.get_duration());
			ContractDurations::<T>::insert(contract_id, runs_until);
			ContractStakedAssets::<T>::insert(contract_id, staked_assets);

			Ok(())
		}

		#[inline]
		fn try_handing_over_contract_reward_to(
			account_to_reward: &AccountIdOf<T>,
			contract_reward: &StakingRewardOf<T>,
		) -> DispatchResult {
			match contract_reward {
				StakingRewardOf::<T>::Tokens(amount) => {
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
				StakingRewardOf::<T>::NFT(asset) => {
					T::NFTHelper::transfer(&asset.0, &asset.1, account_to_reward)?;
				},
			}

			Ok(())
		}

		#[inline]
		fn try_transfer_funds_from_account_to_treasury(
			account: &AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure!(T::Currency::can_slash(account, amount), Error::<T>::AccountLacksFunds);

			let treasury_account = Self::treasury_account_id();

			T::Currency::transfer(
				account,
				&treasury_account,
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

			T::Currency::reserve(&treasury_account, amount)?;

			Ok(())
		}

		#[inline]
		fn try_taking_ownership_of_nft(
			original_owner: &AccountIdOf<T>,
			nft_addr: &NFTAddressOf<T>,
		) -> DispatchResult {
			ensure!(
				T::NFTHelper::owner(&nft_addr.0, &nft_addr.1).as_ref() == Some(original_owner),
				Error::<T>::ContractRewardNotOwned
			);

			ensure!(
				nft_addr.0 !=
					ContractCollectionId::<T>::get()
						.expect("Contract collection id should not be empty"),
				Error::<T>::InvalidContractReward
			);

			T::NFTHelper::transfer(&nft_addr.0, &nft_addr.1, &Self::treasury_account_id())?;

			Ok(())
		}

		#[inline]
		fn try_checking_if_contract_can_be_redeemed(
			contract_redeemer: &AccountIdOf<T>,
			contract_id: &ContractItemIdOf<T>,
		) -> DispatchResult {
			ensure!(
				Self::contract_owners(contract_id).as_ref() == Some(contract_redeemer),
				Error::<T>::ContractNotOwned
			);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let contract_expiry =
				Self::contract_durations(contract_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(current_block >= contract_expiry, Error::<T>::ContractStillActive);

			Ok(())
		}

		#[inline]
		fn try_closing_redeemed_contract(
			contract_id: &ContractItemIdOf<T>,
			contract_redeemer: &AccountIdOf<T>,
		) -> DispatchResult {
			ContractStakedAssets::<T>::remove(contract_id);
			ContractDurations::<T>::remove(contract_id);
			ContractOwners::<T>::remove(contract_id);
			ActiveContracts::<T>::remove(contract_id);

			let contract_collection = ContractCollectionId::<T>::get()
				.expect("Contract collection id should not be empty");
			T::NFTHelper::burn(&contract_collection, contract_id, Some(contract_redeemer))?;

			Ok(())
		}
	}
}
