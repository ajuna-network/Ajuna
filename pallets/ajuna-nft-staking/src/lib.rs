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
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Zero};
use sp_std::prelude::*;

pub use contracts::*;
pub use pallet::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CollectionIdOf<T> = <T as Config>::CollectionId;
	pub(crate) type ItemIdOf<T> = <T as Config>::ItemId;

	pub(crate) type ContractOf<T> = Contract<
		BalanceOf<T>,
		CollectionIdOf<T>,
		ItemIdOf<T>,
		<T as frame_system::Config>::BlockNumber,
		<T as Config>::ContractAttributeKey,
		<T as Config>::ContractAttributeValue,
	>;
	pub(crate) type NftAddressOf<T> = NftAddress<CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type RewardOf<T> = Reward<BalanceOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;
	pub(crate) type StakedItemsOf<T> =
		BoundedVec<NftAddressOf<T>, <T as Config>::MaxStakingClauses>;

	#[derive(
		Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, Default, Eq, PartialEq,
	)]
	pub struct GlobalConfig {
		pub pallet_locked: bool,
	}

	enum Operation {
		Claim,
		Cancel,
		Snipe,
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

		/// The maximum number of staking clauses a contract can have.
		#[pallet::constant]
		type MaxStakingClauses: Get<u32>;

		/// The maximum number of fee clauses a contract can have.
		#[pallet::constant]
		type MaxFeeClauses: Get<u32>;

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
	pub type ContractAccepted<T: Config> = StorageMap<_, Identity, T::ItemId, T::BlockNumber>;

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
		Created { contract_id: T::ItemId },
		/// A new staking contract has been removed.
		Removed { contract_id: T::ItemId },
		/// A staking contract has been accepted.
		Accepted { by: T::AccountId, contract_id: T::ItemId },
		/// A staking contract has been claimed.
		Claimed { by: T::AccountId, contract_id: T::ItemId, reward: RewardOf<T> },
		/// A staking contract has been cancelled.
		Cancelled { by: T::AccountId, contract_id: T::ItemId },
		/// A staking contract has been sniped.
		Sniped { by: T::AccountId, contract_id: T::ItemId, reward: RewardOf<T> },
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The given creator doesn't exist.
		UnknownCreator,
		/// The given collection doesn't exist.
		UnknownCollection,
		/// The given contract collection doesn't exist.
		UnknownContractCollection,
		/// The given item doesn't exist.
		UnknownItem,
		/// The given contract doesn't exist.
		UnknownContract,
		/// The given contract's activation is unknown.
		UnknownActivation,
		/// The given collection or item belongs to someone else.
		Ownership,
		/// The given contract belongs to someone else.
		ContractOwnership,
		/// The pallet is currently locked and cannot be interacted with.
		PalletLocked,
		/// The given contract's activation block number is set in the past.
		IncorrectActivation,
		/// The given contract's active duration is zero. This results in immediate deactivation of
		/// newly created contracts.
		ZeroActiveDuration,
		/// The given contract's claim duration is zero. This results in immediate expiry of
		/// fulfilled contracts,
		ZeroClaimDuration,
		/// The given contract's fee clause is unfulfilled.
		UnfulfilledFeeClause,
		/// The given contract's staking clause is unfulfilled.
		UnfulfilledStakingClause,
		/// The contract is inactive hence cannot be accepted.
		Inactive,
		/// The contract is staking hence cannot be removed.
		Staking,
		/// The contract is expired hence cannot be claimed.
		Expired,
		/// The contract is claimable, so it cannot be cancelled or sniped.
		Claimable,
		/// The contract is available, or not yet accepted.
		Available,
		/// The number of the given contract's staking clauses exceeds maximum allowed.
		MaxStakingClauses,
		/// The number of the given contract's fee clauses exceeds maximum allowed.
		MaxFeeClauses,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set a creator account.
		///
		/// This call allows setting an account to act as a contract creator. It must be called with
		/// root privilege.
		#[pallet::weight(T::WeightInfo::set_creator())]
		#[pallet::call_index(0)]
		pub fn set_creator(origin: OriginFor<T>, creator: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Creator::<T>::put(&creator);
			Self::deposit_event(Event::CreatorSet { creator });
			Ok(())
		}

		/// Set a collection ID for contract NFTs.
		///
		///
		/// This call allows setting an externally created collection ID to associate contract NFTs.
		/// It must be signed with the creator account.
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

		/// Set new values for global configuration.
		///
		/// This call allows updating global configuration. It must be signed by the creator
		/// account.
		#[pallet::weight(T::WeightInfo::set_global_config())]
		#[pallet::call_index(2)]
		pub fn set_global_config(origin: OriginFor<T>, new_config: GlobalConfig) -> DispatchResult {
			let _ = Self::ensure_creator(origin)?;
			GlobalConfigs::<T>::put(new_config);
			Self::deposit_event(Event::SetGlobalConfig { new_config });
			Ok(())
		}

		/// Create a staking contract.
		///
		/// This call allows the creator to define and initiate a new staking contract. The creator
		/// sets the parameters of the contract, such as the assets involved, the staking period,
		/// and any other requirements. The creator will also transfer the necessary reward NFTs or
		/// tokens to the provider, which will be locked until a staker claims them, or the contract
		/// is removed in an unaccepted state.
		#[pallet::weight(
			T::WeightInfo::create_token_reward()
				.max(T::WeightInfo::create_nft_reward())
		)]
		#[pallet::call_index(3)]
		pub fn create(
			origin: OriginFor<T>,
			contract_id: T::ItemId,
			contract: ContractOf<T>,
		) -> DispatchResult {
			let creator = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_contract_clauses(&contract)?;
			Self::create_contract(creator, contract_id, contract)
		}

		/// Remove a staking contract.
		///
		/// This call enables the creator to remove inactive staking contracts that haven't been
		/// accepted by any staker. This can be done to clean up the available staking contracts or
		/// to adjust the parameters before re-creating the contract.
		#[pallet::weight(12_345)]
		#[pallet::call_index(4)]
		pub fn remove(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let _ = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::remove_contract(contract_id)
		}

		/// Accept an available staking contract.
		///
		/// This call allows a player (staker) to accept an available staking contract. When
		/// executing this call, the staker will transfer the required stake and fee NFTs to the
		/// provider, thus engaging in the contract. The provider will issue a contract NFT to the
		/// staker, acknowledging their participation in the staking contract.
		#[pallet::weight(T::WeightInfo::accept())]
		#[pallet::call_index(5)]
		pub fn accept(
			origin: OriginFor<T>,
			contract_id: T::ItemId,
			stakes: Vec<NftAddressOf<T>>,
			fees: Vec<NftAddressOf<T>>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_acceptable(&contract_id, &staker, &stakes, &fees)?;
			Self::accept_contract(contract_id, staker, &stakes, &fees)
		}

		/// Cancel an active staking contract.
		///
		/// This call allows the staker, holding a contract NFT, to terminate a staking contract
		/// prematurely. Doing so will return the stake NFT, but an additional cancellation fee will
		/// be charged. The staker will not receive any rewards associated with the canceled
		/// contract.
		#[pallet::weight(12_345)]
		#[pallet::call_index(6)]
		pub fn cancel(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_contract(Operation::Cancel, &contract_id, &staker)?;
			Self::process_contract(Operation::Cancel, contract_id, staker)
		}

		/// Claim a fulfilled staking contract.
		///
		/// The staker, who holds a contract NFT, can call this function to claim the rewards
		/// associated with the fulfilled staking contract. Upon successful execution, the provider
		/// will transfer the reward NFTs / tokens to the staker and return the stake NFT.
		#[pallet::weight(
			T::WeightInfo::claim_token_reward()
				.max(T::WeightInfo::claim_nft_reward())
		)]
		#[pallet::call_index(7)]
		pub fn claim(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let claimer = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_contract(Operation::Claim, &contract_id, &claimer)?;
			Self::process_contract(Operation::Claim, contract_id, claimer)
		}

		/// Snipe someone's claimable contract.
		///
		/// This call allows any user to claim expired, fulfilled contracts. If a staker hasn't
		/// claimed their rewards within the specified expiration time, another user (sniper) can
		/// claim them. When this occurs, the stake NFT is returned to the original contract NFT
		/// holder, but the rewards are transferred to the sniper. This feature encourages the
		/// timely claiming of contracts and ensures the contract's completion.
		#[pallet::weight(12_345)]
		#[pallet::call_index(8)]
		pub fn snipe(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let sniper = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_contract(Operation::Snipe, &contract_id, &sniper)?;
			Self::process_contract(Operation::Snipe, contract_id, sniper)?;
			Ok(())
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
			mut contract: ContractOf<T>,
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

			// Check and set activation block number.
			let now = frame_system::Pallet::<T>::block_number();
			contract.activation = match contract.activation {
				Some(block_number) => {
					ensure!(block_number > now, Error::<T>::IncorrectActivation);
					Some(block_number)
				},
				None => Some(now),
			};
			ensure!(contract.active_duration > Zero::zero(), Error::<T>::ZeroActiveDuration);
			ensure!(contract.claim_duration > Zero::zero(), Error::<T>::ZeroClaimDuration);
			Contracts::<T>::insert(contract_id, contract);

			Self::deposit_event(Event::<T>::Created { contract_id });
			Ok(())
		}

		fn remove_contract(contract_id: T::ItemId) -> DispatchResult {
			Self::ensure_contract_ownership(&contract_id, &Self::account_id())
				.map_err(|_| Error::<T>::Staking)?;
			Contracts::<T>::remove(contract_id);
			Self::deposit_event(Event::<T>::Removed { contract_id });
			Ok(())
		}

		pub(crate) fn accept_contract(
			contract_id: T::ItemId,
			who: T::AccountId,
			stake_addresses: &[NftAddressOf<T>],
			fee_addresses: &[NftAddressOf<T>],
		) -> DispatchResult {
			// Transfer contract, stake and fee NFTs.
			let contract_address = NftAddress(Self::contract_collection_id()?, contract_id);
			Self::transfer_items(&[contract_address], &who)?;
			Self::transfer_items(stake_addresses, &Self::account_id())?;
			Self::transfer_items(fee_addresses, &Self::creator()?)?;

			// Record staked NFTs' addresses.
			let bounded_stakes = StakedItemsOf::<T>::try_from(stake_addresses.to_vec())
				.map_err(|_| Error::<T>::MaxStakingClauses)?;
			ContractStakedItems::<T>::insert(contract_id, bounded_stakes);

			// Record contract accepted block.
			let now = frame_system::Pallet::<T>::block_number();
			ContractAccepted::<T>::insert(contract_id, now);

			// Emit events.
			Self::deposit_event(Event::<T>::Accepted { by: who, contract_id });
			Ok(())
		}

		fn process_contract(
			op: Operation,
			contract_id: T::ItemId,
			who: T::AccountId,
		) -> DispatchResult {
			// Transfer rewards.
			let creator = Self::creator()?;
			let Contract { cancel_fee, reward, .. } = Self::contract(&contract_id)?;
			let beneficiary = match op {
				Operation::Claim | Operation::Snipe => &who,
				Operation::Cancel => {
					T::Currency::transfer(&who, &creator, cancel_fee, AllowDeath)?;
					&creator
				},
			};
			match reward {
				Reward::Tokens(amount) =>
					T::Currency::transfer(&Self::account_id(), beneficiary, amount, AllowDeath),
				Reward::Nft(NftAddress(collection_id, item_id)) =>
					T::NftHelper::transfer(&collection_id, &item_id, beneficiary),
			}?;

			// Return staked items.
			let contract_owner = Self::contract_owner(&contract_id)?;
			Self::transfer_items(&Self::staked_items(&contract_id)?, &contract_owner)?;

			// Burn contract NFT.
			let collection_id = Self::contract_collection_id()?;
			T::NftHelper::burn(&collection_id, &contract_id, Some(&contract_owner))?;

			// Clean up storage.
			ContractStakedItems::<T>::remove(contract_id);
			ContractAccepted::<T>::remove(contract_id);
			Contracts::<T>::remove(contract_id);

			// Emit events.
			match op {
				Operation::Claim =>
					Self::deposit_event(Event::<T>::Claimed { by: who, contract_id, reward }),
				Operation::Cancel =>
					Self::deposit_event(Event::<T>::Cancelled { by: who, contract_id }),
				Operation::Snipe =>
					Self::deposit_event(Event::<T>::Sniped { by: who, contract_id, reward }),
			};

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
			let owner = Self::contract_owner(contract_id)?;
			ensure!(who == &owner, Error::<T>::ContractOwnership);
			let contract = Self::contract(contract_id)?;
			Ok(contract)
		}

		fn ensure_contract_accepted(
			contract_id: &T::ItemId,
		) -> Result<ContractOf<T>, DispatchError> {
			let owner = Self::contract_owner(contract_id)?;
			ensure!(owner != Self::account_id(), Error::<T>::Available);
			let contract = Self::contract(contract_id)?;
			Ok(contract)
		}

		fn ensure_contract_clauses(contract: &ContractOf<T>) -> DispatchResult {
			ensure!(
				contract.stake_clauses.len() as u32 <= T::MaxStakingClauses::get(),
				Error::<T>::MaxStakingClauses
			);
			ensure!(
				contract.fee_clauses.len() as u32 <= T::MaxFeeClauses::get(),
				Error::<T>::MaxFeeClauses
			);
			Ok(())
		}

		fn ensure_acceptable(
			contract_id: &T::ItemId,
			who: &T::AccountId,
			stake_addresses: &[NftAddressOf<T>],
			fee_addresses: &[NftAddressOf<T>],
		) -> DispatchResult {
			ensure!(
				stake_addresses.len() as u32 <= T::MaxStakingClauses::get(),
				Error::<T>::MaxStakingClauses
			);
			ensure!(
				fee_addresses.len() as u32 <= T::MaxFeeClauses::get(),
				Error::<T>::MaxFeeClauses
			);

			let contract = Self::ensure_contract_ownership(contract_id, &Self::account_id())?;
			let activation = contract.activation.ok_or(Error::<T>::UnknownActivation)?;
			let active_duration = contract.active_duration;
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(now >= activation && now <= activation + active_duration, Error::<T>::Inactive);

			stake_addresses.iter().try_for_each(|NftAddress(collection_id, contract_id)| {
				Self::ensure_item_ownership(collection_id, contract_id, who)
			})?;

			ensure!(
				contract.evaluate_stakes::<T::AccountId, T::NftHelper>(stake_addresses),
				Error::<T>::UnfulfilledStakingClause
			);
			ensure!(
				contract.evaluate_fees::<T::AccountId, T::NftHelper>(fee_addresses),
				Error::<T>::UnfulfilledFeeClause
			);

			Ok(())
		}

		fn ensure_contract(
			op: Operation,
			contract_id: &T::ItemId,
			who: &T::AccountId,
		) -> DispatchResult {
			let Contract { claim_duration, stake_duration, .. } = match op {
				Operation::Claim | Operation::Cancel =>
					Self::ensure_contract_ownership(contract_id, who),
				Operation::Snipe => Self::ensure_contract_accepted(contract_id),
			}?;
			let now = <frame_system::Pallet<T>>::block_number();
			let accepted_block = Self::contract_accepted(contract_id)?;
			let end = accepted_block + stake_duration;

			match op {
				Operation::Claim => {
					ensure!(now <= end + claim_duration, Error::<T>::Expired);
					ensure!(now >= end, Error::<T>::Staking);
				},
				Operation::Cancel => {
					ensure!(now <= end + claim_duration, Error::<T>::Expired);
					ensure!(now < end, Error::<T>::Claimable);
				},
				Operation::Snipe => {
					ensure!(now >= end, Error::<T>::Staking);
					ensure!(now > end + claim_duration, Error::<T>::Claimable);
				},
			}
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
			let creator = Creator::<T>::get().ok_or(Error::<T>::UnknownCreator)?;
			Ok(creator)
		}
		fn contract(contract_id: &T::ItemId) -> Result<ContractOf<T>, DispatchError> {
			let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			Ok(contract)
		}
		fn contract_owner(contract_id: &T::ItemId) -> Result<T::AccountId, DispatchError> {
			let collection_id = Self::contract_collection_id()?;
			let owner = T::NftHelper::owner(&collection_id, contract_id)
				.ok_or(Error::<T>::UnknownContract)?;
			Ok(owner)
		}
		fn staked_items(contract_id: &T::ItemId) -> Result<StakedItemsOf<T>, DispatchError> {
			let items =
				ContractStakedItems::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			Ok(items)
		}
		fn contract_accepted(contract_id: &T::ItemId) -> Result<T::BlockNumber, DispatchError> {
			let accepted_block =
				ContractAccepted::<T>::get(contract_id).ok_or(Error::<T>::UnknownContract)?;
			Ok(accepted_block)
		}
	}
}
