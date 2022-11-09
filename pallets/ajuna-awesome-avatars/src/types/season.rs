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
use frame_support::pallet_prelude::*;
use sp_runtime::traits::{AtLeast32Bit, UniqueSaturatedInto, Zero};
use sp_std::vec::Vec;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub struct SeasonStatus {
	pub active: bool,
	pub early: bool,
	pub prematurely_ended: bool,
}

#[derive(
	Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum RarityTier {
	Common = 0,
	Uncommon = 1,
	Rare = 2,
	Epic = 3,
	Legendary = 4,
	Mythical = 5,
}

pub type RarityPercent = u16;

#[derive(Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, PartialEq)]
pub struct Season<BlockNumber> {
	pub name: BoundedVec<u8, ConstU32<100>>,
	pub description: BoundedVec<u8, ConstU32<1_000>>,
	pub early_start: BlockNumber,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub max_tier_forges: u32,
	pub max_variations: u8,
	pub max_components: u8,
	pub min_sacrifices: u8,
	pub max_sacrifices: u8,
	pub tiers: BoundedVec<RarityTier, ConstU32<6>>,
	pub p_single_mint: BoundedVec<RarityPercent, ConstU32<5>>,
	pub p_batch_mint: BoundedVec<RarityPercent, ConstU32<5>>,
	pub per_period: BlockNumber,
	pub periods: u16,
}

impl<BlockNumber: AtLeast32Bit + Copy> Season<BlockNumber> {
	pub(crate) fn is_active(&self, now: BlockNumber) -> bool {
		now >= self.start && now <= self.end
	}

	pub(crate) fn is_early(&self, now: BlockNumber) -> bool {
		now >= self.early_start && now < self.start
	}

	pub(crate) fn validate<T: Config>(&mut self) -> DispatchResult {
		self.sort();
		self.validate_block_numbers::<T>()?;
		self.validate_max_variations::<T>()?;
		self.validate_max_components::<T>()?;
		self.validate_tiers::<T>()?;
		self.validate_percentages::<T>()?;
		Ok(())
	}

	#[allow(dead_code)]
	fn current_period(&self, now: &BlockNumber) -> u16 {
		let cycles = now.checked_rem(&self.full_cycle()).unwrap_or_else(Zero::zero);
		let current_period = if cycles.is_zero() { Zero::zero() } else { cycles / self.per_period };
		current_period.unique_saturated_into()
	}

	#[allow(dead_code)]
	fn full_cycle(&self) -> BlockNumber {
		self.per_period.saturating_mul(self.periods.unique_saturated_into())
	}

	fn sort(&mut self) {
		// tiers are sorted in ascending order
		self.tiers.sort_by(|a, b| a.cmp(b));
		// probabilities are sorted in descending order
		self.p_single_mint.sort_by(|a, b| b.cmp(a));
		self.p_batch_mint.sort_by(|a, b| b.cmp(a));
	}

	fn validate_block_numbers<T: Config>(&self) -> DispatchResult {
		ensure!(self.early_start < self.start, Error::<T>::EarlyStartTooLate);
		ensure!(self.start < self.end, Error::<T>::SeasonStartTooLate);
		Ok(())
	}

	fn validate_max_variations<T: Config>(&self) -> DispatchResult {
		ensure!(self.max_variations > 1, Error::<T>::MaxVariationsTooLow);
		ensure!(self.max_variations <= 0b0000_1111, Error::<T>::MaxVariationsTooHigh);
		Ok(())
	}

	fn validate_max_components<T: Config>(&self) -> DispatchResult {
		ensure!(self.max_components > 1, Error::<T>::MaxComponentsTooLow);
		ensure!(
			self.max_components.checked_mul(2).ok_or(Error::<T>::MaxComponentsTooHigh)? as usize <=
				T::Hash::max_encoded_len(),
			Error::<T>::MaxComponentsTooHigh
		);
		Ok(())
	}

	fn validate_tiers<T: Config>(&self) -> DispatchResult {
		let l = self.tiers.len();
		let mut tiers = Vec::from(self.tiers.clone());
		tiers.dedup();
		ensure!(l == tiers.len(), Error::<T>::DuplicatedRarityTier);
		Ok(())
	}

	fn validate_percentages<T: Config>(&self) -> DispatchResult {
		let p_1 = self.p_single_mint.iter().sum::<RarityPercent>();
		let p_2 = self.p_batch_mint.iter().sum::<RarityPercent>();
		ensure!(p_1 == MAX_PERCENTAGE as u16, Error::<T>::IncorrectRarityPercentages);
		ensure!(p_2 == MAX_PERCENTAGE as u16, Error::<T>::IncorrectRarityPercentages);
		ensure!(self.p_single_mint.len() < self.tiers.len(), Error::<T>::TooManyRarityPercentages);
		ensure!(self.p_batch_mint.len() < self.tiers.len(), Error::<T>::TooManyRarityPercentages);
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn current_period_works() {
		let per_period = 3;
		let periods = 7;
		let season = Season::default().per_period(per_period).periods(periods);

		#[allow(clippy::erasing_op, clippy::identity_op)]
		for (range, expected_period) in [
			// first cycle
			((0 * per_period)..(0 * per_period + per_period), 0),
			((1 * per_period)..(1 * per_period + per_period), 1),
			((2 * per_period)..(2 * per_period + per_period), 2),
			((3 * per_period)..(3 * per_period + per_period), 3),
			((4 * per_period)..(4 * per_period + per_period), 4),
			((5 * per_period)..(5 * per_period + per_period), 5),
			((6 * per_period)..(6 * per_period + per_period), 6),
			// second cycle
			((7 * per_period)..(7 * per_period + per_period), 0),
			((8 * per_period)..(8 * per_period + per_period), 1),
			((9 * per_period)..(9 * per_period + per_period), 2),
			((10 * per_period)..(10 * per_period + per_period), 3),
			((11 * per_period)..(11 * per_period + per_period), 4),
			((12 * per_period)..(12 * per_period + per_period), 5),
			((13 * per_period)..(13 * per_period + per_period), 6),
			// third cycle
			((14 * per_period)..(14 * per_period + per_period), 0),
			((15 * per_period)..(15 * per_period + per_period), 1),
			((16 * per_period)..(16 * per_period + per_period), 2),
			((17 * per_period)..(17 * per_period + per_period), 3),
			((18 * per_period)..(18 * per_period + per_period), 4),
			((19 * per_period)..(19 * per_period + per_period), 5),
			((20 * per_period)..(20 * per_period + per_period), 6),
		] {
			for now in range {
				assert_eq!(season.current_period(&now), expected_period);
			}
		}
	}

	#[test]
	fn current_periods_defaults_to_zero_when_divided_by_zero() {
		for now in 0..10 {
			for (per_period, periods) in [(0, 0), (0, 1), (1, 0)] {
				let season = Season::default().per_period(per_period).periods(periods);
				assert_eq!(season.current_period(&now), 0);
			}
		}
	}
}
