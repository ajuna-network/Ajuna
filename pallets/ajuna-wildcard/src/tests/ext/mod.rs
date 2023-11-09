mod challenge;
mod deposit;
mod end2end;
mod freeze;
mod propagate_freeze;
mod refund_frozen;
mod respond_challenge;
mod respond_zero_challenge;
mod withdraw;
mod withdraw_frozen;

use super::{mock, mock::*};
use crate::*;

use frame_support::{assert_noop, assert_ok};
use sp_core::{Pair, H256};

const MOCK_NON_NATIVE_FUNGIBLE_PAD: [u8; 30] = [11; 30];
const MOCK_NON_NATIVE_NON_FUNGIBLE_PAD: [u8; 30] = [62; 30];

fn generate_wide_id(id: u16, asset_type: AssetType, native: bool) -> WideId {
	let mut wide_id = Vec::with_capacity(32);

	let pad = match asset_type {
		AssetType::Fungible =>
			if native {
				NATIVE_FUNGIBLE_PAD
			} else {
				MOCK_NON_NATIVE_FUNGIBLE_PAD
			},
		AssetType::NonFungible =>
			if native {
				NATIVE_NON_FUNGIBLE_PAD
			} else {
				MOCK_NON_NATIVE_NON_FUNGIBLE_PAD
			},
	};

	wide_id.extend(pad);
	wide_id.extend(id.to_le_bytes());
	H256::from_slice(wide_id.as_slice())
}

pub(crate) fn generate_native_fungible_wide_id(asset_id: MockAssetId) -> WideId {
	generate_wide_id(asset_id as u16, AssetType::Fungible, true)
}

pub(crate) fn generate_foreign_fungible_wide_id(asset_id: MockAssetId) -> WideId {
	generate_wide_id(asset_id as u16, AssetType::Fungible, false)
}

pub(crate) fn generate_native_non_fungible_wide_id(
	collection_id: MockCollectionId,
	item_id: MockItemId,
) -> (WideId, WideId) {
	(
		generate_wide_id(collection_id as u16, AssetType::NonFungible, true),
		generate_wide_id(item_id as u16, AssetType::NonFungible, true),
	)
}

pub(crate) fn generate_foreign_non_fungible_wide_id(
	collection_id: MockCollectionId,
	item_id: MockItemId,
) -> (WideId, WideId) {
	(
		generate_wide_id(collection_id as u16, AssetType::NonFungible, false),
		generate_wide_id(item_id as u16, AssetType::NonFungible, false),
	)
}

pub(crate) fn generate_wide_id_for_amount(amt: MockBalance) -> WideId {
	let mut wide_id = Vec::with_capacity(32);
	wide_id.extend(amt.to_le_bytes());
	wide_id.extend([0; 16]);
	H256::from_slice(wide_id.as_slice())
}

pub(crate) fn generate_signature_for<T: Proof>(proof: &T) -> sp_core::sr25519::Signature {
	MockKeyPair::get().sign(proof.extract_msg().as_slice())
}

pub(crate) fn make_challenge(account: MockAccountId) -> (MockAccountId, EpochNumber, ChunkIndex) {
	let epoch_number = Pallet::<Test>::calculate_epoch_number_from(<Test as Config>::Time::now())
		.expect("Should get epoch");

	assert_ok!(Wildcard::challenge(RuntimeOrigin::signed(account)));

	(account, epoch_number - 1, Challenges::<Test>::get((epoch_number - 1, account)).unwrap())
}
