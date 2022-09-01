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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

pub const DNA_VERSION: u8 = 0;
pub const MAX_COMPONENTS: u32 = 100;

#[derive(
	Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum RarityTier {
	Common = 1,
	Uncommon = 2,
	Rare = 3,
	Epic = 4,
	Legendary = 5,
	Mythical = 6,
}

pub type RarityPercent = u8;

pub type RarityTiers = BoundedVec<(RarityTier, RarityPercent), ConstU32<6>>;

pub type SeasonId = u16;

pub type Dna = BoundedVec<u8, ConstU32<MAX_COMPONENTS>>;

#[derive(Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, PartialEq)]
pub struct Season<BlockNumber> {
	pub early_start: BlockNumber,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub max_mints: u16,
	pub max_mythical_mints: u16,
	pub rarity_tiers: RarityTiers,
	pub max_variations: u8,
	pub max_components: u32,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, Eq, PartialEq)]
pub struct SeasonMetadata {
	pub name: BoundedVec<u8, ConstU32<100>>,
	pub description: BoundedVec<u8, ConstU32<1000>>,
}

#[derive(Encode, Decode, Clone, Default, TypeInfo, MaxEncodedLen)]
pub struct Avatar {
	pub season: SeasonId,
	pub dna: Dna,
}
