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
use sp_std::{fmt, prelude::*};

#[derive(
	Encode,
	Decode,
	MaxEncodedLen,
	RuntimeDebug,
	TypeInfo,
	Clone,
	Default,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
)]
pub enum RarityTier {
	#[default]
	Common,
	Uncommon,
	Rare,
	Epic,
	Legendary,
	Mythical,
}

impl fmt::Display for RarityTier {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			RarityTier::Common => write!(f, "Common"),
			RarityTier::Uncommon => write!(f, "Uncommon"),
			RarityTier::Rare => write!(f, "Rare"),
			RarityTier::Epic => write!(f, "Epic"),
			RarityTier::Legendary => write!(f, "Legendary"),
			RarityTier::Mythical => write!(f, "Mythical"),
		}
	}
}

impl From<RarityTier> for u8 {
	fn from(x: RarityTier) -> Self {
		match x {
			RarityTier::Common => 0,
			RarityTier::Uncommon => 1,
			RarityTier::Rare => 2,
			RarityTier::Epic => 3,
			RarityTier::Legendary => 4,
			RarityTier::Mythical => 5,
		}
	}
}

impl TryFrom<u8> for RarityTier {
	type Error = ();

	fn try_from(x: u8) -> Result<Self, Self::Error> {
		match x {
			x if x == 0 => Ok(RarityTier::Common),
			x if x == 1 => Ok(RarityTier::Uncommon),
			x if x == 2 => Ok(RarityTier::Rare),
			x if x == 3 => Ok(RarityTier::Epic),
			x if x == 4 => Ok(RarityTier::Legendary),
			x if x == 5 => Ok(RarityTier::Mythical),
			_ => Err(()),
		}
	}
}

impl From<RarityTier> for BoundedVec<u8, ConstU32<20>> {
	fn from(x: RarityTier) -> Self {
		x.to_string().as_bytes().to_owned().try_into().unwrap_or_default()
	}
}
