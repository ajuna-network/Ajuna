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
use sp_runtime::traits::{Saturating, Zero};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

pub type SeasonId = u16;
pub type Dna = BoundedVec<u8, ConstU32<100>>;
pub type SoulCount = u32; // TODO: is u32 enough?

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Default)]
pub struct Avatar {
	pub season_id: SeasonId,
	pub dna: Dna,
	pub souls: SoulCount,
}

impl Avatar {
	pub(crate) fn compare_all<T: Config>(
		&mut self,
		others: &[Self],
		max_variations: u8,
		max_tier: u8,
	) -> Result<(BTreeSet<usize>, u8), DispatchError> {
		let upgradable_indexes = self.upgradable_indexes::<T>()?;
		Ok(others.iter().fold(
			(BTreeSet::<usize>::new(), 0),
			|(mut matched_components, mut matches), other| {
				let (is_match, matching_components) =
					self.compare(other, &upgradable_indexes, max_variations, max_tier);

				if is_match {
					matches += 1;
					matched_components.extend(matching_components.iter());
				}

				// TODO: is u32 enough?
				self.souls.saturating_accrue(other.souls);
				(matched_components, matches)
			},
		))
	}

	pub(crate) fn upgradable_indexes<T: Config>(&self) -> Result<Vec<usize>, DispatchError> {
		let min_tier = self.min_tier::<T>()?;
		Ok(self
			.dna
			.iter()
			.enumerate()
			.filter(|(_, x)| (*x >> 4) == min_tier)
			.map(|(i, _)| i)
			.collect::<Vec<usize>>())
	}

	pub(crate) fn min_tier<T: Config>(&self) -> Result<u8, DispatchError> {
		self.dna
			.iter()
			.map(|x| *x >> 4)
			.min()
			.ok_or_else(|| Error::<T>::IncorrectDna.into())
	}

	pub(crate) fn compare(
		&self,
		other: &Self,
		indexes: &[usize],
		max_variations: u8,
		max_tier: u8,
	) -> (bool, BTreeSet<usize>) {
		let compare_variation = |lhs: u8, rhs: u8| -> bool {
			let diff = if lhs > rhs { lhs - rhs } else { rhs - lhs };
			diff == 1 || diff == (max_variations - 1)
		};

		let (matching_indexes, matches, mirrors) =
			self.dna.clone().into_iter().zip(other.dna.clone()).enumerate().fold(
				(BTreeSet::new(), 0, 0),
				|(mut matching_indexes, mut matches, mut mirrors), (i, (lhs, rhs))| {
					let lhs_variation = lhs & 0b0000_1111;
					let rhs_variation = rhs & 0b0000_1111;
					if lhs_variation == rhs_variation {
						mirrors += 1;
					}

					if indexes.contains(&i) {
						let lhs_tier = lhs >> 4;
						let rhs_tier = rhs >> 4;
						let is_matching_tier = lhs_tier == rhs_tier;
						let is_maxed_tier = lhs_tier == max_tier;

						let is_similar_variation = compare_variation(lhs_variation, rhs_variation);

						if is_matching_tier && !is_maxed_tier && is_similar_variation {
							matching_indexes.insert(i);
							matches += 1;
						}
					}
					(matching_indexes, matches, mirrors)
				},
			);

		// 1 upgradable component requires 1 match + 4 mirrors
		// 2 upgradable component requires 2 match + 2 mirrors
		// 3 upgradable component requires 3 match + 0 mirrors
		let mirrors_required = (3_u8.saturating_sub(matches)) * 2;
		let is_match = matches >= 3 || (matches >= 1 && mirrors >= mirrors_required);
		(is_match, matching_indexes)
	}

	pub(crate) fn forge_probability<T: Config>(
		&self,
		season: &SeasonOf<T>,
		now: &T::BlockNumber,
		matches: u8,
	) -> u8 {
		let period_multiplier = self.forge_multiplier::<T>(season, now);
		((MAX_PERCENTAGE / season.max_sacrifices) * matches) / period_multiplier
	}

	fn forge_multiplier<T: Config>(&self, season: &SeasonOf<T>, now: &T::BlockNumber) -> u8 {
		let mut current_period = season.current_period(now);
		let mut last_variation = (self.dna.last().unwrap_or(&0) & 0b0000_1111) as u16;

		current_period.saturating_inc();
		last_variation.saturating_inc();

		let max_variations = season.max_variations as u16;
		let is_in_period = if last_variation == max_variations {
			(current_period % max_variations).is_zero()
		} else {
			(current_period % max_variations) == last_variation
		};

		// TODO: [AAATAR-352]
		if (current_period == last_variation) || is_in_period {
			1
		} else {
			2
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{mock::*, types::*};

	impl Avatar {
		fn dna(mut self, dna: &[u8]) -> Self {
			self.dna = Dna::try_from(dna.to_vec()).unwrap();
			self
		}
	}

	#[test]
	fn forge_probability_works() {
		// | variation |  period |
		// + --------- + ------- +
		// |         1 |   1,  7 |
		// |         2 |   2,  8 |
		// |         3 |   3,  9 |
		// |         4 |   4, 10 |
		// |         5 |   5, 11 |
		// |         6 |   6, 12 |
		let per_period = 2;
		let periods = 6;
		let max_variations = 6;
		let max_sacrifices = 4;

		let season = Season::default()
			.per_period(per_period)
			.periods(periods)
			.max_variations(max_variations)
			.max_sacrifices(max_sacrifices);

		let avatar = Avatar::default().dna(&[1, 3, 3, 7, 0]);

		// in period
		let now = 1;
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 1), 25);
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 2), 50);
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 3), 75);
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 4), 100);

		// not in period
		let now = 2;
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 1), 12);
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 2), 25);
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 3), 37);
		assert_eq!(avatar.forge_probability::<Test>(&season, &now, 4), 50);
	}

	#[test]
	fn forge_multiplier_works() {
		// | variation |      period |
		// + --------- + ----------- +
		// |         1 | 1, 4, 7, 10 |
		// |         2 | 2, 5, 8, 11 |
		// |         3 | 3, 6, 9, 12 |
		let per_period = 4;
		let periods = 3;
		let max_variations = 3;

		let season = Season::default()
			.per_period(per_period)
			.periods(periods)
			.max_variations(max_variations);

		#[allow(clippy::erasing_op, clippy::identity_op)]
		for (range, dna, expected_period, expected_multiplier) in [
			// cycle 0, period 0, last_variation must be 0
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 0], 0, 1),
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 1], 0, 2),
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 2], 0, 2),
			// cycle 0, period 1, last_variation must be 1
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 0], 1, 2),
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 1], 1, 1),
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 2], 1, 2),
			// cycle 0, period 2, last_variation must be 2
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 0], 2, 2),
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 1], 2, 2),
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 2], 2, 1),
			// cycle 1, period 0, last_variation must be 0
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 0], 0, 1),
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 1], 0, 2),
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 2], 0, 2),
			// cycle 1, period 1, last_variation must be 1
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 0], 1, 2),
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 1], 1, 1),
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 2], 1, 2),
			// cycle 1, period 2, last_variation must be 2
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 0], 2, 2),
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 1], 2, 2),
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 2], 2, 1),
		] {
			for now in range {
				assert_eq!(season.current_period(&now), expected_period);

				let avatar = Avatar::default().dna(&dna);
				assert_eq!(avatar.forge_multiplier::<Test>(&season, &now), expected_multiplier);
			}
		}
	}
}
