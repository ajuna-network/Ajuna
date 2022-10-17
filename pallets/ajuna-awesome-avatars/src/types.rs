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
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::ArithmeticError;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

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
pub type MintCount = u16;

#[derive(Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, PartialEq)]
pub struct Season<BlockNumber> {
	pub name: BoundedVec<u8, ConstU32<100>>,
	pub description: BoundedVec<u8, ConstU32<1_000>>,
	pub early_start: BlockNumber,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub max_variations: u8,
	pub max_components: u8,
	pub tiers: BoundedVec<RarityTier, ConstU32<6>>,
	pub p_single_mint: BoundedVec<RarityPercent, ConstU32<5>>,
	pub p_batch_mint: BoundedVec<RarityPercent, ConstU32<5>>,
}

impl<BlockNumber: PartialOrd> Season<BlockNumber> {
	pub(crate) fn is_active(&self, now: BlockNumber) -> bool {
		now >= self.early_start && now <= self.end
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
			// TODO: 32 must come from T::Hashing::len()
			self.max_components.checked_mul(2).ok_or(ArithmeticError::Overflow)? <= 32,
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

pub type SeasonId = u16;
pub type Dna = BoundedVec<u8, ConstU32<100>>;
pub type SoulCount = u32; // TODO: is u32 enough?

#[derive(Encode, Decode, Clone, Default, TypeInfo, MaxEncodedLen)]
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
				self.souls = self.souls.saturating_add(other.souls);
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

/// Number of avatars to be minted.
#[derive(Copy, Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub enum MintPackSize {
	One = 1,
	Three = 3,
	Six = 6,
}

impl Default for MintPackSize {
	fn default() -> Self {
		MintPackSize::One
	}
}

#[derive(Copy, Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct MintFees<Balance> {
	pub one: Balance,
	pub three: Balance,
	pub six: Balance,
}

impl<Balance> MintFees<Balance> {
	pub fn fee_for(self, mint_count: MintPackSize) -> Balance {
		match mint_count {
			MintPackSize::One => self.one,
			MintPackSize::Three => self.three,
			MintPackSize::Six => self.six,
		}
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub enum MintType {
	Free,
	Normal,
}

impl Default for MintType {
	fn default() -> Self {
		MintType::Free
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, Eq, PartialEq)]
pub struct MintOption {
	pub mint_type: MintType,
	pub count: MintPackSize,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct MintConfig<Balance, BlockNumber> {
	pub open: bool,
	pub fees: MintFees<Balance>,
	pub cooldown: BlockNumber,
	pub free_mint_fee_multiplier: MintCount,
	pub free_mint_transfer_fee: MintCount,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct ForgeConfig {
	pub open: bool,
	pub min_sacrifices: u8,
	pub max_sacrifices: u8,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TradeConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct GlobalConfig<Balance, BlockNumber> {
	pub max_avatars_per_player: u32,
	pub mint: MintConfig<Balance, BlockNumber>,
	pub forge: ForgeConfig,
	pub trade: TradeConfig,
}
