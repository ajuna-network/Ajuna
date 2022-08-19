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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Default, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Season<BlockNumber> {
	pub early_access_start: BlockNumber,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub max_mints: u16,
	pub max_mythical_mints: u16,
}

impl<BlockNumber: PartialOrd> Season<BlockNumber> {
	pub fn new(
		early_access_start: BlockNumber,
		start: BlockNumber,
		end: BlockNumber,
		max_mints: u16,
		max_mythical_mints: u16,
	) -> Self {
		Self { early_access_start, start, end, max_mints, max_mythical_mints }
	}

	/// Checks that first season end is set before second season early access start.
	pub fn are_seasons_overlapped(
		first_season: &Season<BlockNumber>,
		second_season: &Season<BlockNumber>,
	) -> bool {
		first_season.end >= second_season.early_access_start
	}

	/// Checks if season early access start is set before start.
	pub fn is_early_access_start_too_late(&self) -> bool {
		self.early_access_start >= self.start
	}

	/// Checks if season start is set before end.
	pub fn is_season_start_too_late(&self) -> bool {
		self.start >= self.end
	}
}
