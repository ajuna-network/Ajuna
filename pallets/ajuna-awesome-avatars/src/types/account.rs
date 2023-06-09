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

use super::{MintCount, SeasonId};
use frame_support::pallet_prelude::*;
use sp_runtime::{traits::Get, BoundedBTreeSet};

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
pub enum StorageTier {
	#[default]
	One = 25,
	Two = 50,
	Three = 75,
	Four = 100,
	Five = 150,
	Max = 200,
}

impl StorageTier {
	pub(crate) fn upgrade(self) -> Self {
		match self {
			Self::One => Self::Two,
			Self::Two => Self::Three,
			Self::Three => Self::Four,
			Self::Four => Self::Five,
			Self::Five => Self::Max,
			Self::Max => Self::Max,
		}
	}
}

pub struct MaxAvatarsPerPlayer;
impl Get<u32> for MaxAvatarsPerPlayer {
	fn get() -> u32 {
		StorageTier::Max as u32
	}
}

pub struct MaxSeasons;
impl Get<u32> for MaxSeasons {
	fn get() -> u32 {
		1_000
	}
}

pub type Stat = u32;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct PlayStats<BlockNumber> {
	pub first: BlockNumber,
	pub last: BlockNumber,
	pub seasons_participated: BoundedBTreeSet<SeasonId, MaxSeasons>,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct TradeStats {
	pub bought: Stat,
	pub sold: Stat,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct Stats<BlockNumber> {
	pub mint: PlayStats<BlockNumber>,
	pub forge: PlayStats<BlockNumber>,
	pub trade: TradeStats,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct PlayerConfig {
	pub free_mints: MintCount,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct PlayerSeasonConfig<BlockNumber> {
	pub storage_tier: StorageTier,
	pub stats: Stats<BlockNumber>,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct SeasonInfo {
	pub minted: Stat,
	pub forged: Stat,
}
