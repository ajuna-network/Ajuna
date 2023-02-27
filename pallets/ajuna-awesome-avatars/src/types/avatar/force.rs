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
	Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Force {
	Kinetic = 0,
	Dream = 1,
	Solar = 2,
	Thermal = 3,
	Astral = 4,
	Empathy = 5,
}

impl Default for Force {
	fn default() -> Self {
		Force::Kinetic
	}
}

impl fmt::Display for Force {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Force::Kinetic => write!(f, "Kinetic"),
			Force::Dream => write!(f, "Dream"),
			Force::Solar => write!(f, "Solar"),
			Force::Thermal => write!(f, "Thermal"),
			Force::Astral => write!(f, "Astral"),
			Force::Empathy => write!(f, "Empathy"),
		}
	}
}

impl TryFrom<u8> for Force {
	type Error = ();

	fn try_from(x: u8) -> Result<Self, Self::Error> {
		match x {
			x if x == 0 => Ok(Force::Kinetic),
			x if x == 1 => Ok(Force::Dream),
			x if x == 2 => Ok(Force::Solar),
			x if x == 3 => Ok(Force::Thermal),
			x if x == 4 => Ok(Force::Astral),
			x if x == 5 => Ok(Force::Empathy),
			_ => Err(()),
		}
	}
}

impl From<Force> for BoundedVec<u8, ConstU32<20>> {
	fn from(x: Force) -> Self {
		x.to_string().as_bytes().to_owned().try_into().unwrap_or_default()
	}
}
