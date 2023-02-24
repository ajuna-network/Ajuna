use crate::types::{Avatar, AvatarCodec, Force, RarityTier};
use codec::{Decode, Encode};
use pallet_ajuna_nft_transfer::traits::{AttributeCode, NftConvertible};

pub const DNA_ATTRIBUTE_CODE: u16 = 10;
pub const SOUL_POINTS_ATTRIBUTE_CODE: u16 = 11;
pub const RARITY_ATTRIBUTE_CODE: u16 = 12;
pub const FORCE_ATTRIBUTE_CODE: u16 = 13;

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
		vec![
			DNA_ATTRIBUTE_CODE,
			SOUL_POINTS_ATTRIBUTE_CODE,
			RARITY_ATTRIBUTE_CODE,
			FORCE_ATTRIBUTE_CODE,
		]
	}

	fn get_encoded_attributes(&self) -> Vec<(AttributeCode, Vec<u8>)> {
		vec![
			(DNA_ATTRIBUTE_CODE, self.dna.clone().encode()),
			(SOUL_POINTS_ATTRIBUTE_CODE, self.souls.encode()),
			(
				RARITY_ATTRIBUTE_CODE,
				RarityTier::try_from(self.min_tier()).unwrap_or_default().encode(),
			),
			(
				FORCE_ATTRIBUTE_CODE,
				Force::try_from(self.last_variation()).unwrap_or_default().encode(),
			),
		]
	}
}
