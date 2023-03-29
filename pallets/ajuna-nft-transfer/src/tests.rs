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

use crate::{mock::*, traits::*, Error, *};
use codec::{Decode, Encode};
use frame_support::{
	assert_noop, assert_ok,
	traits::tokens::{
		nonfungibles_v2::{Create, Inspect},
		AttributeNamespace,
	},
	BoundedVec,
};
use sp_runtime::testing::H256;

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
	const ITEM_CODE: ItemCode = 1;

	fn get_attribute_codes() -> Vec<AttributeCode> {
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
	fn can_store_item_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockStruct::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(
				BOB,
				collection_id,
				item_id,
				item.clone(),
				item_config
			));

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(crate::Event::ItemStored {
				collection_id,
				item_id,
				owner: BOB,
			}));

			assert_eq!(
				LockItemStatus::<Test>::get(collection_id, item_id),
				Some(NftStatus::Stored)
			);

			assert_eq!(NftClaimants::<Test>::get(collection_id, item_id), Some(BOB));

			let stored_item =
				<Test as crate::pallet::Config>::NftHelper::typed_attribute::<
					ItemCode,
					EncodedItemOf<Test>,
				>(&collection_id, &item_id, &AttributeNamespace::Pallet, &MockStruct::ITEM_CODE)
				.map(|item| item.into_inner());

			assert_eq!(stored_item, Some(item.encode_into()))
		});
	}

	#[test]
	fn cannot_store_duplicates_under_same_collection() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockStruct::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(
				ALICE,
				collection_id,
				item_id,
				item.clone(),
				item_config
			));
			assert_noop!(
				NftTransfer::store_as_nft(ALICE, collection_id, item_id, item, item_config),
				pallet_nfts::Error::<Test>::AlreadyExists
			);
		});
	}

	#[test]
	fn cannot_store_item_above_encoding_limit() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockStruct { data: vec![1; MAX_ENCODING_SIZE as usize] };
			let item_config = pallet_nfts::ItemConfig::default();

			assert_noop!(
				NftTransfer::store_as_nft(BOB, collection_id, item_id, item, item_config),
				Error::<Test>::ItemSizeAboveEncodingLimit
			);
		});
	}
}

mod recover_from_nft {
	use super::*;

	#[test]
	fn can_recover_item_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockStruct::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(
				BOB,
				collection_id,
				item_id,
				item.clone(),
				item_config,
			));

			let result = NftTransfer::recover_from_nft(BOB, collection_id, item_id);

			assert_eq!(result, Ok(item));

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(
				crate::Event::ItemRestored { collection_id, item_id, owner: BOB },
			));

			assert_eq!(LockItemStatus::<Test>::get(collection_id, item_id), None);

			let stored_item =
				<Test as crate::pallet::Config>::NftHelper::typed_attribute::<
					ItemCode,
					EncodedItemOf<Test>,
				>(&collection_id, &item_id, &AttributeNamespace::Pallet, &MockStruct::ITEM_CODE);

			assert_eq!(stored_item, None);
		});
	}

	#[test]
	fn cannot_restore_uploaded_item() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockStruct::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(BOB, collection_id, item_id, item, item_config));

			LockItemStatus::<Test>::insert(collection_id, item_id, NftStatus::Uploaded);

			let result: Result<MockStruct, _> =
				NftTransfer::recover_from_nft(BOB, collection_id, item_id);

			assert_noop!(result, Error::<Test>::NftOutsideOfChain);
		});
	}

	#[test]
	fn cannot_restore_nft_if_not_claimant() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockStruct::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(BOB, collection_id, item_id, item, item_config));

			let result: Result<MockStruct, _> =
				NftTransfer::recover_from_nft(ALICE, collection_id, item_id);

			assert_noop!(result, Error::<Test>::NftNotOwned);
		});
	}
}
