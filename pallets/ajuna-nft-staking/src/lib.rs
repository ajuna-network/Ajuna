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

use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::nonfungibles_v2::{Destroy, Inspect, Mutate, Transfer},
		Currency,
		ExistenceRequirement::AllowDeath,
		Get, Imbalance, WithdrawReasons,
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
	pub(crate) type NftAddressOf<T> = NftAddress<CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type RewardOf<T> = Reward<BalanceOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type StakedItemsOf<T> = BoundedVec<NftAddressOf<T>, <T as Config>::MaxClauses>;

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
		type Currency: Currency<Self::AccountId>;

		/// Identifier for the collection of an Nft.
		type CollectionId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		/// The type used to identify a unique item within a collection.
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		/// Type that holds the specific configurations for an item.
		type ItemConfig: Default + MaxEncodedLen + TypeInfo;

		type NftHelper: Inspect<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Mutate<Self::AccountId, Self::ItemConfig>
			+ Destroy<Self::AccountId>
			+ Transfer<Self::AccountId>;

		/// The maximum number of clauses a contract can have.
		#[pallet::constant]
		type MaxClauses: Get<u32>;

		/// Type of the contract attributes keys, used on contract condition evaluation
		type ContractAttributeKey: Member + Encode + Decode + MaxEncodedLen + TypeInfo;

		/// Type of the contract attributes values, used on contract condition evaluation
		type ContractAttributeValue: Member + Encode + Decode + MaxEncodedLen + TypeInfo;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Creator<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type GlobalConfigs<T: Config> = StorageValue<_, GlobalConfig, ValueQuery>;

	#[pallet::storage]
	pub type ContractCollectionId<T: Config> = StorageValue<_, T::CollectionId>;

	#[pallet::storage]
	pub type Contracts<T: Config> = StorageMap<_, Identity, T::ItemId, ContractOf<T>>;

	#[pallet::storage]
	pub type ContractOwners<T: Config> = StorageMap<_, Identity, T::ItemId, T::AccountId>;

	#[pallet::storage]
	pub type ContractEnds<T: Config> = StorageMap<_, Identity, T::ItemId, T::BlockNumber>;

	#[pallet::storage]
	pub type ContractStakedItems<T: Config> = StorageMap<_, Identity, T::ItemId, StakedItemsOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An creator has been set.
		CreatorSet { creator: T::AccountId },
		/// The collection holding the staking contracts has been set.
		ContractCollectionSet { collection_id: T::CollectionId },
		/// The pallet's global config has been set.
		SetGlobalConfig { new_config: GlobalConfig },
		/// A new staking contract has been created.
		Created { creator: T::AccountId, contract_id: T::ItemId },
		/// A staking contract has been accepted.
		Accepted { accepted_by: T::AccountId, contract_id: T::ItemId },
		/// A staking contract has been claimed.
		Claimed { claimed_by: T::AccountId, contract_id: T::ItemId, reward: RewardOf<T> },
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
		/// The given contract doesn't exist.
		UnknownContract,
		/// The given collection or item belongs to someone else.
		Ownership,
		/// The given contract belongs to someone else.
		ContractOwnership,
		/// The pallet is currently locked and cannot be interacted with.
		PalletLocked,
		/// The given contract is already accepted by an account.
		AlreadyAccepted,
		/// The treasury doesn't have enough funds to pay the contract rewards.
		TreasuryLacksFunds,
		/// Account doesn't have enough the minimum amount of funds necessary to contribute.
		AccountLacksFunds,
		/// The given contract clause is unfulfilled.
		UnfulfilledClause,
		/// The contract reward is not valid. Either an invalid Nft or not enough tokens.
		InvalidContractReward,
		/// The account that tried to take a staking contract didn't own one or more of the
		/// staked assets.
		StakedAssetNotOwned,
		/// The account that tried to create a contract didn't actually own it's reward.
		ContractRewardNotOwned,
		/// The account that tried to redeemed a contract didn't own it
		ContractNotOwned,
		/// The contract is still active, so it cannot be redeemed
		ContractStillActive,
		/// The given data cannot be bounded.
		IncorrectData,
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
		pub fn create(
			origin: OriginFor<T>,
			contract_id: T::ItemId,
			contract: ContractOf<T>,
		) -> DispatchResult {
			let creator = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::create_contract(creator, contract_id, contract)
		}

		#[pallet::weight(T::WeightInfo::accept())]
		#[pallet::call_index(5)]
		pub fn accept(
			origin: OriginFor<T>,
			contract_id: T::ItemId,
			stakes: StakedItemsOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_acceptable(&contract_id, &staker, &stakes)?;
			Self::accept_contract(contract_id, staker, &stakes)
		}

		#[pallet::weight(
			T::WeightInfo::claim_token_reward()
				.max(T::WeightInfo::claim_nft_reward())
		)]
		#[pallet::call_index(6)]
		pub fn claim(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let claimer = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_claimable(&contract_id, &claimer)?;
			Self::claim_contract(contract_id, claimer)
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account identifier of the pallet account.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		/// Return the balance of the pallet account.
		pub fn account_balance() -> BalanceOf<T> {
			T::Currency::free_balance(&Self::account_id())
		}

		pub(crate) fn create_contract(
			creator: T::AccountId,
			contract_id: T::ItemId,
			contract: ContractOf<T>,
		) -> DispatchResult {
			// Lock contract rewards in pallet account.
			let pallet_account_id = Self::account_id();
			match &contract.reward {
				Reward::Tokens(amount) => {
					let imbalance = T::Currency::withdraw(
						&creator,
						*amount,
						WithdrawReasons::TRANSFER,
						AllowDeath,
					)?;
					T::Currency::deposit_creating(&pallet_account_id, imbalance.peek());
					Ok(())
				},
				Reward::Nft(address) => {
					let NftAddress(collection_id, item_id) = address;
					Self::ensure_item_ownership(collection_id, item_id, &creator)?;
					T::NftHelper::transfer(collection_id, item_id, &pallet_account_id)
				},
			}?;

			// Create a contract NFT.
			let collection_id = Self::contract_collection_id()?;
			T::NftHelper::mint_into(
				&collection_id,
				&contract_id,
				&pallet_account_id,
				&T::ItemConfig::default(),
				true,
			)?;
			Contracts::<T>::insert(contract_id, contract);

			Self::deposit_event(Event::<T>::Created { creator, contract_id });
			Ok(())
		}

		pub(crate) fn accept_contract(
			contract_id: T::ItemId,
			who: T::AccountId,
			stake_addresses: &[NftAddressOf<T>],
		) -> DispatchResult {
			let contract_address = NftAddress(Self::contract_collection_id()?, contract_id);
			Self::transfer_items(&[contract_address], &who)?;
			Self::transfer_items(stake_addresses, &Self::account_id())?;

			let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			let current_block_number = frame_system::Pallet::<T>::block_number();
			let contract_end = current_block_number.saturating_add(contract.duration);
			ContractEnds::<T>::insert(contract_id, contract_end);

			let bounded_stakes = StakedItemsOf::<T>::truncate_from(stake_addresses.to_vec());
			ContractStakedItems::<T>::insert(contract_id, bounded_stakes);

			Self::deposit_event(Event::<T>::Accepted { accepted_by: who, contract_id });
			Ok(())
		}

		fn claim_contract(contract_id: T::ItemId, who: T::AccountId) -> DispatchResult {
			let staked_items =
				ContractStakedItems::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			Self::transfer_items(&staked_items, &who)?;

			let reward = Self::ensure_contract_ownership(&contract_id, &who)?.reward;
			match reward {
				Reward::Tokens(amount) =>
					T::Currency::transfer(&Self::account_id(), &who, amount, AllowDeath),
				Reward::Nft(NftAddress(collection_id, item_id)) =>
					T::NftHelper::transfer(&collection_id, &item_id, &who),
			}?;

			let collection_id = Self::contract_collection_id()?;
			T::NftHelper::burn(&collection_id, &contract_id, Some(&who))?;

			ContractStakedItems::<T>::remove(contract_id);
			ContractEnds::<T>::remove(contract_id);
			ContractOwners::<T>::remove(contract_id);
			Contracts::<T>::remove(contract_id);

			Self::deposit_event(Event::<T>::Claimed { claimed_by: who, contract_id, reward });
			Ok(())
		}

		fn transfer_items(addresses: &[NftAddressOf<T>], to: &T::AccountId) -> DispatchResult {
			addresses.iter().try_for_each(|NftAddress(collection_id, item_id)| {
				T::NftHelper::transfer(collection_id, item_id, to)
			})
		}
	}

	// Implementation of ensure checks.
	impl<T: Config> Pallet<T> {
		fn ensure_creator(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
			let maybe_creator = ensure_signed(origin)?;
			let existing_creator = Self::creator()?;
			ensure!(maybe_creator == existing_creator, DispatchError::BadOrigin);
			Ok(maybe_creator)
		}

		fn ensure_pallet_unlocked() -> DispatchResult {
			ensure!(!GlobalConfigs::<T>::get().pallet_locked, Error::<T>::PalletLocked);
			Ok(())
		}

		fn ensure_collection_ownership(
			who: &T::AccountId,
			collection_id: &T::CollectionId,
		) -> DispatchResult {
			let owner = T::NftHelper::collection_owner(collection_id)
				.ok_or(Error::<T>::UnknownCollection)?;
			ensure!(who == &owner, Error::<T>::Ownership);
			Ok(())
		}

		fn ensure_item_ownership(
			collection_id: &T::CollectionId,
			item_id: &T::ItemId,
			who: &T::AccountId,
		) -> DispatchResult {
			let owner =
				T::NftHelper::owner(collection_id, item_id).ok_or(Error::<T>::UnknownItem)?;
			ensure!(who == &owner, Error::<T>::Ownership);
			Ok(())
		}

		fn ensure_contract_ownership(
			contract_id: &T::ItemId,
			who: &T::AccountId,
		) -> Result<ContractOf<T>, DispatchError> {
			let collection_id = Self::contract_collection_id()?;
			let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			Self::ensure_item_ownership(&collection_id, contract_id, who)
				.map_err(|_| Error::<T>::ContractOwnership)?;
			Ok(contract)
		}

		fn ensure_acceptable(
			contract_id: &T::ItemId,
			who: &T::AccountId,
			stake_addresses: &[NftAddressOf<T>],
		) -> DispatchResult {
			stake_addresses.iter().try_for_each(|NftAddress(collection_id, contract_id)| {
				Self::ensure_item_ownership(collection_id, contract_id, who)
			})?;

			let contract = Self::ensure_contract_ownership(contract_id, &Self::account_id())?;
			ensure!(
				contract.evaluate_for::<T::AccountId, T::NftHelper>(stake_addresses),
				Error::<T>::UnfulfilledClause
			);

			Ok(())
		}

		fn ensure_claimable(contract_id: &T::ItemId, who: &T::AccountId) -> DispatchResult {
			let _ = Self::ensure_contract_ownership(contract_id, who)?;
			let current_block = <frame_system::Pallet<T>>::block_number();
			let end = ContractEnds::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			ensure!(current_block >= end, Error::<T>::ContractStillActive);
			Ok(())
		}
	}

	// Implementation of storage getters.
	impl<T: Config> Pallet<T> {
		fn contract_collection_id() -> Result<T::CollectionId, DispatchError> {
			let id =
				ContractCollectionId::<T>::get().ok_or(Error::<T>::UnknownContractCollection)?;
			Ok(id)
		}
		fn creator() -> Result<T::AccountId, DispatchError> {
			let creator = Creator::<T>::get().ok_or(Error::<T>::CreatorNotSet)?;
			Ok(creator)
		}
	}
}
