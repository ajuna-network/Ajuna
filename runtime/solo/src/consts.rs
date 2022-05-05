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

use ajuna_primitives::{Balance, BlockNumber, Moment};

pub mod currency {
	use super::*;
	pub const PICO_AJUNS: Balance = 1;
	pub const NANO_AJUNS: Balance = 1_000 * PICO_AJUNS;
	pub const MICRO_AJUNS: Balance = 1_000 * NANO_AJUNS;
	pub const MILLI_AJUNS: Balance = 1_000 * MICRO_AJUNS;
	pub const AJUNS: Balance = 1_000 * MILLI_AJUNS;
}

pub mod time {
	use super::*;

	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const BLOCK_TIME_MS: Moment = 6_000;

	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const SLOT_DURATION: Moment = BLOCK_TIME_MS;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (BLOCK_TIME_MS as BlockNumber);
	pub const HOURS: BlockNumber = 60 * MINUTES;
	pub const DAYS: BlockNumber = 24 * HOURS;
}

pub mod ajuna {
	pub const MAX_ACKNOWLEDGE_BATCH: u32 = 10;
}
