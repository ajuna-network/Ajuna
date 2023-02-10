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
				nonfungibles_v2::{Create, Destroy, Inspect, Mutate},
				AttributeNamespace,
			},
			Locker,
		},
	};
	use sp_runtime::{traits::AtLeast32BitUnsigned, Saturating};

	pub type EncodedAssetOf<T> = BoundedVec<u8, <T as Config>::MaxAssetEncodedSize>;

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximum amount of bytes that an asset may be encoded as.
		#[pallet::constant]
		type MaxAssetEncodedSize: Get<u32>;

		/// Identifier for the collection of an NFT.
		type CollectionId: Member + Parameter + Copy + MaxEncodedLen + AtLeast32BitUnsigned;

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
		type ItemId: Member + Parameter + Default + Copy + MaxEncodedLen + AtLeast32BitUnsigned;

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
			+ Destroy<Self::AccountId>;
	}

	#[pallet::storage]
	pub(crate) type NextItemId<T: Config> =
		StorageMap<_, Identity, T::CollectionId, T::ItemId, ValueQuery>;

	#[pallet::storage]
	pub(crate) type LockItemStatus<T: Config> =
		StorageDoubleMap<_, Identity, T::CollectionId, Identity, T::ItemId, NftStatus, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Asset has been stored as an NFT [collection_id, asset_id, owner]
		AssetStored { collection_id: T::CollectionId, asset_id: T::ItemId, owner: T::AccountId },
		/// Asset has been restored back from its NFT representation [collection_id, asset_id,
		/// owner]
		AssetRestored { collection_id: T::CollectionId, asset_id: T::ItemId, owner: T::AccountId },
		/// Asset has been transferred outside the chain [collection_id, asset_id, owner]
		AssetTransferred {
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The given asset resulted in an encoded size larger that the defined encoding limit.
		AssetSizeAboveEncodingLimit,
		/// The given NFT id didn't match any entries for the specified collection.
		NftNotFound,
		/// The given NFT id doesn't have the proper attribute set.
		NftAttributeMissing,
		/// The given NFT is not owned by the requester.
		NftNotOwned,
		/// The given NFT is currently outside of the chain, transfer it back before attempting a
		/// restore.
		NftOutsideOfChain,
		/// The process of restoring an NFT into an Asset has failed.
		AssetRestoreFailure,
	}

	impl<T: Config, Asset: NftConvertible> NftHandler<T::AccountId, Asset> for Pallet<T> {
		type CollectionId = T::CollectionId;
		type AssetId = T::ItemId;
		type AssetConfig = T::ItemConfig;

		fn store_as_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			asset: Asset,
			asset_config: Option<Self::AssetConfig>,
		) -> Result<Self::AssetId, DispatchError> {
			let encoded_asset: EncodedAssetOf<T> = asset
				.encode_into()
				.try_into()
				.map_err(|_| Error::<T>::AssetSizeAboveEncodingLimit)?;

			let next_item_id = NextItemId::<T>::mutate(collection_id, |item_id| {
				let next_item_id = *item_id;
				item_id.saturating_inc();
				next_item_id
			});

			T::NftHelper::mint_into(
				&collection_id,
				&next_item_id,
				&owner,
				&asset_config.unwrap_or_default(),
				true,
			)?;
			T::NftHelper::set_typed_attribute(
				&collection_id,
				&next_item_id,
				&Asset::get_asset_code(),
				&encoded_asset,
			)?;
			LockItemStatus::<T>::insert(collection_id, next_item_id, NftStatus::Stored);

			Self::deposit_event(Event::<T>::AssetStored {
				collection_id,
				asset_id: next_item_id,
				owner,
			});

			Ok(next_item_id)
		}

		fn recover_from_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			asset_id: Self::AssetId,
		) -> Result<Asset, DispatchError> {
			let nft_owner =
				T::NftHelper::owner(&collection_id, &asset_id).ok_or(Error::<T>::NftNotFound)?;

			ensure!(nft_owner == owner, Error::<T>::NftNotOwned);
			ensure!(
				LockItemStatus::<T>::get(collection_id, asset_id) == Some(NftStatus::Stored),
				Error::<T>::NftOutsideOfChain
			);

			let encoded_asset_data = T::NftHelper::typed_attribute::<AssetCode, EncodedAssetOf<T>>(
				&collection_id,
				&asset_id,
				&AttributeNamespace::Pallet,
				&Asset::get_asset_code(),
			)
			.ok_or(Error::<T>::NftAttributeMissing)?;

			let asset = Asset::decode_from(encoded_asset_data.into_inner())
				.map_err(|_| Error::<T>::AssetRestoreFailure)?;

			T::NftHelper::clear_typed_attribute(
				&collection_id,
				&asset_id,
				&Asset::get_asset_code(),
			)?;
			T::NftHelper::burn(&collection_id, &asset_id, Some(&owner))?;
			LockItemStatus::<T>::remove(collection_id, asset_id);

			Self::deposit_event(Event::<T>::AssetRestored { collection_id, asset_id, owner });

			Ok(asset)
		}

		fn schedule_nft_upload(
			_owner: T::AccountId,
			_collection_id: Self::CollectionId,
			_asset_id: Self::AssetId,
		) -> DispatchResult {
			todo!()
		}
	}

	impl<T: Config> Locker<T::CollectionId, T::ItemId> for Pallet<T> {
		fn is_locked(collection: T::CollectionId, item: T::ItemId) -> bool {
			LockItemStatus::<T>::get(collection, item).is_some()
		}
	}
}
