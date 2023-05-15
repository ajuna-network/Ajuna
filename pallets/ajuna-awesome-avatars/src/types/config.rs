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

use crate::types::AvatarVersion;
use frame_support::pallet_prelude::*;

/// Number of avatars to be minted.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum MintPackSize {
	#[default]
	One = 1,
	Three = 3,
	Six = 6,
}

impl MintPackSize {
	pub(crate) fn is_batched(&self) -> bool {
		self != &Self::One
	}
}

/// Minting fee per pack of avatars.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
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

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum MintPayment {
	/// Mint using free mint credits.
	#[default]
	Free,
	/// Normal minting consuming currency.
	Normal,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum PackType {
	#[default]
	Material = 1,
	Equipment = 2,
	Special = 3,
}

/// Minting options
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct MintOption {
	/// The choice of payment for minting.
	pub payment: MintPayment,
	/// Type of pack to mint
	pub mint_pack: PackType,
	/// The version of avatar to mint.
	pub version: AvatarVersion,
	/// Number of avatars to mint.
	pub count: MintPackSize,
}

pub type MintCount = u16;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct MintConfig<Balance, BlockNumber> {
	pub open: bool,
	pub fees: MintFees<Balance>,
	pub cooldown: BlockNumber,
	pub free_mint_fee_multiplier: MintCount,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct ForgeConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TransferConfig<Balance> {
	pub open: bool,
	pub free_mint_transfer_fee: MintCount,
	pub min_free_mint_transfer: MintCount,
	pub avatar_transfer_fee: Balance,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TradeConfig<Balance> {
	pub open: bool,
	pub min_fee: Balance,
	pub percent_fee: u8,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct AccountConfig<Balance> {
	pub storage_upgrade_fee: Balance,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct NftTransferConfig<Balance> {
	pub open: bool,
	pub prepare_fee: Balance,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct GlobalConfig<Balance, BlockNumber> {
	pub mint: MintConfig<Balance, BlockNumber>,
	pub forge: ForgeConfig,
	pub transfer: TransferConfig<Balance>,
	pub trade: TradeConfig<Balance>,
	pub account: AccountConfig<Balance>,
	pub nft_transfer: NftTransferConfig<Balance>,
}
