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
use sp_runtime::traits::Saturating;
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
}
