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

use frame_support::pallet_prelude::*;

/// Number of avatars to be minted.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, Debug, Eq, PartialEq)]
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

impl MintPackSize {
	pub(crate) fn is_batched(&self) -> bool {
		self != &Self::One
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, Debug, Default, PartialEq)]
pub struct MintFees<Balance> {
	pub one: Balance,
	pub three: Balance,
	pub six: Balance,
}

impl<Balance> MintFees<Balance> {
	pub fn fee_for(self, mint_count: &MintPackSize) -> Balance {
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

pub type MintCount = u16;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct MintConfig<Balance, BlockNumber> {
	pub open: bool,
	pub fees: MintFees<Balance>,
	pub cooldown: BlockNumber,
	pub free_mint_fee_multiplier: MintCount,
	pub free_mint_transfer_fee: MintCount,
	pub min_free_mint_transfer: MintCount,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct ForgeConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TradeConfig<Balance> {
	pub open: bool,
	pub buy_fee: Balance,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct GlobalConfig<Balance, BlockNumber> {
	pub mint: MintConfig<Balance, BlockNumber>,
	pub forge: ForgeConfig,
	pub trade: TradeConfig<Balance>,
}
