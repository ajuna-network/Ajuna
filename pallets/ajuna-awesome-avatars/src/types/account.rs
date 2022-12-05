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

use super::MintCount;
use frame_support::pallet_prelude::*;
use sp_runtime::traits::Get;

const MAX_AVATARS_PER_PLAYER: isize = 100;

pub struct MaxAvatarsPerPlayer;
impl Get<u32> for MaxAvatarsPerPlayer {
	fn get() -> u32 {
		MAX_AVATARS_PER_PLAYER as u32
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub enum StorageTier {
	One = MAX_AVATARS_PER_PLAYER.saturating_div(4).saturating_mul(1),
	Two = MAX_AVATARS_PER_PLAYER.saturating_div(4).saturating_mul(2),
	Three = MAX_AVATARS_PER_PLAYER.saturating_div(4).saturating_mul(3),
	Four = MAX_AVATARS_PER_PLAYER.saturating_div(4).saturating_mul(4),
}

impl Default for StorageTier {
	fn default() -> Self {
		Self::One
	}
}

impl StorageTier {
	pub(crate) fn upgrade(self) -> Self {
		match self {
			Self::One => Self::Two,
			Self::Two => Self::Three,
			Self::Three => Self::Four,
			Self::Four => Self::Four,
		}
	}
}

pub type Stat = u32;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Default)]
pub struct PlayStats<BlockNumber> {
	pub total: Stat,
	pub current_season: Stat,
	pub first: BlockNumber,
	pub last: BlockNumber,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Default)]
pub struct TradeStats {
	pub bought: Stat,
	pub sold: Stat,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Default)]
pub struct Stats<BlockNumber> {
	pub mint: PlayStats<BlockNumber>,
	pub forge: PlayStats<BlockNumber>,
	pub trade: TradeStats,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Default)]
pub struct AccountInfo<BlockNumber> {
	pub free_mints: MintCount,
	pub storage_tier: StorageTier,
	pub stats: Stats<BlockNumber>,
}
