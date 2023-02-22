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
	fn get_asset_code() -> AssetCode {
		1
	}
}

fn create_random_mock_nft_collection(account: MockAccountId) -> MockCollectionId {
	let collection_config = CollectionConfig::default();
	<Test as crate::pallet::Config>::NftHelper::create_collection(
		&account,
		&account,
		&collection_config,
	)
	.expect("Should have create contract collection")
}

mod organizer {
	use super::*;

	#[test]
	fn set_organizer_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(NftTransfer::organizer(), None);
			assert_ok!(NftTransfer::set_organizer(RuntimeOrigin::root(), ALICE));
			assert_eq!(NftTransfer::organizer(), Some(ALICE), "Organizer should be Alice");
			System::assert_last_event(mock::RuntimeEvent::NftTransfer(
				crate::Event::OrganizerSet { organizer: ALICE },
			));
		});
	}

	#[test]
	fn set_organizer_should_reject_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				NftTransfer::set_organizer(RuntimeOrigin::signed(BOB), ALICE),
				sp_runtime::DispatchError::BadOrigin
			);
		});
	}
}

mod set_lock_state {
	use super::*;

	#[test]
	fn set_lock_state_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(NftTransfer::set_organizer(RuntimeOrigin::root(), ALICE));

			assert_ok!(NftTransfer::set_locked_state(
				RuntimeOrigin::signed(ALICE),
				PalletLockedState::Locked
			));
			assert_eq!(
				NftTransfer::lock_status(),
				PalletLockedState::Locked,
				"Pallet should be locked"
			);
			System::assert_last_event(mock::RuntimeEvent::NftTransfer(
				crate::Event::LockedStateSet { locked_state: PalletLockedState::Locked },
			));

			let collection_id = create_random_mock_nft_collection(ALICE);
			let asset = MockStruct { data: vec![1; MAX_ENCODING_SIZE as usize] };

			assert_noop!(
				NftTransfer::store_as_nft(BOB, collection_id, asset, None),
				Error::<Test>::PalletLocked
			);
		});
	}

	#[test]
	fn set_lock_state_should_fail_with_non_organizer_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(NftTransfer::set_organizer(RuntimeOrigin::root(), ALICE));

			assert_noop!(
				NftTransfer::set_locked_state(
					RuntimeOrigin::signed(BOB),
					PalletLockedState::Locked
				),
				sp_runtime::DispatchError::BadOrigin
			);
		});
	}
}

mod store_as_nft {
	use super::*;

	#[test]
	fn asset_properly_stored() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_random_mock_nft_collection(ALICE);
			let asset = MockStruct::default();

			let result = NftTransfer::store_as_nft(BOB, collection_id, asset.clone(), None);

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

			let stored_asset = <Test as crate::pallet::Config>::NftHelper::typed_attribute::<
				AssetCode,
				EncodedAssetOf<Test>,
			>(
				&collection_id,
				&asset_id,
				&AttributeNamespace::<MockAccountId>::Pallet,
				&MockStruct::get_asset_code(),
			)
			.map(|item| item.into_inner());

			assert_eq!(stored_asset, Some(asset.encode_into()))
		});
	}

	#[test]
	fn cannot_store_asset_above_encoding_limit() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_random_mock_nft_collection(ALICE);
			let asset = MockStruct { data: vec![1; MAX_ENCODING_SIZE as usize] };

			assert_noop!(
				NftTransfer::store_as_nft(BOB, collection_id, asset, None),
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
			let collection_id = create_random_mock_nft_collection(ALICE);
			let asset = MockStruct::default();

			let asset_id = NftTransfer::store_as_nft(BOB, collection_id, asset.clone(), None)
				.expect("Storage should have been successful!");

			let result = NftTransfer::recover_from_nft(BOB, collection_id, asset_id);

			assert_eq!(result, Ok(asset));

			System::assert_last_event(mock::RuntimeEvent::NftTransfer(
				crate::Event::AssetRestored { collection_id, asset_id, owner: BOB },
			));

			assert_eq!(LockItemStatus::<Test>::get(collection_id, asset_id), None);

			let stored_asset = <Test as crate::pallet::Config>::NftHelper::typed_attribute::<
				AssetCode,
				EncodedAssetOf<Test>,
			>(
				&collection_id,
				&asset_id,
				&AttributeNamespace::<MockAccountId>::Pallet,
				&MockStruct::get_asset_code(),
			);

			assert_eq!(stored_asset, None);
		});
	}

	#[test]
	fn cannot_restore_uploaded_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = create_random_mock_nft_collection(ALICE);
			let asset = MockStruct::default();

			let asset_id = NftTransfer::store_as_nft(BOB, collection_id, asset, None)
				.expect("Storage should have been successful!");

			LockItemStatus::<Test>::insert(collection_id, asset_id, NftStatus::Uploaded);

			let result: Result<MockStruct, _> =
				NftTransfer::recover_from_nft(BOB, collection_id, asset_id);

			assert_noop!(result, Error::<Test>::NftOutsideOfChain);
		});
	}
}
