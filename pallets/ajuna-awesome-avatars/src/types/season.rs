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
}

impl<BlockNumber: PartialOrd> Season<BlockNumber> {
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
