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
use sp_std::{fmt, ops::Range, prelude::*};

#[derive(Encode, Clone, Debug, Default, PartialEq)]
pub enum Force {
	Null,
	#[default]
	Kinetic,
	Dream,
	Solar,
	Thermal,
	Astral,
	Empathy,
}

impl fmt::Display for Force {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Force::Null => write!(f, ""),
			Force::Kinetic => write!(f, "Kinetic"),
			Force::Dream => write!(f, "Dream"),
			Force::Solar => write!(f, "Solar"),
			Force::Thermal => write!(f, "Thermal"),
			Force::Astral => write!(f, "Astral"),
			Force::Empathy => write!(f, "Empathy"),
		}
	}
}

impl ByteConvertible for Force {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Kinetic,
			2 => Self::Dream,
			3 => Self::Solar,
			4 => Self::Thermal,
			5 => Self::Astral,
			6 => Self::Empathy,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl Ranged for Force {
	fn range() -> Range<usize> {
		1..7
	}
}

impl From<Force> for BoundedVec<u8, ConstU32<20>> {
	fn from(x: Force) -> Self {
		x.to_string().as_bytes().to_owned().try_into().unwrap_or_default()
	}
}
