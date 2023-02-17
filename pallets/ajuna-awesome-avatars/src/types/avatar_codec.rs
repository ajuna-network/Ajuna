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
use codec::alloc::string::ToString;
use scale_info::prelude::string::String;

pub struct AvatarCodec {
	pub season_id: SeasonId,
	pub dna: Dna,
	pub soul_points: SoulCount,
	pub rarity: String,
	// force: String,
}

impl From<Avatar> for AvatarCodec {
	fn from(avatar: Avatar) -> Self {
		let rarity_tier: RarityTier = avatar.min_tier().try_into().unwrap_or_default();

		Self {
			season_id: avatar.season_id,
			dna: avatar.dna.clone(),
			soul_points: avatar.souls,
			rarity: rarity_tier.to_string(),
			// force: "".into(),
		}
	}
}
