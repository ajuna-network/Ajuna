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
	use codec::HasCompact;
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
	pub enum NFTStatus {
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

		#[pallet::constant]
		/// Maximum amount of bytes that an asset may be encoded as.
		type MaxAssetEncodedSize: Get<u32>;

		/// Identifier for the collection of an NFT.
		type CollectionId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ From<u32>
			+ Into<u32>
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
			+ From<u128>
			+ Into<u128>
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
			+ Destroy<Self::AccountId>;
	}

	#[pallet::storage]
	pub(crate) type NextItemId<T: Config> =
		StorageMap<_, Identity, T::CollectionId, T::ItemId, ValueQuery>;

	#[pallet::storage]
	pub(crate) type LockItemStatus<T: Config> =
		StorageDoubleMap<_, Identity, T::CollectionId, Identity, T::ItemId, NFTStatus, OptionQuery>;

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
		NFTNotFound,
		/// The given NFT id doesn't have the proper attribute set.
		NFTAttributeMissing,
		/// The given NFT is not owned by the requester.
		NFTNotOwned,
		/// The given NFT is currently outside of the chain, transfer it back before attempting a
		/// restore.
		NFTOutsideOfChain,
		/// The process of restoring an NFT into an Asset has failed.
		AssetRestoreFailure,
	}

	impl<T: Config, Asset: NFTConvertible>
		NFTHandler<T::AccountId, T::CollectionId, T::ItemId, Asset, T::ItemConfig> for Pallet<T>
	{
		fn store_as_nft(
			owner: T::AccountId,
			collection_id: T::CollectionId,
			asset: Asset,
			asset_config: Option<T::ItemConfig>,
		) -> Result<T::ItemId, DispatchError> {
			let encoded_asset: EncodedAssetOf<T> = asset
				.encode_into()
				.try_into()
				.map_err(|_| Error::<T>::AssetSizeAboveEncodingLimit)?;

			let next_id = NextItemId::<T>::mutate(collection_id, |item| {
				let next_id = *item;
				item.saturating_inc();
				next_id
			});

			T::NFTHelper::mint_into(
				&collection_id,
				&next_id,
				&owner,
				&asset_config.unwrap_or_default(),
				true,
			)?;
			T::NFTHelper::set_typed_attribute(
				&collection_id,
				&next_id,
				&Asset::get_asset_code(),
				&encoded_asset,
			)?;
			LockItemStatus::<T>::insert(collection_id, next_id, NFTStatus::Stored);

			Self::deposit_event(Event::<T>::AssetStored {
				collection_id,
				asset_id: next_id,
				owner,
			});

			Ok(next_id)
		}

		fn recover_from_nft(
			owner: T::AccountId,
			collection_id: T::CollectionId,
			nft_id: T::ItemId,
		) -> Result<Asset, DispatchError> {
			let nft_owner = T::NFTHelper::owner(&collection_id, &nft_id);

			ensure!(nft_owner.is_some(), Error::<T>::NFTNotFound);
			ensure!(nft_owner.unwrap() == owner, Error::<T>::NFTNotOwned);
			ensure!(
				LockItemStatus::<T>::get(collection_id, nft_id) == Some(NFTStatus::Stored),
				Error::<T>::NFTOutsideOfChain
			);

			let encoded_nft_data = T::NFTHelper::typed_attribute::<AssetCode, EncodedAssetOf<T>>(
				&collection_id,
				&nft_id,
				&AttributeNamespace::Pallet,
				&Asset::get_asset_code(),
			)
			.ok_or(Error::<T>::NFTAttributeMissing)?;

			let asset = Asset::decode_from(encoded_nft_data.into_inner())
				.map_err(|_| Error::<T>::AssetRestoreFailure)?;

			T::NFTHelper::clear_typed_attribute(&collection_id, &nft_id, &Asset::get_asset_code())?;
			T::NFTHelper::burn(&collection_id, &nft_id, Some(&owner))?;
			LockItemStatus::<T>::remove(collection_id, nft_id);

			Self::deposit_event(Event::<T>::AssetRestored {
				collection_id,
				asset_id: nft_id,
				owner,
			});

			Ok(asset)
		}

		fn schedule_nft_upload(
			_owner: T::AccountId,
			_collection_id: T::CollectionId,
			_nft_id: T::ItemId,
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
