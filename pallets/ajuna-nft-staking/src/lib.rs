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
use scale_info::prelude::string::String as Str;
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, Zero},
	ArithmeticError,
};
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
		<T as Config>::AttributeKey,
		<T as Config>::AttributeValueLimit,
	>;
	pub type NftIdOf<T> = NftId<CollectionIdOf<T>, ItemIdOf<T>>;
	pub type RewardOf<T> = Reward<BalanceOf<T>, CollectionIdOf<T>, ItemIdOf<T>>;

	pub(crate) type ContractIdsOf<T> = BoundedVec<ItemIdOf<T>, <T as Config>::MaxContracts>;
	pub(crate) type StakedItemsOf<T> = BoundedVec<NftIdOf<T>, <T as Config>::MaxStakingClauses>;

	#[derive(
		Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, Default, Eq, PartialEq,
	)]
	pub struct GlobalConfig {
		pub pallet_locked: bool,
	}

	#[derive(PartialEq)]
	enum Operation {
		Claim,
		Cancel,
		Snipe,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<ContractKey, ItemId> {
		fn contract_key(i: u32) -> ContractKey;
		fn contract_value(i: u64) -> u64;
		fn item_id(i: u16) -> ItemId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<ContractKey: From<u32>, ItemId: From<[u8; 32]>> BenchmarkHelper<ContractKey, ItemId> for () {
		fn contract_key(i: u32) -> ContractKey {
			i.into()
		}
		fn contract_value(i: u64) -> u64 {
			i
		}
		fn item_id(i: u16) -> ItemId {
			let mut id = [0_u8; 32];
			let bytes = i.to_be_bytes();
			id[0] = bytes[0];
			id[1] = bytes[1];
			id.into()
		}
	}

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

		/// The maximum number of contracts an account can have.
		#[pallet::constant]
		type MaxContracts: Get<u32>;

		/// The maximum number of staking clauses a contract can have.
		#[pallet::constant]
		type MaxStakingClauses: Get<u32>;

		/// The maximum number of fee clauses a contract can have.
		#[pallet::constant]
		type MaxFeeClauses: Get<u32>;

		/// The maximum number of bytes used for a contract's metadata.
		#[pallet::constant]
		type MaxMetadataLength: Get<u32>;

		/// Type of the contract's attribute keys, used on contract condition evaluation
		type AttributeKey: Member + Encode + Decode + MaxEncodedLen + TypeInfo;

		/// The maximum length of an attribute value in bytes.
		type AttributeValueLimit: sp_std::fmt::Debug
			+ Clone
			+ Encode
			+ Decode
			+ MaxEncodedLen
			+ TypeInfo
			+ Get<u32>;

		/// A set of helper functions for benchmarking.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::AttributeKey, Self::ItemId>;

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
	pub type ContractsMetadata<T: Config> =
		StorageMap<_, Identity, T::ItemId, BoundedVec<u8, T::MaxMetadataLength>>;

	#[pallet::storage]
	pub type ContractAccepted<T: Config> = StorageMap<_, Identity, T::ItemId, T::BlockNumber>;

	#[pallet::storage]
	pub type ContractStakedItems<T: Config> = StorageMap<_, Identity, T::ItemId, StakedItemsOf<T>>;

	#[pallet::storage]
	pub type ContractIds<T: Config> = StorageMap<_, Identity, T::AccountId, ContractIdsOf<T>>;

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
		/// The contract is staking hence cannot be claimed or sniped.
		Staking,
		/// The contract is expired hence cannot be claimed.
		Expired,
		/// The contract is claimable, so it cannot be cancelled or sniped.
		Claimable,
		/// The contract is available, or not yet accepted.
		Available,
		/// The number of the given account's contracts exceeds maximum allowed.
		MaxContracts,
		/// The number of the given contract's staking clauses exceeds maximum allowed.
		MaxStakingClauses,
		/// The number of the given contract's fee clauses exceeds maximum allowed.
		MaxFeeClauses,
		/// The number of staked NFTs doesn't match the contract specs.
		InvalidNFTStakeAmount,
		/// The number of fee NFTs doesn't match the contract specs.
		InvalidNFTFeeAmount,
		/// Metadata for the contract is too long.
		MetadataTooLong,
		/// The given account does not hold any contracts.
		NotContractHolder,
		/// The given account does not meet the criteria to be a sniper.
		NotSniper,
		/// The given account attempts to snipe its own contract.
		CannotSnipeOwnContract,
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
			metadata: Option<Str>,
		) -> DispatchResult {
			let creator = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_contract_clauses(&contract)?;
			Self::create_contract(creator, contract_id, contract, metadata)
		}

		/// Remove a staking contract.
		///
		/// This call enables the creator to remove inactive staking contracts that haven't been
		/// accepted by any staker. This can be done to clean up the available staking contracts or
		/// to adjust the parameters before re-creating the contract.
		#[pallet::weight(
			T::WeightInfo::remove_token_reward()
				.max(T::WeightInfo::remove_nft_reward())
		)]
		#[pallet::call_index(4)]
		pub fn remove(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let _ = Self::ensure_creator(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_removable(&contract_id)?;
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
			stakes: Vec<NftIdOf<T>>,
			fees: Vec<NftIdOf<T>>,
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
		#[pallet::weight(
			T::WeightInfo::cancel_token_reward()
				.max(T::WeightInfo::cancel_nft_reward())
		)]
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
		#[pallet::weight(
			T::WeightInfo::snipe_token_reward()
				.max(T::WeightInfo::snipe_nft_reward())
		)]
		#[pallet::call_index(8)]
		pub fn snipe(origin: OriginFor<T>, contract_id: T::ItemId) -> DispatchResult {
			let sniper = ensure_signed(origin)?;
			Self::ensure_pallet_unlocked()?;
			Self::ensure_sniper(&sniper)?;
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
			metadata: Option<Str>,
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
					let NftId(collection_id, item_id) = address;
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

			if let Some(msg) = metadata {
				let msg_bytes: BoundedVec<_, _> =
					msg.into_bytes().try_into().map_err(|_| Error::<T>::MetadataTooLong)?;
				ContractsMetadata::<T>::insert(contract_id, msg_bytes);
			}

			Self::deposit_event(Event::<T>::Created { contract_id });
			Ok(())
		}

		fn remove_contract(contract_id: T::ItemId) -> DispatchResult {
			Contracts::<T>::remove(contract_id);
			Self::deposit_event(Event::<T>::Removed { contract_id });
			Ok(())
		}

		pub(crate) fn accept_contract(
			contract_id: T::ItemId,
			who: T::AccountId,
			stake_addresses: &[NftIdOf<T>],
			fee_addresses: &[NftIdOf<T>],
		) -> DispatchResult {
			// Transfer contract, stake and fee NFTs.
			let contract_address = NftId(Self::contract_collection_id()?, contract_id);
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

			// Record contract holder.
			ContractIds::<T>::try_append(&who, contract_id)
				.map_err(|_| Error::<T>::MaxContracts)?;

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
				Reward::Nft(NftId(collection_id, item_id)) =>
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

			// Retain contract IDs held.
			let mut contract_ids = Self::contract_ids(&contract_owner)?;
			contract_ids.retain(|id| id != &contract_id);
			if contract_ids.is_empty() {
				ContractIds::<T>::remove(contract_owner);
			} else {
				ContractIds::<T>::insert(contract_owner, contract_ids);
			}

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

		fn transfer_items(addresses: &[NftIdOf<T>], to: &T::AccountId) -> DispatchResult {
			addresses.iter().try_for_each(|NftId(collection_id, item_id)| {
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
			is_snipe: bool,
		) -> Result<ContractOf<T>, DispatchError> {
			let owner = Self::contract_owner(contract_id)?;
			if is_snipe {
				ensure!(owner != Self::account_id(), Error::<T>::Available);
				ensure!(who != &owner, Error::<T>::CannotSnipeOwnContract);
			} else {
				ensure!(who == &owner, Error::<T>::ContractOwnership);
			}
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

		fn ensure_removable(contract_id: &T::ItemId) -> DispatchResult {
			Self::ensure_contract_ownership(contract_id, &Self::account_id(), false).map_err(
				|err| {
					if err == Error::<T>::ContractOwnership.into() {
						Error::<T>::Staking.into()
					} else {
						err
					}
				},
			)?;
			Ok(())
		}

		fn ensure_acceptable(
			contract_id: &T::ItemId,
			who: &T::AccountId,
			stake_addresses: &[NftIdOf<T>],
			fee_addresses: &[NftIdOf<T>],
		) -> DispatchResult {
			let contract =
				Self::ensure_contract_ownership(contract_id, &Self::account_id(), false)?;

			ensure!(
				Self::count_consecutive_unique_nft(stake_addresses) == contract.nft_stake_amount,
				Error::<T>::InvalidNFTStakeAmount
			);
			ensure!(
				Self::count_consecutive_unique_nft(fee_addresses) == contract.nft_fee_amount,
				Error::<T>::InvalidNFTFeeAmount
			);

			let activation = contract.activation.ok_or(Error::<T>::UnknownActivation)?;
			let now = <frame_system::Pallet<T>>::block_number();
			let inactive = activation
				.checked_add(&contract.active_duration)
				.ok_or(ArithmeticError::Overflow)?;
			ensure!(now >= activation && now <= inactive, Error::<T>::Inactive);

			stake_addresses.iter().try_for_each(|NftId(collection_id, contract_id)| {
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
			let Contract { claim_duration, stake_duration, .. } =
				Self::ensure_contract_ownership(contract_id, who, op == Operation::Snipe)?;
			let now = <frame_system::Pallet<T>>::block_number();
			let accepted = Self::contract_accepted(contract_id)?;
			let end = accepted.checked_add(&stake_duration).ok_or(ArithmeticError::Overflow)?;
			let expiry = end.checked_add(&claim_duration).ok_or(ArithmeticError::Overflow)?;

			match op {
				Operation::Claim => {
					ensure!(now <= expiry, Error::<T>::Expired);
					ensure!(now >= end, Error::<T>::Staking);
				},
				Operation::Cancel => {
					ensure!(now <= expiry, Error::<T>::Expired);
					ensure!(now < end, Error::<T>::Claimable);
				},
				Operation::Snipe => {
					ensure!(now >= end, Error::<T>::Staking);
					ensure!(now > expiry, Error::<T>::Claimable);
				},
			}
			Ok(())
		}

		fn ensure_sniper(sniper: &T::AccountId) -> DispatchResult {
			let contract_ids = Self::contract_ids(sniper).map_err(|_| Error::<T>::NotSniper)?;
			let now = <frame_system::Pallet<T>>::block_number();

			for contract_id in contract_ids {
				let Contract { stake_duration, .. } = Self::contract(&contract_id)?;
				let accepted = Self::contract_accepted(&contract_id)?;
				let end = accepted.checked_add(&stake_duration).ok_or(ArithmeticError::Overflow)?;
				// Not a sniper if any contracts are in claimable phase.
				if now > end {
					return Err(Error::<T>::NotSniper.into())
				}
			}
			Ok(())
		}

		#[inline]
		fn count_consecutive_unique_nft(nft_set: &[NftIdOf<T>]) -> u8 {
			nft_set
				.iter()
				.map(|i| i.encode())
				.collect::<sp_std::collections::btree_set::BTreeSet<_>>()
				.len() as u8
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
		fn contract_ids(who: &T::AccountId) -> Result<ContractIdsOf<T>, DispatchError> {
			let contract_ids = ContractIds::<T>::get(who).ok_or(Error::<T>::NotContractHolder)?;
			Ok(contract_ids)
		}
	}
}

sp_core::generate_feature_enabled_macro!(runtime_benchmarks_enabled, feature = "runtime-benchmarks", $);
