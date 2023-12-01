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
			tokens::nonfungibles_v2::{Inspect, Mutate},
			Locker,
		},
		PalletId,
	};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
	pub enum NftStatus {
		/// The NFT exists in storage in the chain
		Stored,
		/// The NFT has been uploaded outside the chain
		Uploaded,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The NFT-transfer's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Identifier for the collection of item.
		type CollectionId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		/// The type used to identify a unique item within a collection.
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		/// Type that holds the specific configurations for an item.
		type ItemConfig: Default + MaxEncodedLen + TypeInfo;

		/// The maximum length of an attribute key.
		#[pallet::constant]
		type KeyLimit: Get<u32>;

		/// The maximum length of an attribute value.
		#[pallet::constant]
		type ValueLimit: Get<u32>;

		/// An NFT helper for the management of collections and items.
		type NftHelper: Inspect<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Mutate<Self::AccountId, Self::ItemConfig>;
	}

	#[pallet::storage]
	pub type NftStatuses<T: Config> =
		StorageDoubleMap<_, Identity, T::CollectionId, Identity, T::ItemId, NftStatus, OptionQuery>;

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
		/// IPFS URL must not be an empty string.
		EmptyIpfsUrl,
		/// Item code must be different to attribute codes.
		DuplicateItemCode,
		/// The given NFT item doesn't exist.
		UnknownItem,
		/// The given claim doesn't exist.
		UnknownClaim,
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

	impl<T: Config, Item: NftConvertible<T::KeyLimit, T::ValueLimit>>
		NftHandler<T::AccountId, T::ItemId, T::KeyLimit, T::ValueLimit, Item> for Pallet<T>
	{
		type CollectionId = T::CollectionId;

		fn store_as_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			item_id: T::ItemId,
			item: Item,
			ipfs_url: IpfsUrl,
		) -> DispatchResult {
			let config = T::ItemConfig::default();
			T::NftHelper::mint_into(&collection_id, &item_id, &owner, &config, false)?;
			T::NftHelper::set_attribute(
				&collection_id,
				&item_id,
				Item::ITEM_CODE,
				item.encode().as_slice(),
			)?;

			ensure!(!ipfs_url.is_empty(), Error::<T>::EmptyIpfsUrl);
			T::NftHelper::set_attribute(
				&collection_id,
				&item_id,
				Item::IPFS_URL_CODE,
				ipfs_url.as_slice(),
			)?;

			item.get_encoded_attributes()
				.iter()
				.try_for_each(|(attribute_code, attribute)| {
					ensure!(
						attribute_code.as_slice() != Item::ITEM_CODE,
						Error::<T>::DuplicateItemCode
					);
					T::NftHelper::set_attribute(&collection_id, &item_id, attribute_code, attribute)
				})?;

			NftStatuses::<T>::insert(collection_id, item_id, NftStatus::Stored);

			Self::deposit_event(Event::<T>::ItemStored { collection_id, item_id, owner });
			Ok(())
		}

		fn recover_from_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			item_id: T::ItemId,
		) -> Result<Item, DispatchError> {
			ensure!(
				NftStatuses::<T>::get(collection_id, item_id) == Some(NftStatus::Stored),
				Error::<T>::NftOutsideOfChain
			);

			let item =
				T::NftHelper::system_attribute(&collection_id, Some(&item_id), Item::ITEM_CODE)
					.ok_or(Error::<T>::UnknownItem)?;

			T::NftHelper::clear_attribute(&collection_id, &item_id, Item::ITEM_CODE)?;
			T::NftHelper::clear_attribute(&collection_id, &item_id, Item::IPFS_URL_CODE)?;
			for attribute_key in Item::get_attribute_codes() {
				T::NftHelper::clear_attribute(&collection_id, &item_id, &attribute_key)?;
			}

			NftStatuses::<T>::remove(collection_id, item_id);
			T::NftHelper::burn(&collection_id, &item_id, Some(&owner))?;

			Self::deposit_event(Event::<T>::ItemRestored { collection_id, item_id, owner });
			Item::decode(&mut item.as_slice()).map_err(|_| Error::<T>::ItemRestoreFailure.into())
		}
	}

	impl<T: Config> Locker<T::CollectionId, T::ItemId> for Pallet<T> {
		fn is_locked(collection_id: T::CollectionId, item_id: T::ItemId) -> bool {
			matches!(NftStatuses::<T>::get(collection_id, item_id), Some(NftStatus::Uploaded))
		}
	}
}
