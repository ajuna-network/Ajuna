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

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
pub struct SeasonStatus {
	pub season_id: SeasonId,
	pub early: bool,
	pub active: bool,
	pub early_ended: bool,
	pub max_tier_avatars: u32,
}
impl SeasonStatus {
	pub(crate) fn is_in_season(&self) -> bool {
		self.early || self.active || self.early_ended
	}
}

pub type RarityPercent = u8;
pub type SacrificeCount = u8;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct Season<BlockNumber> {
	pub name: BoundedVec<u8, ConstU32<100>>,
	pub description: BoundedVec<u8, ConstU32<1_000>>,
	pub early_start: BlockNumber,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub max_tier_forges: u32,
	pub max_variations: u8,
	pub max_components: u8,
	pub min_sacrifices: SacrificeCount,
	pub max_sacrifices: SacrificeCount,
	pub tiers: BoundedVec<RarityTier, ConstU32<6>>,
	pub single_mint_probs: BoundedVec<RarityPercent, ConstU32<5>>,
	pub batch_mint_probs: BoundedVec<RarityPercent, ConstU32<5>>,
	pub base_prob: RarityPercent,
	pub per_period: BlockNumber,
	pub periods: u16,
}

impl<BlockNumber: AtLeast32Bit> Season<BlockNumber> {
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
		self.validate_periods::<T>()?;
		Ok(())
	}

	pub(crate) fn current_period(&self, now: &BlockNumber) -> u16 {
		let cycles = now.checked_rem(&self.full_cycle()).unwrap_or_else(Zero::zero);
		let current_period =
			if cycles.is_zero() { Zero::zero() } else { cycles / self.per_period.clone() };
		current_period.unique_saturated_into()
	}

	pub(crate) fn max_tier(&self) -> RarityTier {
		self.tiers.clone().into_iter().max().unwrap_or_default()
	}

	fn full_cycle(&self) -> BlockNumber {
		self.per_period.clone().saturating_mul(self.periods.unique_saturated_into())
	}

	fn sort(&mut self) {
		// tiers are sorted in ascending order
		self.tiers.sort_by(|a, b| a.cmp(b));
		// probabilities are sorted in descending order
		self.single_mint_probs.sort_by(|a, b| b.cmp(a));
		self.batch_mint_probs.sort_by(|a, b| b.cmp(a));
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
		let p_1 = self
			.single_mint_probs
			.iter()
			.copied()
			.try_fold(RarityPercent::default(), |acc, x| acc.checked_add(x))
			.ok_or(Error::<T>::SingleMintProbsOverflow)?;
		let p_2 = self
			.batch_mint_probs
			.iter()
			.copied()
			.try_fold(RarityPercent::default(), |acc, x| acc.checked_add(x))
			.ok_or(Error::<T>::BatchMintProbsOverflow)?;

		ensure!(p_1 == MAX_PERCENTAGE, Error::<T>::IncorrectRarityPercentages);
		ensure!(p_2 == MAX_PERCENTAGE, Error::<T>::IncorrectRarityPercentages);
		ensure!(
			self.single_mint_probs.len() < self.tiers.len(),
			Error::<T>::TooManyRarityPercentages
		);
		ensure!(
			self.batch_mint_probs.len() < self.tiers.len(),
			Error::<T>::TooManyRarityPercentages
		);
		ensure!(self.base_prob < MAX_PERCENTAGE, Error::<T>::BaseProbTooHigh);
		Ok(())
	}

	fn validate_periods<T: Config>(&self) -> DispatchResult {
		ensure!(
			self.periods.is_zero() || (self.periods % self.max_variations as u16).is_zero(),
			Error::<T>::PeriodsIndivisible
		);
		ensure!(
			// TODO: is there more meaningful maximum for full cycle?
			self.full_cycle() <= u16::MAX.unique_saturated_into(),
			Error::<T>::PeriodConfigOverflow
		);
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{mock::*, types::RarityTier::*};
	use frame_support::{assert_err, assert_ok};

	impl Default for Season<MockBlockNumber> {
		fn default() -> Self {
			Self {
				name: b"cool season".to_vec().try_into().unwrap(),
				description: b"this is a really cool season".to_vec().try_into().unwrap(),
				early_start: 2,
				start: 3,
				end: 4,
				max_tier_forges: 10,
				max_variations: 2,
				max_components: 2,
				min_sacrifices: 1,
				max_sacrifices: 4,
				tiers: vec![
					RarityTier::Common,
					RarityTier::Uncommon,
					RarityTier::Rare,
					RarityTier::Epic,
					RarityTier::Legendary,
					RarityTier::Mythical,
				]
				.try_into()
				.unwrap(),
				single_mint_probs: vec![50, 30, 15, 4, 1].try_into().unwrap(),
				batch_mint_probs: vec![50, 30, 15, 4, 1].try_into().unwrap(),
				base_prob: 0,
				per_period: 10,
				periods: 12,
			}
		}
	}

	impl Season<MockBlockNumber> {
		pub fn early_start(mut self, early_start: MockBlockNumber) -> Self {
			self.early_start = early_start;
			self
		}
		pub fn start(mut self, start: MockBlockNumber) -> Self {
			self.start = start;
			self
		}
		pub fn end(mut self, end: MockBlockNumber) -> Self {
			self.end = end;
			self
		}
		pub fn max_tier_forges(mut self, max_tier_forges: u32) -> Self {
			self.max_tier_forges = max_tier_forges;
			self
		}
		pub fn max_components(mut self, max_components: u8) -> Self {
			self.max_components = max_components;
			self
		}
		pub fn max_variations(mut self, max_variations: u8) -> Self {
			self.max_variations = max_variations;
			self
		}
		pub fn min_sacrifices(mut self, min_sacrifices: SacrificeCount) -> Self {
			self.min_sacrifices = min_sacrifices;
			self
		}
		pub fn max_sacrifices(mut self, max_sacrifices: SacrificeCount) -> Self {
			self.max_sacrifices = max_sacrifices;
			self
		}
		pub fn tiers(mut self, tiers: &[RarityTier]) -> Self {
			self.tiers = tiers.to_vec().try_into().unwrap();
			self
		}
		pub fn single_mint_probs(mut self, percentages: &[RarityPercent]) -> Self {
			self.single_mint_probs = percentages.to_vec().try_into().unwrap();
			self
		}
		pub fn batch_mint_probs(mut self, percentages: &[RarityPercent]) -> Self {
			self.batch_mint_probs = percentages.to_vec().try_into().unwrap();
			self
		}
		pub fn base_prob(mut self, base_prob: RarityPercent) -> Self {
			self.base_prob = base_prob;
			self
		}
		pub fn per_period(mut self, per_period: MockBlockNumber) -> Self {
			self.per_period = per_period;
			self
		}
		pub fn periods(mut self, periods: u16) -> Self {
			self.periods = periods;
			self
		}
	}

	impl SeasonStatus {
		fn early(mut self, early: bool) -> Self {
			self.early = early;
			self
		}
		fn active(mut self, active: bool) -> Self {
			self.active = active;
			self
		}
		fn early_ended(mut self, early_ended: bool) -> Self {
			self.early_ended = early_ended;
			self
		}
	}

	#[test]
	fn is_in_season_works() {
		assert!(!SeasonStatus {
			season_id: 123,
			early: false,
			active: false,
			early_ended: false,
			max_tier_avatars: 0
		}
		.is_in_season());

		for season_status in [
			SeasonStatus::default().early(true).active(false).early_ended(false),
			SeasonStatus::default().early(false).active(true).early_ended(false),
			SeasonStatus::default().early(false).active(false).early_ended(true),
			SeasonStatus::default().early(false).active(true).early_ended(true),
			SeasonStatus::default().early(true).active(false).early_ended(true),
			SeasonStatus::default().early(true).active(true).early_ended(false),
			SeasonStatus::default().early(true).active(true).early_ended(true),
		] {
			assert!(season_status.is_in_season());
		}
	}

	#[test]
	fn validate_works() {
		let mut season = Season::default()
			.tiers(&[Common, Rare, Legendary])
			.single_mint_probs(&[10, 90])
			.batch_mint_probs(&[20, 80])
			.max_variations(5)
			.per_period(10)
			.periods(15);

		for (mut season, error) in [
			// block_numbers
			(season.clone().early_start(10).start(0), Error::<Test>::EarlyStartTooLate),
			(season.clone().start(10).end(0), Error::<Test>::SeasonStartTooLate),
			// max_variations
			(season.clone().max_variations(0), Error::<Test>::MaxVariationsTooLow),
			(season.clone().max_variations(16), Error::<Test>::MaxVariationsTooHigh),
			// max_components
			(season.clone().max_components(0), Error::<Test>::MaxComponentsTooLow),
			(season.clone().max_components(17), Error::<Test>::MaxComponentsTooHigh),
			// tiers
			(season.clone().tiers(&[Common, Common]), Error::<Test>::DuplicatedRarityTier),
			// percentages
			(
				season.clone().single_mint_probs(&[1, 100]),
				Error::<Test>::IncorrectRarityPercentages,
			),
			(season.clone().batch_mint_probs(&[1, 100]), Error::<Test>::IncorrectRarityPercentages),
			(
				season.clone().single_mint_probs(&[1, 2, 97]),
				Error::<Test>::TooManyRarityPercentages,
			),
			(season.clone().batch_mint_probs(&[1, 2, 97]), Error::<Test>::TooManyRarityPercentages),
			(
				season.clone().single_mint_probs(&[u8::MAX, 1]),
				Error::<Test>::SingleMintProbsOverflow,
			),
			(season.clone().batch_mint_probs(&[u8::MAX, 1]), Error::<Test>::BatchMintProbsOverflow),
			(season.clone().base_prob(101), Error::<Test>::BaseProbTooHigh),
			// periods
			(season.clone().per_period(2).periods(u16::MAX), Error::<Test>::PeriodConfigOverflow),
			(season.clone().periods(123).max_variations(7), Error::<Test>::PeriodsIndivisible),
		] {
			assert_err!(season.validate::<Test>(), error);
		}
		assert_ok!(season.validate::<Test>());
	}

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

		let season = Season::default().per_period(20).periods(12);
		assert_eq!(season.current_period(&15_792), 9);
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
