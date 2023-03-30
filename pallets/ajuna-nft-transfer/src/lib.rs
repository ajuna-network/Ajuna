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

pub mod traits;

#[frame_support::pallet]
pub mod pallet {
	use crate::traits::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::{
				nonfungibles_v2::{Destroy, Inspect, Mutate},
				AttributeNamespace,
			},
			Locker,
		},
		PalletId,
	};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	pub type EncodedItemOf<T> = BoundedVec<u8, <T as Config>::MaxItemEncodedSize>;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
	pub enum NftStatus {
		/// The NFT exists in storage in the chain
		Stored,
		/// The NFT has been uploaded outside the chain
		Uploaded,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The NFT-transfer's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximum amount of bytes that an item may be encoded as.
		#[pallet::constant]
		type MaxItemEncodedSize: Get<u32>;

		/// Identifier for the collection of item.
		type CollectionId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		/// The type used to identify a unique item within a collection.
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		/// Type that holds the specific configurations for an item.
		type ItemConfig: Copy
			+ Clone
			+ Default
			+ PartialEq
			+ Encode
			+ Decode
			+ MaxEncodedLen
			+ TypeInfo;

		/// An NFT helper for the management of collections and items.
		type NftHelper: Inspect<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Mutate<Self::AccountId, Self::ItemConfig>
			+ Destroy<Self::AccountId>;
	}

	#[pallet::storage]
	pub type LockItemStatus<T: Config> =
		StorageDoubleMap<_, Identity, T::CollectionId, Identity, T::ItemId, NftStatus, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nft_claimants)]
	pub type NftClaimants<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::CollectionId,
		Identity,
		T::ItemId,
		T::AccountId,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Item has been stored as an NFT [collection_id, item_id, owner]
		ItemStored { collection_id: T::CollectionId, item_id: T::ItemId, owner: T::AccountId },
		/// Item has been restored back from its NFT representation [collection_id, item_id, owner]
		ItemRestored { collection_id: T::CollectionId, item_id: T::ItemId, owner: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The given item resulted in an encoded size larger that the defined encoding limit.
		ItemSizeAboveEncodingLimit,
		/// The given NFT id didn't match any entries for the specified collection.
		NftNotFound,
		/// The given NFT id doesn't have the proper attribute set.
		NftAttributeMissing,
		/// The given NFT is not owned by the requester.
		NftNotOwned,
		/// The given NFT is currently outside of the chain, transfer it back before attempting a
		/// restore.
		NftOutsideOfChain,
		/// The process of restoring an NFT into an item has failed.
		ItemRestoreFailure,
	}

	impl<T: Config> Pallet<T> {
		/// The account identifier to delegate NFT transfer operations.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	impl<T: Config, Item: NftConvertible> NftHandler<T::AccountId, T::ItemId, Item, T::ItemConfig>
		for Pallet<T>
	{
		type CollectionId = T::CollectionId;

		fn store_as_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			item_id: T::ItemId,
			item: Item,
			item_config: T::ItemConfig,
		) -> DispatchResult {
			let encoded_attributes = item.get_encoded_attributes();
			let encoded_item: EncodedItemOf<T> = item
				.encode_into()
				.try_into()
				.map_err(|_| Error::<T>::ItemSizeAboveEncodingLimit)?;

			T::NftHelper::mint_into(&collection_id, &item_id, &owner, &item_config, true)?;
			T::NftHelper::set_typed_attribute(
				&collection_id,
				&item_id,
				&Item::ITEM_CODE,
				&encoded_item,
			)?;
			encoded_attributes.iter().try_for_each(|(attribute_key, attribute)| {
				T::NftHelper::set_typed_attribute(
					&collection_id,
					&item_id,
					&attribute_key,
					&attribute,
				)
			})?;

			LockItemStatus::<T>::insert(collection_id, item_id, NftStatus::Stored);
			NftClaimants::<T>::insert(collection_id, item_id, &owner);

			Self::deposit_event(Event::<T>::ItemStored { collection_id, item_id, owner });
			Ok(())
		}

		fn recover_from_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			item_id: T::ItemId,
		) -> Result<Item, DispatchError> {
			ensure!(
				NftClaimants::<T>::get(collection_id, item_id) == Some(owner.clone()),
				Error::<T>::NftNotOwned
			);
			ensure!(
				LockItemStatus::<T>::get(collection_id, item_id) == Some(NftStatus::Stored),
				Error::<T>::NftOutsideOfChain
			);

			let encoded_item = T::NftHelper::typed_attribute::<ItemCode, EncodedItemOf<T>>(
				&collection_id,
				&item_id,
				&AttributeNamespace::Pallet,
				&Item::ITEM_CODE,
			)
			.ok_or(Error::<T>::NftAttributeMissing)?;

			let item = Item::decode_from(encoded_item.into_inner())
				.map_err(|_| Error::<T>::ItemRestoreFailure)?;

			T::NftHelper::clear_typed_attribute(&collection_id, &item_id, &Item::ITEM_CODE)?;

			for attribute_key in Item::get_attribute_codes() {
				T::NftHelper::clear_typed_attribute(&collection_id, &item_id, &attribute_key)?;
			}

			T::NftHelper::burn(&collection_id, &item_id, Some(&owner))?;
			LockItemStatus::<T>::remove(collection_id, item_id);

			Self::deposit_event(Event::<T>::ItemRestored { collection_id, item_id, owner });
			Ok(item)
		}

		fn schedule_upload(
			_owner: T::AccountId,
			_collection_id: Self::CollectionId,
			_item_id: T::ItemId,
		) -> DispatchResult {
			todo!()
		}
	}

	impl<T: Config> Locker<T::CollectionId, T::ItemId> for Pallet<T> {
		fn is_locked(collection_id: T::CollectionId, item_id: T::ItemId) -> bool {
			LockItemStatus::<T>::contains_key(collection_id, item_id)
		}
	}
}
