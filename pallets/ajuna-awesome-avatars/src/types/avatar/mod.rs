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

mod force;
mod nft;
mod rarity_tier;
mod tools;

pub use force::*;
pub use nft::*;
pub use rarity_tier::*;
pub(crate) use tools::*;

use frame_support::pallet_prelude::*;
use sp_std::prelude::*;

pub type IpfsUrl = BoundedVec<u8, MaxIpfsUrl>;
pub struct MaxIpfsUrl;
impl Get<u32> for MaxIpfsUrl {
	fn get() -> u32 {
		80
	}
}

pub type SeasonId = u16;
pub type Dna = BoundedVec<u8, ConstU32<100>>;
pub type SoulCount = u32;

/// Used to indicate which version of the forging and/or mint logic should be used.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AvatarVersion {
	#[default]
	V1,
	V2,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct Avatar {
	pub season_id: SeasonId,
	pub version: AvatarVersion,
	pub dna: Dna,
	pub souls: SoulCount,
}

impl Avatar {
	#[inline]
	pub(crate) fn min_tier(&self) -> u8 {
		self.version
			.with_mapper(|mapper: Box<dyn AttributeMapper>| mapper.min_tier(self))
	}

	#[inline]
	pub(crate) fn last_variation(&self) -> u8 {
		self.version
			.with_mapper(|mapper: Box<dyn AttributeMapper>| mapper.last_variation(self))
	}
}
