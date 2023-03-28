use crate::types::{Avatar, AvatarCodec, Force, RarityTier};
use codec::{Decode, Encode};
use pallet_ajuna_nft_transfer::traits::{AttributeCode, NftConvertible};
use sp_std::prelude::*;

pub const DNA: AttributeCode = 10;
pub const SOUL_POINTS: AttributeCode = 11;
pub const RARITY: AttributeCode = 12;
pub const FORCE: AttributeCode = 13;

impl NftConvertible for Avatar {
	const ASSET_CODE: u16 = 0;

	fn encode_into(self) -> Vec<u8> {
		let avatar_codec = AvatarCodec::from(self);
		avatar_codec.encode()
	}

	fn decode_from(input: Vec<u8>) -> Result<Self, codec::Error> {
		let avatar_codec = AvatarCodec::decode(&mut input.as_slice())?;
		Ok(Avatar::from(avatar_codec))
	}

	fn get_attribute_table() -> Vec<AttributeCode> {
		vec![DNA, SOUL_POINTS, RARITY, FORCE]
	}

	fn get_encoded_attributes(&self) -> Vec<(AttributeCode, Vec<u8>)> {
		vec![
			(DNA, self.dna.clone().encode()),
			(SOUL_POINTS, self.souls.encode()),
			(RARITY, RarityTier::try_from(self.min_tier()).unwrap_or_default().encode()),
			(FORCE, Force::try_from(self.last_variation()).unwrap_or_default().encode()),
		]
	}
}
