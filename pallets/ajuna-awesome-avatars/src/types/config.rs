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

pub type MintCount = u16;

/// Number of avatars to be minted.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum MintPackSize {
	#[default]
	One,
	Three,
	Six,
}

impl MintPackSize {
	pub(crate) fn is_batched(&self) -> bool {
		self != &Self::One
	}
	pub(crate) fn as_mint_count(&self) -> MintCount {
		match self {
			MintPackSize::One => 1,
			MintPackSize::Three => 3,
			MintPackSize::Six => 6,
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
	/// The choice of pack to mint.
	pub pack_type: PackType,
	/// The version of avatar to mint.
	pub version: AvatarVersion,
	/// The number of avatars to mint.
	pub pack_size: MintPackSize,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct MintConfig<BlockNumber> {
	pub open: bool,
	pub cooldown: BlockNumber,
	pub free_mint_fee_multiplier: MintCount,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct ForgeConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TransferConfig {
	pub open: bool,
	pub free_mint_transfer_fee: MintCount,
	pub min_free_mint_transfer: MintCount,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct TradeConfig {
	pub open: bool,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct NftTransferConfig<Balance> {
	pub open: bool,
	pub prepare_fee: Balance,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct GlobalConfig<Balance, BlockNumber> {
	pub mint: MintConfig<BlockNumber>,
	pub forge: ForgeConfig,
	pub transfer: TransferConfig,
	pub trade: TradeConfig,
	pub nft_transfer: NftTransferConfig<Balance>,
}
