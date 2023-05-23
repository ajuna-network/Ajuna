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
use codec::Encode;
use pallet_ajuna_nft_transfer::traits::{AttributeCode, NftConvertible};
use sp_std::prelude::*;

pub const DNA: AttributeCode = 10;
pub const SOUL_POINTS: AttributeCode = 11;
pub const RARITY: AttributeCode = 12;
pub const FORCE: AttributeCode = 13;

impl NftConvertible for Avatar {
	const ITEM_CODE: AttributeCode = 0;
	const IPFS_URL_CODE: AttributeCode = 1;

	fn get_attribute_codes() -> Vec<AttributeCode> {
		vec![DNA, SOUL_POINTS, RARITY, FORCE]
	}

	fn get_encoded_attributes(&self) -> Vec<(AttributeCode, Vec<u8>)> {
		vec![
			(DNA, self.dna.clone().encode()),
			(SOUL_POINTS, self.souls.encode()),
			(RARITY, RarityTier::try_from(self.min_tier()).unwrap_or_default().encode()),
			(FORCE, Force::from_byte(self.force()).encode()),
		]
	}
}
