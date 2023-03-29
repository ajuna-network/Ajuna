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

use crate::{
	mock::*,
	traits::{AssetCode, NftConvertible, *},
	Error, *,
};
use codec::{Decode, Encode};
use frame_support::{
	assert_noop, assert_ok,
	traits::tokens::{
		nonfungibles_v2::{Create, Inspect},
		AttributeNamespace,
	},
	BoundedVec,
};

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
struct MockStruct {
	data: Vec<u8>,
}

impl Default for MockStruct {
	fn default() -> Self {
		Self { data: vec![1; 32] }
	}
}

impl NftConvertible for MockStruct {
	const ASSET_CODE: AssetCode = 1;

	fn get_attribute_table() -> Vec<AttributeCode> {
		vec![10]
	}

	fn get_encoded_attributes(&self) -> Vec<(AttributeCode, Vec<u8>)> {
		vec![(10, vec![1; 1])]
	}
}

type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, MockBlockNumber, MockCollectionId>;

fn create_collection(organizer: MockAccountId) -> MockCollectionId {
	<Test as crate::pallet::Config>::NftHelper::create_collection(
		&organizer,
		&NftTransfer::account_id(),
		&CollectionConfig::default(),
	)
	.expect("Should have create contract collection")
}

#[test]
fn attributes_cannot_be_mutated_via_extrinsic() {
	let k = <BoundedVec<u8, KeyLimit>>::try_from(b"key".to_vec()).unwrap();
	let v = <BoundedVec<u8, ValueLimit>>::try_from(b"value".to_vec()).unwrap();

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Nft::create(RuntimeOrigin::signed(ALICE), ALICE, CollectionConfig::default()));
		for ns in [
			AttributeNamespace::Pallet,
			AttributeNamespace::CollectionOwner,
			AttributeNamespace::ItemOwner,
			AttributeNamespace::<MockAccountId>::Account(ALICE),
		] {
			assert_ok!(Nft::force_set_attribute(
				RuntimeOrigin::root(),
				None,
				0,
				None,
				ns.clone(),
				k.clone(),
				v.clone()
			));
			assert_noop!(
				Nft::set_attribute(RuntimeOrigin::signed(ALICE), 0, None, ns, k.clone(), v.clone()),
				pallet_nfts::Error::<Test>::MethodDisabled
			);
		}
	});
}

mod store_as_nft {
	use super::*;

	#[test]
	fn asset_properly_stored() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let asset = MockStruct::default();
			let asset_config = pallet_nfts::ItemConfig::default();

			let result = NftTransfer::store_as_nft(BOB, collection_id, asset.clone(), asset_config);

			assert_ok!(result);

			let asset_id = result.unwrap();

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(crate::Event::AssetStored {
				collection_id,
				asset_id,
				owner: BOB,
			}));

			assert_eq!(
				LockItemStatus::<Test>::get(collection_id, asset_id),
				Some(NftStatus::Stored)
			);

			assert_eq!(NftClaimants::<Test>::get(collection_id, asset_id), Some(BOB));

			let stored_asset =
				<Test as crate::pallet::Config>::NftHelper::typed_attribute::<
					AssetCode,
					EncodedAssetOf<Test>,
				>(&collection_id, &asset_id, &AttributeNamespace::Pallet, &MockStruct::ASSET_CODE)
				.map(|item| item.into_inner());

			assert_eq!(stored_asset, Some(asset.encode_into()))
		});
	}

	#[test]
	fn cannot_store_asset_above_encoding_limit() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let asset = MockStruct { data: vec![1; MAX_ENCODING_SIZE as usize] };
			let asset_config = pallet_nfts::ItemConfig::default();

			assert_noop!(
				NftTransfer::store_as_nft(BOB, collection_id, asset, asset_config),
				Error::<Test>::AssetSizeAboveEncodingLimit
			);
		});
	}
}

mod recover_from_nft {
	use super::*;

	#[test]
	fn asset_properly_recovered() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let asset = MockStruct::default();
			let asset_config = pallet_nfts::ItemConfig::default();

			let asset_id =
				NftTransfer::store_as_nft(BOB, collection_id, asset.clone(), asset_config)
					.expect("Storage should have been successful!");

			let result = NftTransfer::recover_from_nft(BOB, collection_id, asset_id);

			assert_eq!(result, Ok(asset));

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(
				crate::Event::AssetRestored { collection_id, asset_id, owner: BOB },
			));

			assert_eq!(LockItemStatus::<Test>::get(collection_id, asset_id), None);

			let stored_asset =
				<Test as crate::pallet::Config>::NftHelper::typed_attribute::<
					AssetCode,
					EncodedAssetOf<Test>,
				>(&collection_id, &asset_id, &AttributeNamespace::Pallet, &MockStruct::ASSET_CODE);

			assert_eq!(stored_asset, None);
		});
	}

	#[test]
	fn cannot_restore_uploaded_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let asset = MockStruct::default();
			let asset_config = pallet_nfts::ItemConfig::default();

			let asset_id = NftTransfer::store_as_nft(BOB, collection_id, asset, asset_config)
				.expect("Storage should have been successful!");

			LockItemStatus::<Test>::insert(collection_id, asset_id, NftStatus::Uploaded);

			let result: Result<MockStruct, _> =
				NftTransfer::recover_from_nft(BOB, collection_id, asset_id);

			assert_noop!(result, Error::<Test>::NftOutsideOfChain);
		});
	}

	#[test]
	fn cannot_restore_nft_if_not_claimant() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let asset = MockStruct::default();
			let asset_config = pallet_nfts::ItemConfig::default();

			let asset_id = NftTransfer::store_as_nft(BOB, collection_id, asset, asset_config)
				.expect("Storage should have been successful!");

			let result: Result<MockStruct, _> =
				NftTransfer::recover_from_nft(ALICE, collection_id, asset_id);

			assert_noop!(result, Error::<Test>::NftNotOwned);
		});
	}
}
