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
		BalanceStatus, Currency,
		ExistenceRequirement::AllowDeath,
		Get, Imbalance, ReservableCurrency, WithdrawReasons,
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

		/// The maximum number of clauses a contract can have.
		#[pallet::constant]
		type MaxClauses: Get<u32>;

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
	pub type ContractCollectionId<T: Config> = StorageValue<_, CollectionIdOf<T>>;

	#[pallet::storage]
	pub type NextContractId<T: Config> = StorageValue<_, ItemIdOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type Contracts<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, ContractOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type ContractOwners<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type ContractDurations<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, T::BlockNumber, OptionQuery>;

	#[pallet::storage]
	pub type ContractStakedItems<T: Config> =
		StorageMap<_, Identity, ItemIdOf<T>, StakedItemsOf<T>, OptionQuery>;

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
		Created { creator: T::AccountId, contract_id: ItemIdOf<T> },
		/// A staking contract has been accepted.
		Accepted { accepted_by: T::AccountId, contract_id: ItemIdOf<T> },
		/// A staking contract has been claimed.
		Claimed { claimed_by: T::AccountId, contract_id: ItemIdOf<T>, reward: RewardOf<T> },
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
		/// The given collection belongs to someone else.
		Ownership,
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
		pub fn create(origin: OriginFor<T>, contract: ContractOf<T>) -> DispatchResult {
			let creator = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			let contract_id = Self::create_contract(&creator, &contract)?;
			Self::deposit_event(Event::<T>::Created { creator, contract_id });
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::accept())]
		#[pallet::call_index(5)]
		pub fn accept(
			origin: OriginFor<T>,
			contract_id: ItemIdOf<T>,
			stakes: StakedItemsOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_acceptable(&contract_id, &staker, &stakes)?;
			Self::accept_contract(&contract_id, &staker, &stakes)?;
			Self::deposit_event(Event::<T>::Accepted { accepted_by: staker, contract_id });
			Ok(())
		}

		// #[pallet::weight(
		// 	T::WeightInfo::claim_token_reward()
		// 		.max(T::WeightInfo::claim_nft_reward())
		// )]
		// #[pallet::call_index(6)]
		// pub fn claim(origin: OriginFor<T>, contract_id: ItemIdOf<T>) -> DispatchResult {
		// 	Self::ensure_pallet_unlocked()?;

		// 	let account = ensure_signed(origin)?;

		// 	Self::try_checking_if_contract_can_be_redeemed(&account, &contract_id)?;

		// 	let staked_assets =
		// 		ContractStakedItems::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;

		// 	Self::transfer_items(
		// 		&staked_assets,
		// 		&account,
		// 	)?;

		// 	let reward = Contracts::<T>::get(contract_id)
		// 		.ok_or(Error::<T>::UnknownContract)?
		// 		.reward;

		// 	Self::try_handing_over_contract_reward_to(&account, &reward)?;
		// 	Self::try_closing_redeemed_contract(&contract_id, &account)?;

		// 	Self::deposit_event(Event::<T>::Claimed { claimed_by: account, contract_id, reward });

		// 	Ok(())
		// }
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

		fn contract_collection_id() -> Result<CollectionIdOf<T>, DispatchError> {
			let id =
				ContractCollectionId::<T>::get().ok_or(Error::<T>::UnknownContractCollection)?;
			Ok(id)
		}

		fn ensure_creator(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
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

		fn ensure_acceptable(
			contract_id: &ItemIdOf<T>,
			who: &T::AccountId,
			stake_addresses: &[NftAddressOf<T>],
		) -> DispatchResult {
			let collection_id = Self::contract_collection_id()?;
			ensure!(
				Self::ensure_item_ownership(who, &collection_id, contract_id).is_err(),
				Error::<T>::AlreadyAccepted
			);

			stake_addresses.iter().try_for_each(|NftAddress(collection_id, contract_id)| {
				Self::ensure_item_ownership(who, collection_id, contract_id)
			})?;

			let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			ensure!(
				contract.evaluate_for::<T::AccountId, T::NftHelper>(stake_addresses),
				Error::<T>::UnfulfilledClause
			);

			Ok(())
		}

		pub(crate) fn create_contract(
			creator: &T::AccountId,
			contract: &ContractOf<T>,
		) -> Result<ItemIdOf<T>, DispatchError> {
			// Lock contract rewards in pallet account.
			let pallet_account_id = Self::account_id();
			match &contract.reward {
				Reward::Tokens(amount) => {
					let imbalance = T::Currency::withdraw(
						creator,
						*amount,
						WithdrawReasons::TRANSFER,
						AllowDeath,
					)?;
					T::Currency::deposit_creating(&pallet_account_id, imbalance.peek());
					Ok(())
				},
				Reward::Nft(address) => {
					let NftAddress(collection_id, item_id) = address;
					Self::ensure_item_ownership(creator, collection_id, item_id)?;
					T::NftHelper::transfer(collection_id, item_id, &pallet_account_id)
				},
			}?;

			// Create a contract NFT.
			let collection_id = Self::contract_collection_id()?;
			let contract_id = NextContractId::<T>::get();
			T::NftHelper::mint_into(
				&collection_id,
				&contract_id,
				&pallet_account_id,
				&T::ContractCollectionItemConfig::get(),
				true,
			)?;
			Contracts::<T>::insert(contract_id, contract);
			NextContractId::<T>::mutate(|id| id.saturating_inc());

			Ok(contract_id)
		}

		fn accept_contract(
			contract_id: &ItemIdOf<T>,
			who: &T::AccountId,
			stake_addresses: &[NftAddressOf<T>],
		) -> DispatchResult {
			let contract_address = NftAddress(Self::contract_collection_id()?, *contract_id);
			Self::transfer_items(&[contract_address], who)?;
			Self::transfer_items(stake_addresses, &Self::account_id())?;

			let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			let current_block_number = frame_system::Pallet::<T>::block_number();
			let contract_end = current_block_number.saturating_add(contract.duration);
			ContractDurations::<T>::insert(contract_id, contract_end);

			let bounded_stakes = StakedItemsOf::<T>::truncate_from(stake_addresses.to_vec());
			ContractStakedItems::<T>::insert(contract_id, bounded_stakes);

			Ok(())
		}

		fn transfer_items(addresses: &[NftAddressOf<T>], to: &T::AccountId) -> DispatchResult {
			addresses.iter().try_for_each(|NftAddress(collection_id, item_id)| {
				T::NftHelper::transfer(collection_id, item_id, to)
			})
		}

		fn try_checking_if_contract_can_be_redeemed(
			contract_redeemer: &T::AccountId,
			contract_id: &ItemIdOf<T>,
		) -> DispatchResult {
			ensure!(
				ContractOwners::<T>::get(contract_id).as_ref() == Some(contract_redeemer),
				Error::<T>::ContractNotOwned
			);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let contract_expiry =
				ContractDurations::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;

			ensure!(current_block >= contract_expiry, Error::<T>::ContractStillActive);

			Ok(())
		}

		fn try_closing_redeemed_contract(
			contract_id: &ItemIdOf<T>,
			contract_redeemer: &T::AccountId,
		) -> DispatchResult {
			ContractStakedAssets::<T>::remove(contract_id);
			ContractDurations::<T>::remove(contract_id);
			ContractOwners::<T>::remove(contract_id);
			Contracts::<T>::remove(contract_id);

			let contract_collection = ContractCollectionId::<T>::get()
				.expect("Contract collection id should not be empty");
			T::NftHelper::burn(&contract_collection, contract_id, Some(contract_redeemer))?;

			Ok(())
		}
	}
}
