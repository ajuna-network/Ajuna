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

use crate::*;
use sp_std::prelude::*;

#[derive(Encode, Decode, Debug, Default, PartialEq)]
pub struct AvatarCodec {
	pub season_id: SeasonId,
	pub dna: Dna,
	pub soul_points: SoulCount,
	pub rarity: BoundedVec<u8, ConstU32<20>>,
	pub force: BoundedVec<u8, ConstU32<20>>,
}

impl From<Avatar> for AvatarCodec {
	fn from(avatar: Avatar) -> Self {
		let rarity_tier = RarityTier::try_from(avatar.min_tier()).unwrap_or_default();
		let last_variation = Force::try_from(avatar.last_variation()).unwrap_or_default();

		Self {
			season_id: avatar.season_id,
			dna: avatar.dna.clone(),
			soul_points: avatar.souls,
			rarity: rarity_tier.into(),
			force: last_variation.into(),
		}
	}
}

impl From<AvatarCodec> for Avatar {
	fn from(avatar_codec: AvatarCodec) -> Self {
		Self {
			season_id: avatar_codec.season_id,
			dna: avatar_codec.dna,
			souls: avatar_codec.soul_points,
		}
	}
}
