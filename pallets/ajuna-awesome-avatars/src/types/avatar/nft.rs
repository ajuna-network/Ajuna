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

use crate::types::{Avatar, ByteConvertible, Force, RarityTier};
use frame_support::{traits::Get, BoundedVec};
use pallet_ajuna_nft_transfer::traits::{NFTAttribute, NftConvertible};
use parity_scale_codec::alloc::string::ToString;
use scale_info::prelude::format;
use sp_std::prelude::*;

impl<KL, VL> NftConvertible<KL, VL> for Avatar
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	const ITEM_CODE: &'static [u8] = b"AVATAR";
	const IPFS_URL_CODE: &'static [u8] = b"IPFS_URL";

	fn get_attribute_codes() -> Vec<NFTAttribute<KL>> {
		vec![
			BoundedVec::try_from(b"DNA".to_vec()).unwrap(),
			BoundedVec::try_from(b"SOUL_POINTS".to_vec()).unwrap(),
			BoundedVec::try_from(b"RARITY".to_vec()).unwrap(),
			BoundedVec::try_from(b"FORCE".to_vec()).unwrap(),
			BoundedVec::try_from(b"SEASON_ID".to_vec()).unwrap(),
		]
	}

	fn get_encoded_attributes(&self) -> Vec<(NFTAttribute<KL>, NFTAttribute<VL>)> {
		vec![
			(
				BoundedVec::try_from(b"DNA".to_vec()).unwrap(),
				BoundedVec::try_from(
					format!("0x{}", hex::encode(self.dna.as_slice())).into_bytes(),
				)
				.unwrap(),
			),
			(
				BoundedVec::try_from(b"SOUL_POINTS".to_vec()).unwrap(),
				BoundedVec::try_from(format!("{}", self.souls).into_bytes()).unwrap(),
			),
			(BoundedVec::try_from(b"RARITY".to_vec()).unwrap(), {
				let rarity_value = RarityTier::from_byte(if self.season_id == 1 {
					self.rarity() + 1
				} else {
					self.rarity()
				});
				BoundedVec::try_from(rarity_value.to_string().to_uppercase().into_bytes()).unwrap()
			}),
			(
				BoundedVec::try_from(b"FORCE".to_vec()).unwrap(),
				BoundedVec::try_from(
					Force::from_byte(self.force()).to_string().to_uppercase().into_bytes(),
				)
				.unwrap(),
			),
			(
				BoundedVec::try_from(b"SEASON_ID".to_vec()).unwrap(),
				BoundedVec::try_from(format!("{}", self.season_id).into_bytes()).unwrap(),
			),
		]
	}
}
