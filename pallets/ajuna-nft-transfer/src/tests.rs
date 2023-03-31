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
};
use sp_runtime::testing::H256;

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
struct MockItem {
	field_1: Vec<u8>,
	field_2: u16,
	field_3: bool,
}

impl Default for MockItem {
	fn default() -> Self {
		Self { field_1: vec![1, 2, 3], field_2: 333, field_3: true }
	}
}

impl NftConvertible for MockItem {
	const ITEM_CODE: ItemCode = 1;

	fn get_attribute_codes() -> Vec<AttributeCode> {
		vec![111, 222, 333]
	}

	fn get_encoded_attributes(&self) -> Vec<(AttributeCode, Vec<u8>)> {
		vec![
			(111, self.field_1.encode()),
			(222, self.field_2.encode()),
			(333, self.field_3.encode()),
		]
	}
}

type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, MockBlockNumber, MockCollectionId>;

fn create_collection(organizer: MockAccountId) -> MockCollectionId {
	<Test as Config>::NftHelper::create_collection(
		&organizer,
		&NftTransfer::account_id(),
		&CollectionConfig::default(),
	)
	.expect("Should have create contract collection")
}

mod store_as_nft {
	use frame_support::assert_err;

	use super::*;

	#[test]
	fn can_store_item_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockItem::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(
				BOB,
				collection_id,
				item_id,
				item.clone(),
				item_config
			));

			assert_eq!(Nft::collection_owner(collection_id), Some(ALICE));
			assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
			assert_eq!(
				Nft::typed_attribute::<ItemCode, MockItem>(
					&collection_id,
					&item_id,
					&AttributeNamespace::Pallet,
					&MockItem::ITEM_CODE,
				),
				Some(item.clone())
			);
			for (attribute_code, encoded_attributes) in item.get_encoded_attributes() {
				assert_eq!(
					Nft::typed_attribute(
						&collection_id,
						&item_id,
						&AttributeNamespace::Pallet,
						&attribute_code,
					),
					Some(encoded_attributes)
				);
			}

			assert_eq!(NftTransfer::nft_statuses(collection_id, item_id), Some(NftStatus::Stored));

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(crate::Event::ItemStored {
				collection_id,
				item_id,
				owner: BOB,
			}));
		});
	}

	#[test]
	fn cannot_store_duplicates_under_same_collection() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockItem::default();
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
			let item = MockItem {
				field_1: vec![1; ValueLimit::get() as usize],
				field_2: 1,
				field_3: false,
			};
			let item_config = pallet_nfts::ItemConfig::default();

			assert!(item.encode().len() > ValueLimit::get() as usize);
			assert_err!(
				// TODO: find why this is not atomic
				NftTransfer::store_as_nft(BOB, collection_id, item_id, item, item_config),
				pallet_nfts::Error::<Test>::IncorrectData
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
			let item = MockItem::default();
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

			assert!(NftStatuses::<Test>::get(collection_id, item_id).is_none());
			assert!(Nft::typed_attribute::<ItemCode, MockItem>(
				&collection_id,
				&item_id,
				&AttributeNamespace::Pallet,
				&MockItem::ITEM_CODE,
			)
			.is_none());
			for attribute_code in MockItem::get_attribute_codes() {
				assert!(Nft::attribute(
					&collection_id,
					&item_id,
					&AttributeNamespace::Pallet,
					&attribute_code.encode(),
				)
				.is_none());
			}

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(
				crate::Event::ItemRestored { collection_id, item_id, owner: BOB },
			));
		});
	}

	#[test]
	fn cannot_restore_uploaded_item() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockItem::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(BOB, collection_id, item_id, item, item_config));

			NftStatuses::<Test>::insert(collection_id, item_id, NftStatus::Uploaded);

			let result: Result<MockItem, _> =
				NftTransfer::recover_from_nft(BOB, collection_id, item_id);

			assert_noop!(result, Error::<Test>::NftOutsideOfChain);
		});
	}

	#[test]
	fn cannot_restore_nft_if_not_owned() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_collection(ALICE);
			let item_id = H256::random();
			let item = MockItem::default();
			let item_config = pallet_nfts::ItemConfig::default();

			assert_ok!(NftTransfer::store_as_nft(BOB, collection_id, item_id, item, item_config));

			let result: Result<MockItem, _> =
				NftTransfer::recover_from_nft(ALICE, collection_id, item_id);

			assert_noop!(result, pallet_nfts::Error::<Test>::NoPermission);
		});
	}
}
