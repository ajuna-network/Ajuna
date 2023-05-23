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
	Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum RarityTier {
	#[default]
	None,
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
			RarityTier::None => write!(f, ""),
			RarityTier::Common => write!(f, "Common"),
			RarityTier::Uncommon => write!(f, "Uncommon"),
			RarityTier::Rare => write!(f, "Rare"),
			RarityTier::Epic => write!(f, "Epic"),
			RarityTier::Legendary => write!(f, "Legendary"),
			RarityTier::Mythical => write!(f, "Mythical"),
		}
	}
}

impl From<u8> for RarityTier {
	fn from(value: u8) -> Self {
		match value {
			x if x == 0 => Self::None,
			x if x == 1 => Self::Common,
			x if x == 2 => Self::Uncommon,
			x if x == 3 => Self::Rare,
			x if x == 4 => Self::Epic,
			x if x == 5 => Self::Legendary,
			x if x == 6 => Self::Mythical,
			_ => Self::default(),
		}
	}
}

impl From<RarityTier> for BoundedVec<u8, ConstU32<20>> {
	fn from(x: RarityTier) -> Self {
		x.to_string().as_bytes().to_owned().try_into().unwrap_or_default()
	}
}
