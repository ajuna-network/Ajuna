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

impl ByteConvertible for RarityTier {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0 => Self::None,
			1 => Self::Common,
			2 => Self::Uncommon,
			3 => Self::Rare,
			4 => Self::Epic,
			5 => Self::Legendary,
			6 => Self::Mythical,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl From<RarityTier> for BoundedVec<u8, ConstU32<20>> {
	fn from(x: RarityTier) -> Self {
		x.to_string().as_bytes().to_owned().try_into().unwrap_or_default()
	}
}

impl RarityTier {
	pub(crate) fn upgrade(&self) -> Self {
		match self {
			Self::None => Self::None,
			Self::Common => Self::Uncommon,
			Self::Uncommon => Self::Rare,
			Self::Rare => Self::Epic,
			Self::Epic => Self::Legendary,
			Self::Legendary => Self::Mythical,
			Self::Mythical => Self::Mythical,
		}
	}
}
