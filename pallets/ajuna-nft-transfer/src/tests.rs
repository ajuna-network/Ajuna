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
use frame_support::{
	assert_err, assert_noop, assert_ok,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{bounded_vec, testing::H256, BoundedVec};

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

impl NftConvertible<KeyLimit, ValueLimit> for MockItem {
	const ITEM_CODE: &'static [u8] = &[11];
	const IPFS_URL_CODE: &'static [u8] = &[21];

	fn get_attribute_codes() -> Vec<NFTAttribute<KeyLimit>> {
		vec![bounded_vec![111], bounded_vec![222], bounded_vec![240]]
	}

	fn get_encoded_attributes(&self) -> Vec<(NFTAttribute<KeyLimit>, NFTAttribute<ValueLimit>)> {
		vec![
			(bounded_vec![111], BoundedVec::try_from(self.field_1.clone()).unwrap()),
			(bounded_vec![222], BoundedVec::try_from(self.field_2.to_le_bytes().to_vec()).unwrap()),
			(bounded_vec![240], BoundedVec::try_from(vec![self.field_3 as u8]).unwrap()),
		]
	}
}

type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, BlockNumberFor<Test>, MockCollectionId>;

fn create_collection(organizer: MockAccountId) -> MockCollectionId {
	<Test as Config>::NftHelper::create_collection(
		&organizer,
		&NftTransfer::account_id(),
		&CollectionConfig::default(),
	)
	.expect("Should have create contract collection")
}

mod store_as_nft {
	use super::*;
	use sp_runtime::traits::Get;

	#[test]
	fn can_store_item_successfully() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item.clone(),
					url.clone()
				));
				assert_eq!(Nft::collection_owner(collection_id), Some(ALICE));
				assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::ITEM_CODE),
					Some(item.encode())
				);
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::IPFS_URL_CODE),
					Some(url)
				);
				for (attribute_code, encoded_attributes) in item.get_encoded_attributes() {
					assert_eq!(
						Nft::system_attribute(&collection_id, Some(&item_id), &attribute_code),
						Some(encoded_attributes.to_vec())
					);
				}
				assert_eq!(
					NftStatuses::<Test>::get(collection_id, item_id),
					Some(NftStatus::Stored)
				);

				// check players pay for the item deposit
				assert_eq!(Balances::free_balance(BOB), 999);

				System::assert_last_event(mock::RuntimeEvent::NftTransfer(
					crate::Event::ItemStored { collection_id, item_id, owner: BOB },
				));
			});
	}

	#[test]
	fn can_store_item_and_transfer() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item.clone(),
					url.clone()
				));

				System::assert_last_event(mock::RuntimeEvent::NftTransfer(
					crate::Event::ItemStored { collection_id, item_id, owner: BOB },
				));

				assert_eq!(Nft::collection_owner(collection_id), Some(ALICE));
				assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::ITEM_CODE),
					Some(item.encode())
				);
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::IPFS_URL_CODE),
					Some(url)
				);

				assert_ok!(Nft::do_transfer(collection_id, item_id, ALICE, |_, _| { Ok(()) }));

				assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));

				System::assert_last_event(mock::RuntimeEvent::Nft(
					pallet_nfts::Event::Transferred {
						collection: collection_id,
						item: item_id,
						from: BOB,
						to: ALICE,
					},
				));
			});
	}

	#[test]
	fn cannot_store_empty_ipfs_url() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = vec![];

				assert_err!(
					NftTransfer::store_as_nft(ALICE, collection_id, item_id, item, url),
					Error::<Test>::EmptyIpfsUrl
				);
			});
	}

	#[test]
	fn cannot_store_duplicates_under_same_collection() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					ALICE,
					collection_id,
					item_id,
					item.clone(),
					url.clone()
				));
				assert_noop!(
					NftTransfer::store_as_nft(ALICE, collection_id, item_id, item, url),
					pallet_nfts::Error::<Test>::AlreadyExists
				);
			});
	}

	#[test]
	fn cannot_store_item_above_encoding_limit() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem {
					field_1: vec![1; ValueLimit::get() as usize],
					field_2: 1,
					field_3: false,
				};
				let url = b"ipfs://test".to_vec();

				assert!(item.encode().len() > ValueLimit::get() as usize);
				// NOTE: As long as the execution is wrapped in an extrinsic, this is a noop.
				assert_err!(
					NftTransfer::store_as_nft(BOB, collection_id, item_id, item, url),
					pallet_nfts::Error::<Test>::IncorrectData
				);
			});
	}
}

mod recover_from_nft {
	use super::*;

	#[test]
	fn can_recover_item_successfully() {
		let initial_balance = ItemDeposit::get() + 999;
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, initial_balance)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item.clone(),
					url
				));
				assert_eq!(Balances::free_balance(BOB), 999);

				assert_eq!(NftTransfer::recover_from_nft(BOB, collection_id, item_id), Ok(item));
				assert!(NftStatuses::<Test>::get(collection_id, item_id).is_none());
				assert!(Nft::system_attribute(
					&collection_id,
					Some(&item_id),
					&MockItem::ITEM_CODE.encode()
				)
				.is_none());
				assert!(Nft::system_attribute(
					&collection_id,
					Some(&item_id),
					&MockItem::IPFS_URL_CODE.encode()
				)
				.is_none());
				for attribute_code in MockItem::get_attribute_codes() {
					assert!(Nft::attribute(&collection_id, &item_id, &attribute_code.encode())
						.is_none());
				}

				// check players are refunded the item deposit
				assert_eq!(Balances::free_balance(BOB), initial_balance);

				System::assert_last_event(mock::RuntimeEvent::NftTransfer(
					crate::Event::ItemRestored { collection_id, item_id, owner: BOB },
				));
			});
	}

	#[test]
	fn cannot_restore_uploaded_item() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(BOB, collection_id, item_id, item, url));
				NftStatuses::<Test>::insert(collection_id, item_id, NftStatus::Uploaded);

				assert_noop!(
					NftTransfer::recover_from_nft(BOB, collection_id, item_id)
						as Result<MockItem, _>,
					Error::<Test>::NftOutsideOfChain
				);
			});
	}

	#[test]
	fn cannot_restore_if_not_owned() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(BOB, collection_id, item_id, item, url));

				// NOTE: As long as the execution is wrapped in an extrinsic, this is a noop.
				assert_err!(
					NftTransfer::recover_from_nft(ALICE, collection_id, item_id)
						as Result<MockItem, _>,
					pallet_nfts::Error::<Test>::NoPermission
				);
			});
	}
}
