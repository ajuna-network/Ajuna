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

use crate::types::Avatar;
use frame_support::{traits::Get, BoundedVec};
use pallet_ajuna_nft_transfer::traits::{NFTAttribute, NftConvertible};
use sp_std::prelude::*;

impl<KL, VL> NftConvertible<KL, VL> for Avatar
where
	KL: Get<u32>,
	VL: Get<u32>,
{
	const ITEM_CODE: &'static [u8] = b"AVATAR";
	const IPFS_URL_CODE: &'static [u8] = b"AVATAR_IPFS";

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
				BoundedVec::try_from(self.dna.clone().to_vec()).unwrap(),
			),
			(
				BoundedVec::try_from(b"SOUL_POINTS".to_vec()).unwrap(),
				BoundedVec::try_from(self.souls.to_le_bytes().to_vec()).unwrap(),
			),
			(
				BoundedVec::try_from(b"RARITY".to_vec()).unwrap(),
				BoundedVec::try_from(self.rarity().to_le_bytes().to_vec()).unwrap(),
			),
			(
				BoundedVec::try_from(b"FORCE".to_vec()).unwrap(),
				BoundedVec::try_from(self.force().to_le_bytes().to_vec()).unwrap(),
			),
			(
				BoundedVec::try_from(b"SEASON_ID".to_vec()).unwrap(),
				BoundedVec::try_from(self.season_id.to_le_bytes().to_vec()).unwrap(),
			),
		]
	}
}
