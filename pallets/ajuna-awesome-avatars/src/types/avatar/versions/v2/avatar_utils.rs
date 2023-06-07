use super::{constants::*, types::*, ByteType};
use crate::{
	types::{Avatar, AvatarVersion, Dna, SeasonId, SoulCount},
	ByteConvertible, Config, Force, Ranged, RarityTier,
};
use core::cmp::Ordering;
use frame_support::traits::Len;
#[cfg(test)]
use sp_runtime::traits::Hash;
use sp_std::{marker::PhantomData, vec::Vec};

#[derive(Copy, Clone)]
pub enum AvatarAttributes {
	ItemType,
	ItemSubType,
	ClassType1,
	ClassType2,
	CustomType1,
	CustomType2,
	RarityTier,
	Quantity,
}

#[derive(Clone)]
pub enum AvatarSpecBytes {
	SpecByte1,
	SpecByte2,
	SpecByte3,
	SpecByte4,
	SpecByte5,
	SpecByte6,
	SpecByte7,
	SpecByte8,
	#[allow(dead_code)]
	SpecByte9,
	#[allow(dead_code)]
	SpecByte10,
	#[allow(dead_code)]
	SpecByte11,
	#[allow(dead_code)]
	SpecByte12,
	#[allow(dead_code)]
	SpecByte13,
	#[allow(dead_code)]
	SpecByte14,
	#[allow(dead_code)]
	SpecByte15,
	#[allow(dead_code)]
	SpecByte16,
}

#[derive(Default)]
pub(crate) struct AvatarBuilder {
	inner: Avatar,
}

impl AvatarBuilder {
	pub fn with_dna(season_id: SeasonId, dna: Dna) -> Self {
		Self { inner: Avatar { season_id, version: AvatarVersion::V2, dna, souls: 0 } }
	}

	pub fn with_base_avatar(avatar: Avatar) -> Self {
		Self { inner: avatar }
	}

	pub fn with_attribute<T>(self, attribute: &AvatarAttributes, value: &T) -> Self
	where
		T: ByteConvertible,
	{
		self.with_attribute_raw(attribute, value.as_byte())
	}

	pub fn with_attribute_raw(mut self, attribute: &AvatarAttributes, value: u8) -> Self {
		AvatarUtils::write_attribute(&mut self.inner, attribute, value);
		self
	}

	pub fn with_spec_byte_raw(mut self, spec_byte: &AvatarSpecBytes, value: u8) -> Self {
		AvatarUtils::write_spec_byte(&mut self.inner, spec_byte, value);
		self
	}

	pub fn with_spec_bytes(mut self, spec_bytes: [u8; 16]) -> Self {
		AvatarUtils::write_full_spec_bytes(&mut self.inner, spec_bytes);
		self
	}

	pub fn with_soul_count(mut self, soul_count: SoulCount) -> Self {
		self.inner.souls = soul_count;
		self
	}

	pub fn with_progress_array(mut self, progress_array: [u8; 11]) -> Self {
		AvatarUtils::write_progress_array(&mut self.inner, progress_array);
		self
	}

	pub fn into_pet(
		self,
		pet_type: &PetType,
		pet_variation: u8,
		spec_bytes: [u8; 16],
		progress_array: [u8; 11],
		soul_points: SoulCount,
	) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Pet)
			.with_attribute(&AvatarAttributes::ItemSubType, &PetItemType::Pet)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, pet_type)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Legendary)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_attribute_raw(&AvatarAttributes::CustomType2, pet_variation)
			.with_spec_bytes(spec_bytes)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_pet_part(self, pet_type: &PetType, slot_type: &SlotType, quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
		let base_0 = AvatarUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorBase.as_byte() as usize,
		);
		let comp_1 = AvatarUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent1.as_byte() as usize,
		);
		let comp_2 = AvatarUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent2.as_byte() as usize,
		);
		let comp_3 = AvatarUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent3.as_byte() as usize,
		);

		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Pet)
			.with_attribute(&AvatarAttributes::ItemSubType, &PetItemType::PetPart)
			.with_attribute(&AvatarAttributes::ClassType1, slot_type)
			.with_attribute(&AvatarAttributes::ClassType2, pet_type)
			.with_attribute(&AvatarAttributes::CustomType1, &custom_type_1)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(&AvatarAttributes::Quantity, quantity)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte1,
				AvatarUtils::enums_to_bits(&base_0) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte2,
				AvatarUtils::enums_order_to_bits(&base_0) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte3,
				AvatarUtils::enums_to_bits(&comp_1) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte4,
				AvatarUtils::enums_order_to_bits(&comp_1) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte5,
				AvatarUtils::enums_to_bits(&comp_2) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte6,
				AvatarUtils::enums_order_to_bits(&comp_2) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte7,
				AvatarUtils::enums_to_bits(&comp_3) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte8,
				AvatarUtils::enums_order_to_bits(&comp_3) as u8,
			)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_egg(
		self,
		rarity: &RarityTier,
		pet_variation: u8,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Pet)
			.with_attribute(&AvatarAttributes::ItemSubType, &PetItemType::Egg)
			// Unused
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			// Unused
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, rarity)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_attribute_raw(&AvatarAttributes::CustomType2, pet_variation)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_material(self, material_type: &MaterialItemType, quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Material)
			.with_attribute(&AvatarAttributes::ItemSubType, material_type)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &custom_type_1)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Common)
			.with_attribute_raw(&AvatarAttributes::Quantity, quantity)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_glimmer(self, quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Essence)
			.with_attribute(&AvatarAttributes::ItemSubType, &EssenceItemType::Glimmer)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &custom_type_1)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(&AvatarAttributes::Quantity, quantity)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_color_spark(
		self,
		color_pair: &(ColorType, ColorType),
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Essence)
			.with_attribute(&AvatarAttributes::ItemSubType, &EssenceItemType::ColorSpark)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, color_pair.0.as_byte())
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, color_pair.1.as_byte())
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_glow_spark(
		self,
		force: &Force,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Essence)
			.with_attribute(&AvatarAttributes::ItemSubType, &EssenceItemType::GlowSpark)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, force.as_byte())
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_paint_flask(
		self,
		color_pair: &(ColorType, ColorType),
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		let color_bytes = ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
			((color_pair.1.as_byte().saturating_sub(1)) << 4);

		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Essence)
			.with_attribute(&AvatarAttributes::ItemSubType, &EssenceItemType::PaintFlask)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, color_bytes)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, 0b0000_1000)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_glow_flask(
		self,
		force: &Force,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Essence)
			.with_attribute(&AvatarAttributes::ItemSubType, &EssenceItemType::GlowFlask)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, force.as_byte())
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn try_into_armor_and_component<T: Config>(
		self,
		pet_type: &PetType,
		slot_type: &SlotType,
		equippable_type: &[EquippableItemType],
		rarity: &RarityTier,
		color_pair: &(ColorType, ColorType),
		force: &Force,
		soul_points: SoulCount,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Self, ()> {
		if equippable_type.is_empty() ||
			equippable_type.iter().any(|equip| !EquippableItemType::is_armor(*equip))
		{
			return Err(())
		}

		let (armor_assemble_progress, color_flag) = {
			let mut color_flag = 0b0000_0000;
			let mut progress = AvatarUtils::enums_to_bits(equippable_type) as u8;

			if color_pair.0 != ColorType::None && color_pair.1 != ColorType::None {
				color_flag = 0b0000_1000;
				progress |= ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
					((color_pair.1.as_byte().saturating_sub(1)) << 4)
			}

			(progress, color_flag)
		};

		// Guaranteed to work because of check above
		let first_equippable = equippable_type.first().unwrap();

		let progress_array = AvatarUtils::generate_progress_bytes(
			rarity,
			SCALING_FACTOR_PERC,
			PROGRESS_PROBABILITY_PERC,
			hash_provider,
		);

		Ok(self
			.with_attribute(&AvatarAttributes::ItemType, &ItemType::Equippable)
			.with_attribute(&AvatarAttributes::ItemSubType, first_equippable)
			.with_attribute(&AvatarAttributes::ClassType1, slot_type)
			.with_attribute(&AvatarAttributes::ClassType2, pet_type)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, rarity)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, armor_assemble_progress)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, force.as_byte() | color_flag)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points))
	}

	pub fn try_into_weapon<T: Config>(
		self,
		pet_type: &PetType,
		slot_type: &SlotType,
		equippable_type: &EquippableItemType,
		color_pair: &(ColorType, ColorType),
		force: &Force,
		soul_points: SoulCount,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Self, ()> {
		if !EquippableItemType::is_weapon(*equippable_type) {
			return Err(())
		}

		let (weapon_info, color_flag) = {
			let mut color_flag = 0b0000_0000;
			let mut info = AvatarUtils::enums_to_bits(&[*equippable_type]) as u8 >> 4;

			if color_pair.0 != ColorType::None && color_pair.1 != ColorType::None {
				color_flag = 0b0000_1000;
				info |= ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
					((color_pair.1.as_byte().saturating_sub(1)) << 4)
			}

			(info, color_flag)
		};

		let rarity = RarityTier::Legendary;

		let progress_array = AvatarUtils::generate_progress_bytes(
			&rarity,
			SCALING_FACTOR_PERC,
			PROGRESS_PROBABILITY_PERC,
			hash_provider,
		);

		Ok(self
			.with_attribute(&AvatarAttributes::ItemType, &ItemType::Equippable)
			.with_attribute(&AvatarAttributes::ItemSubType, equippable_type)
			.with_attribute(&AvatarAttributes::ClassType1, slot_type)
			.with_attribute(&AvatarAttributes::ClassType2, pet_type)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &rarity)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, weapon_info)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, force.as_byte() | color_flag)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, 0)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points))
	}

	pub fn into_blueprint(
		self,
		blueprint_type: &BlueprintItemType,
		pet_type: &PetType,
		slot_type: &SlotType,
		equippable_item_type: &EquippableItemType,
		pattern: &[MaterialItemType],
		soul_points: SoulCount,
	) -> Self {
		// TODO: add a quantity algorithm
		// - base 8 - 16 and
		// - components 6 - 12
		let mat_req1 = 1;
		let mat_req2 = 1;
		let mat_req3 = 1;
		let mat_req4 = 1;

		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Blueprint)
			.with_attribute(&AvatarAttributes::ItemSubType, blueprint_type)
			.with_attribute(&AvatarAttributes::ClassType1, slot_type)
			.with_attribute(&AvatarAttributes::ClassType2, pet_type)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X1)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(&AvatarAttributes::Quantity, soul_points as u8)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte1,
				AvatarUtils::enums_to_bits(pattern) as u8,
			)
			.with_spec_byte_raw(
				&AvatarSpecBytes::SpecByte2,
				AvatarUtils::enums_order_to_bits(pattern) as u8,
			)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte3, equippable_item_type.as_byte())
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte4, mat_req1)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte5, mat_req2)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte6, mat_req3)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte7, mat_req4)
			.with_soul_count(soul_points)
	}

	pub fn into_unidentified(
		self,
		color_pair: (ColorType, ColorType),
		force: Force,
		soul_points: SoulCount,
	) -> Self {
		let git_info = 0b0000_1111 |
			((color_pair.0.as_byte().saturating_sub(1)) << 6 |
				(color_pair.1.as_byte().saturating_sub(1)) << 4);

		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Special)
			.with_attribute(&AvatarAttributes::ItemSubType, &SpecialItemType::Unidentified)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Legendary)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte1, git_info)
			.with_spec_byte_raw(&AvatarSpecBytes::SpecByte2, force.as_byte())
			.with_soul_count(soul_points)
	}

	pub fn into_dust(self, soul_points: SoulCount) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Special)
			.with_attribute(&AvatarAttributes::ItemSubType, &SpecialItemType::Dust)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X1)
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Common)
			.with_attribute_raw(&AvatarAttributes::Quantity, soul_points as u8)
			.with_soul_count(soul_points)
	}

	pub fn into_toolbox(self, soul_points: SoulCount) -> Self {
		self.with_attribute(&AvatarAttributes::ItemType, &ItemType::Special)
			.with_attribute(&AvatarAttributes::ItemSubType, &SpecialItemType::ToolBox)
			.with_attribute(&AvatarAttributes::ClassType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::ClassType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType1, &HexType::X0)
			.with_attribute(&AvatarAttributes::CustomType2, &HexType::X0)
			.with_attribute(&AvatarAttributes::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(&AvatarAttributes::Quantity, 1)
			.with_progress_array([0xBB; 11])
			.with_soul_count(soul_points)
	}

	pub fn build(self) -> Avatar {
		self.inner
	}
}

/// Struct to wrap DNA interactions with Avatars from V2 upwards.
/// Don't use with Avatars with V1.
pub(crate) struct AvatarUtils;

impl AvatarUtils {
	pub fn has_attribute_with_same_value_as(
		avatar: &Avatar,
		other: &Avatar,
		attribute: &AvatarAttributes,
	) -> bool {
		Self::read_attribute(avatar, attribute) == Self::read_attribute(other, attribute)
	}

	pub fn has_attribute_set_with_same_values_as(
		avatar: &Avatar,
		other: &Avatar,
		attribute_set: &[AvatarAttributes],
	) -> bool {
		attribute_set
			.iter()
			.all(|attribute| Self::has_attribute_with_same_value_as(avatar, other, attribute))
	}

	fn read_dna_strand(avatar: &Avatar, position: usize, byte_type: &ByteType) -> u8 {
		Self::read_dna_at(avatar.dna.as_slice(), position, byte_type)
	}

	fn read_dna_at(dna: &[u8], position: usize, byte_type: &ByteType) -> u8 {
		match byte_type {
			ByteType::Full => dna[position],
			ByteType::High => Self::high_nibble_of(dna[position]),
			ByteType::Low => Self::low_nibble_of(dna[position]),
		}
	}

	pub fn high_nibble_of(byte: u8) -> u8 {
		byte >> 4
	}

	pub fn low_nibble_of(byte: u8) -> u8 {
		byte & 0x0F
	}

	fn write_dna_strand(avatar: &mut Avatar, position: usize, byte_type: ByteType, value: u8) {
		match byte_type {
			ByteType::Full => avatar.dna[position] = value,
			ByteType::High =>
				avatar.dna[position] =
					(avatar.dna[position] & (ByteType::High as u8)) | (value << 4),
			ByteType::Low =>
				avatar.dna[position] = (avatar.dna[position] & (ByteType::Low as u8)) |
					(value & (ByteType::High as u8)),
		}
	}

	fn write_dna_at(dna: &mut [u8], position: usize, byte_type: ByteType, value: u8) {
		match byte_type {
			ByteType::Full => dna[position] = value,
			ByteType::High =>
				dna[position] = (dna[position] & (ByteType::High as u8)) | (value << 4),
			ByteType::Low =>
				dna[position] =
					(dna[position] & (ByteType::Low as u8)) | (value & (ByteType::High as u8)),
		}
	}

	pub fn has_attribute_set_with_values(
		avatar: &Avatar,
		attribute_value_set: &[(AvatarAttributes, u8)],
	) -> bool {
		attribute_value_set
			.iter()
			.all(|(attr, value)| Self::has_attribute_with_value_raw(avatar, attr, *value))
	}

	pub fn has_attribute_with_value<T>(
		avatar: &Avatar,
		attribute: &AvatarAttributes,
		value: T,
	) -> bool
	where
		T: ByteConvertible,
	{
		Self::has_attribute_with_value_raw(avatar, attribute, value.as_byte())
	}

	pub fn has_attribute_with_value_different_than<T>(
		avatar: &Avatar,
		attribute: &AvatarAttributes,
		value: T,
	) -> bool
	where
		T: ByteConvertible + PartialEq,
	{
		Self::read_attribute_as::<T>(avatar, attribute) != value
	}

	pub fn has_attribute_with_value_raw(
		avatar: &Avatar,
		attribute: &AvatarAttributes,
		value: u8,
	) -> bool {
		Self::read_attribute(avatar, attribute) == value
	}

	pub fn read_attribute_as<T>(avatar: &Avatar, attribute: &AvatarAttributes) -> T
	where
		T: ByteConvertible,
	{
		T::from_byte(Self::read_attribute(avatar, attribute))
	}

	pub fn read_attribute(avatar: &Avatar, attribute: &AvatarAttributes) -> u8 {
		match attribute {
			AvatarAttributes::ItemType => Self::read_dna_strand(avatar, 0, &ByteType::High),
			AvatarAttributes::ItemSubType => Self::read_dna_strand(avatar, 0, &ByteType::Low),
			AvatarAttributes::ClassType1 => Self::read_dna_strand(avatar, 1, &ByteType::High),
			AvatarAttributes::ClassType2 => Self::read_dna_strand(avatar, 1, &ByteType::Low),
			AvatarAttributes::CustomType1 => Self::read_dna_strand(avatar, 2, &ByteType::High),
			AvatarAttributes::CustomType2 => Self::read_dna_strand(avatar, 4, &ByteType::Full),
			AvatarAttributes::RarityTier => Self::read_dna_strand(avatar, 2, &ByteType::Low),
			AvatarAttributes::Quantity => Self::read_dna_strand(avatar, 3, &ByteType::Full),
		}
	}

	pub fn write_typed_attribute<T>(avatar: &mut Avatar, attribute: &AvatarAttributes, value: &T)
	where
		T: ByteConvertible,
	{
		Self::write_attribute(avatar, attribute, value.as_byte())
	}

	pub fn write_attribute(avatar: &mut Avatar, attribute: &AvatarAttributes, value: u8) {
		match attribute {
			AvatarAttributes::ItemType => Self::write_dna_strand(avatar, 0, ByteType::High, value),
			AvatarAttributes::ItemSubType =>
				Self::write_dna_strand(avatar, 0, ByteType::Low, value),
			AvatarAttributes::ClassType1 =>
				Self::write_dna_strand(avatar, 1, ByteType::High, value),
			AvatarAttributes::ClassType2 => Self::write_dna_strand(avatar, 1, ByteType::Low, value),
			AvatarAttributes::CustomType1 =>
				Self::write_dna_strand(avatar, 2, ByteType::High, value),
			AvatarAttributes::CustomType2 =>
				Self::write_dna_strand(avatar, 4, ByteType::Full, value),
			AvatarAttributes::RarityTier => Self::write_dna_strand(avatar, 2, ByteType::Low, value),
			AvatarAttributes::Quantity => Self::write_dna_strand(avatar, 3, ByteType::Full, value),
		}
	}

	pub fn read_full_spec_bytes(avatar: &Avatar) -> [u8; 16] {
		let mut out = [0; 16];
		out.copy_from_slice(&avatar.dna[5..21]);
		out
	}

	pub fn read_spec_byte(avatar: &Avatar, spec_byte: &AvatarSpecBytes) -> u8 {
		match spec_byte {
			AvatarSpecBytes::SpecByte1 => Self::read_dna_strand(avatar, 5, &ByteType::Full),
			AvatarSpecBytes::SpecByte2 => Self::read_dna_strand(avatar, 6, &ByteType::Full),
			AvatarSpecBytes::SpecByte3 => Self::read_dna_strand(avatar, 7, &ByteType::Full),
			AvatarSpecBytes::SpecByte4 => Self::read_dna_strand(avatar, 8, &ByteType::Full),
			AvatarSpecBytes::SpecByte5 => Self::read_dna_strand(avatar, 9, &ByteType::Full),
			AvatarSpecBytes::SpecByte6 => Self::read_dna_strand(avatar, 10, &ByteType::Full),
			AvatarSpecBytes::SpecByte7 => Self::read_dna_strand(avatar, 11, &ByteType::Full),
			AvatarSpecBytes::SpecByte8 => Self::read_dna_strand(avatar, 12, &ByteType::Full),
			AvatarSpecBytes::SpecByte9 => Self::read_dna_strand(avatar, 13, &ByteType::Full),
			AvatarSpecBytes::SpecByte10 => Self::read_dna_strand(avatar, 14, &ByteType::Full),
			AvatarSpecBytes::SpecByte11 => Self::read_dna_strand(avatar, 15, &ByteType::Full),
			AvatarSpecBytes::SpecByte12 => Self::read_dna_strand(avatar, 16, &ByteType::Full),
			AvatarSpecBytes::SpecByte13 => Self::read_dna_strand(avatar, 17, &ByteType::Full),
			AvatarSpecBytes::SpecByte14 => Self::read_dna_strand(avatar, 18, &ByteType::Full),
			AvatarSpecBytes::SpecByte15 => Self::read_dna_strand(avatar, 19, &ByteType::Full),
			AvatarSpecBytes::SpecByte16 => Self::read_dna_strand(avatar, 20, &ByteType::Full),
		}
	}

	pub fn read_spec_byte_as<T>(avatar: &Avatar, spec_byte: &AvatarSpecBytes) -> T
	where
		T: ByteConvertible,
	{
		T::from_byte(Self::read_spec_byte(avatar, spec_byte))
	}

	pub fn write_full_spec_bytes(avatar: &mut Avatar, value: [u8; 16]) {
		(avatar.dna[5..21]).copy_from_slice(&value);
	}

	pub fn write_spec_byte(avatar: &mut Avatar, spec_byte: &AvatarSpecBytes, value: u8) {
		match spec_byte {
			AvatarSpecBytes::SpecByte1 => Self::write_dna_strand(avatar, 5, ByteType::Full, value),
			AvatarSpecBytes::SpecByte2 => Self::write_dna_strand(avatar, 6, ByteType::Full, value),
			AvatarSpecBytes::SpecByte3 => Self::write_dna_strand(avatar, 7, ByteType::Full, value),
			AvatarSpecBytes::SpecByte4 => Self::write_dna_strand(avatar, 8, ByteType::Full, value),
			AvatarSpecBytes::SpecByte5 => Self::write_dna_strand(avatar, 9, ByteType::Full, value),
			AvatarSpecBytes::SpecByte6 => Self::write_dna_strand(avatar, 10, ByteType::Full, value),
			AvatarSpecBytes::SpecByte7 => Self::write_dna_strand(avatar, 11, ByteType::Full, value),
			AvatarSpecBytes::SpecByte8 => Self::write_dna_strand(avatar, 12, ByteType::Full, value),
			AvatarSpecBytes::SpecByte9 => Self::write_dna_strand(avatar, 13, ByteType::Full, value),
			AvatarSpecBytes::SpecByte10 =>
				Self::write_dna_strand(avatar, 14, ByteType::Full, value),
			AvatarSpecBytes::SpecByte11 =>
				Self::write_dna_strand(avatar, 15, ByteType::Full, value),
			AvatarSpecBytes::SpecByte12 =>
				Self::write_dna_strand(avatar, 16, ByteType::Full, value),
			AvatarSpecBytes::SpecByte13 =>
				Self::write_dna_strand(avatar, 17, ByteType::Full, value),
			AvatarSpecBytes::SpecByte14 =>
				Self::write_dna_strand(avatar, 18, ByteType::Full, value),
			AvatarSpecBytes::SpecByte15 =>
				Self::write_dna_strand(avatar, 19, ByteType::Full, value),
			AvatarSpecBytes::SpecByte16 =>
				Self::write_dna_strand(avatar, 20, ByteType::Full, value),
		}
	}

	pub fn has_same_spec_byte_as(
		avatar: &Avatar,
		other: &Avatar,
		spec_byte: &AvatarSpecBytes,
	) -> bool {
		Self::read_spec_byte(avatar, spec_byte) == Self::read_spec_byte(other, spec_byte)
	}

	pub fn same_item_type_and_class_types(avatar_1: &Avatar, avatar_2: &Avatar) -> bool {
		AvatarUtils::has_attribute_set_with_same_values_as(
			avatar_1,
			avatar_2,
			&[
				AvatarAttributes::ItemType,
				AvatarAttributes::ItemSubType,
				AvatarAttributes::ClassType1,
				AvatarAttributes::ClassType2,
			],
		)
	}

	pub fn same_assemble_version(avatar_1: &Avatar, avatar_2: &Avatar) -> bool {
		AvatarUtils::has_attribute_set_with_same_values_as(
			avatar_1,
			avatar_2,
			&[
				AvatarAttributes::ItemType,
				AvatarAttributes::ClassType1,
				AvatarAttributes::ClassType2,
			],
		)
	}

	// TODO: Improve return type to [[u8; 3]; 10] if possible
	pub fn spec_byte_split_ten(avatar: &Avatar) -> Vec<Vec<u8>> {
		Self::read_full_spec_bytes(avatar)
			.into_iter()
			.flat_map(|entry| [entry >> 4, entry & 0x0F])
			.take(30)
			.collect::<Vec<u8>>()
			.chunks_exact(3)
			.map(|item| item.into())
			.collect::<Vec<Vec<u8>>>()
	}

	pub fn spec_byte_split_ten_count(avatar: &Avatar) -> usize {
		Self::spec_byte_split_ten(avatar)
			.into_iter()
			.filter(|segment| segment.iter().sum::<u8>() > 0)
			.count()
	}

	pub fn read_progress_array(avatar: &Avatar) -> [u8; 11] {
		let mut out = [0; 11];
		out.copy_from_slice(&avatar.dna[21..32]);
		out
	}

	pub fn is_array_match(
		array_1: [u8; 11],
		array_2: [u8; 11],
		rarity_level: u8,
	) -> Option<Vec<u32>> {
		let (mirror, matches) = Self::match_progress_arrays(array_1, array_2, rarity_level);
		let match_count = matches.len() as u32;
		let mirror_count = mirror.len() as u32;

		(match_count > 0 && (((match_count * 2) + mirror_count) >= 6)).then_some(matches)
	}

	pub fn match_progress_arrays(
		array_1: [u8; 11],
		array_2: [u8; 11],
		rarity_level: u8,
	) -> (Vec<u32>, Vec<u32>) {
		let mut matches = Vec::<u32>::new();
		let mut mirrors = Vec::<u32>::new();

		let lowest_1 = Self::read_lowest_progress_byte(&array_1, &ByteType::High);

		for i in 0..array_1.len() {
			let rarity_1 = Self::read_dna_at(&array_1, i, &ByteType::High);
			let variation_1 = Self::read_dna_at(&array_1, i, &ByteType::Low);

			let rarity_2 = Self::read_dna_at(&array_2, i, &ByteType::High);
			let variation_2 = Self::read_dna_at(&array_2, i, &ByteType::Low);

			let have_same_rarity = rarity_1 == rarity_2 || rarity_2 == 0x0B;
			let is_maxed = rarity_1 > lowest_1 || lowest_1 == RarityTier::Legendary.as_byte();
			let byte_match = Self::match_progress_byte(variation_1, variation_2);

			if have_same_rarity &&
				!is_maxed && (rarity_1 < rarity_level || variation_2 == 0x0B || byte_match)
			{
				matches.push(i as u32);
			} else if is_maxed && (variation_1 == variation_2) {
				mirrors.push(i as u32);
			}
		}

		(mirrors, matches)
	}

	fn match_progress_byte(byte_1: u8, byte_2: u8) -> bool {
		let diff = if byte_1 >= byte_2 { byte_1 - byte_2 } else { byte_2 - byte_1 };
		diff == 1 || diff == (PROGRESS_VARIATIONS - 1)
	}

	pub fn write_progress_array(avatar: &mut Avatar, value: [u8; 11]) {
		(avatar.dna[21..32]).copy_from_slice(&value);
	}

	pub fn can_use_avatar(avatar: &Avatar, quantity: u8) -> bool {
		Self::read_attribute(avatar, &AvatarAttributes::Quantity) >= quantity
	}

	pub fn use_avatar(avatar: &mut Avatar, quantity: u8) -> (bool, bool, SoulCount) {
		let current_qty = Self::read_attribute(avatar, &AvatarAttributes::Quantity);

		if current_qty < quantity {
			return (false, false, 0)
		}

		let new_qty = current_qty - quantity;
		Self::write_attribute(avatar, &AvatarAttributes::Quantity, new_qty);

		let (avatar_consumed, output_soul_points) = if new_qty == 0 {
			let soul_points = avatar.souls;
			avatar.souls = 0;
			(true, soul_points)
		} else {
			let diff = Self::read_attribute(avatar, &AvatarAttributes::CustomType1)
				.saturating_mul(quantity) as SoulCount;
			avatar.souls = avatar.souls.saturating_sub(diff);
			(false, diff)
		};

		(true, avatar_consumed, output_soul_points)
	}

	pub fn create_pattern<T>(mut base_seed: usize, increase_seed: usize) -> Vec<T>
	where
		T: ByteConvertible + Ranged,
	{
		// Equivalent to "0X35AAB76B4482CADFF35BB3BD1C86648697B6F6833B47B939AECE95EDCD0347"
		let fixed_seed: [u8; 32] = [
			0x33, 0x35, 0xAA, 0xB7, 0x6B, 0x44, 0x82, 0xCA, 0xDF, 0xF3, 0x5B, 0xB3, 0xBD, 0x1C,
			0x86, 0x64, 0x86, 0x97, 0xB6, 0xF6, 0x83, 0x3B, 0x47, 0xB9, 0x39, 0xAE, 0xCE, 0x95,
			0xED, 0xCD, 0x03, 0x47,
		];

		let mut all_enum = T::range().map(|variant| variant as u8).collect::<Vec<_>>();
		let mut pattern = Vec::with_capacity(4);

		for _ in 0..4 {
			base_seed = base_seed.saturating_add(increase_seed);
			let rand_1 = fixed_seed[base_seed % 32];

			let enum_type = all_enum.remove(rand_1 as usize % all_enum.len());
			pattern.push(enum_type);
		}

		pattern.into_iter().map(|item| T::from_byte(item)).collect()
	}

	pub fn enums_to_bits<T>(enum_list: &[T]) -> u32
	where
		T: ByteConvertible + Ranged,
	{
		let range_mod = T::range().start as u8;
		enum_list
			.iter()
			.fold(0_u32, |acc, entry| acc | (1 << (entry.as_byte().saturating_sub(range_mod))))
	}

	pub fn enums_order_to_bits<T>(enum_list: &[T]) -> u32
	where
		T: Clone + Ord,
	{
		let mut sorted_list = Vec::with_capacity(enum_list.len());
		sorted_list.extend_from_slice(enum_list);
		sorted_list.sort();

		let mut byte_buff = 0;
		let fill_amount = usize::BITS - sorted_list.len().saturating_sub(1).leading_zeros();

		for entry in enum_list {
			if let Ok(index) = sorted_list.binary_search(entry) {
				byte_buff |= index as u32;
				byte_buff <<= fill_amount;
			}
		}

		byte_buff >> fill_amount
	}

	pub fn bits_to_enums<T>(bits: u32) -> Vec<T>
	where
		T: ByteConvertible + Ranged,
	{
		let mut enums = Vec::new();

		for (i, value) in T::range().enumerate() {
			if (bits & (1 << i)) != 0 {
				enums.push(T::from_byte(value as u8));
			}
		}

		enums
	}

	pub fn bits_order_to_enum<T>(bit_order: u32, step_count: usize, enum_list: Vec<T>) -> Vec<T>
	where
		T: Clone + Ord,
	{
		let mut sorted_enum_list = enum_list;
		sorted_enum_list.sort();

		let mut output_enums = Vec::new();

		let mask_width = step_count * 2;
		let bit_mask = 0b0000_0000_0000_0000_0000_0000_0000_0011 << mask_width.saturating_sub(2);

		for i in (0..mask_width).step_by(2) {
			let bit_segment = bit_order & (bit_mask >> i);
			let bit_position = (bit_segment >> (mask_width - (i + 2))) as usize;

			if sorted_enum_list.len() > bit_position {
				output_enums.push(sorted_enum_list[bit_position].clone());
			}
		}

		output_enums
	}

	pub fn generate_progress_bytes<T: Config>(
		rarity: &RarityTier,
		scale_factor: u32,
		probability: u32,
		hash_provider: &mut HashProvider<T, 32>,
	) -> [u8; 11] {
		let mut progress_bytes = [0; 11];

		for i in 0..progress_bytes.len() {
			let random_value = hash_provider.get_hash_byte();

			// Upcast random_value
			let new_rarity =
				if (random_value as u32).saturating_mul(scale_factor) < (probability * MAX_BYTE) {
					rarity.upgrade().as_byte()
				} else {
					rarity.as_byte()
				};

			Self::write_dna_at(&mut progress_bytes, i, ByteType::High, new_rarity);
			Self::write_dna_at(
				&mut progress_bytes,
				i,
				ByteType::Low,
				random_value % PROGRESS_VARIATIONS,
			);
		}

		Self::write_dna_at(&mut progress_bytes, 10, ByteType::High, rarity.as_byte());

		progress_bytes
	}

	pub fn read_lowest_progress_byte(progress_bytes: &[u8; 11], byte_type: &ByteType) -> u8 {
		let mut result = u8::MAX;

		for i in 0..progress_bytes.len() {
			let value = Self::read_dna_at(progress_bytes, i, byte_type);
			if result > value {
				result = value;
			}
		}

		result
	}

	#[allow(dead_code)]
	pub fn read_lowest_progress_indexes(
		progress_bytes: &[u8; 11],
		byte_type: &ByteType,
	) -> Vec<usize> {
		let mut lowest = u8::MAX;

		let mut result = Vec::new();

		for i in 0..progress_bytes.len() {
			let value = Self::read_dna_at(progress_bytes, i, byte_type);

			match lowest.cmp(&value) {
				Ordering::Greater => {
					lowest = value;
					result = Vec::new();
					result.push(i);
				},
				Ordering::Equal => result.push(i),
				_ => continue,
			}
		}

		result
	}
}

pub(crate) struct HashProvider<T: Config, const N: usize> {
	pub(crate) hash: [u8; N],
	current_index: usize,
	_marker: PhantomData<T>,
}

impl<T: Config, const N: usize> Default for HashProvider<T, N> {
	fn default() -> Self {
		Self { hash: [0; N], current_index: 0, _marker: PhantomData }
	}
}

impl<T: Config, const N: usize> HashProvider<T, N> {
	pub fn new(hash: &T::Hash) -> Self {
		Self::new_starting_at(hash, 0)
	}

	#[cfg(test)]
	pub fn new_with_bytes(bytes: [u8; N]) -> Self {
		Self { hash: bytes, current_index: 0, _marker: PhantomData }
	}

	pub fn new_starting_at(hash: &T::Hash, index: usize) -> Self {
		// TODO: Improve
		let mut bytes = [0; N];

		let hash_ref = hash.as_ref();
		let hash_len = hash_ref.len();

		bytes[0..hash_len].copy_from_slice(hash_ref);

		Self { hash: bytes, current_index: index, _marker: PhantomData }
	}

	#[cfg(test)]
	pub fn full_hash(&self, mutate_seed: usize) -> T::Hash {
		let mut full_hash = self.hash;

		for (i, hash) in full_hash.iter_mut().enumerate() {
			*hash = self.hash[(i + mutate_seed) % N];
		}

		T::Hashing::hash(&full_hash)
	}

	pub fn get_hash_byte(&mut self) -> u8 {
		self.next().unwrap_or_default()
	}
}

impl<T: Config, const N: usize> Iterator for HashProvider<T, N> {
	type Item = u8;

	fn next(&mut self) -> Option<Self::Item> {
		let item = self.hash[self.current_index];
		self.current_index = (self.current_index + 1) % N;
		Some(item)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_bits_to_enums_consistency_1() {
		let bits = 0b_01_01_01_01;

		let result = AvatarUtils::bits_to_enums::<NibbleType>(bits);
		let expected = vec![NibbleType::X0, NibbleType::X2, NibbleType::X4, NibbleType::X6];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_bits_to_enums_consistency_2() {
		let bits = 0b_11_01_10_01;

		let result = AvatarUtils::bits_to_enums::<MaterialItemType>(bits);
		let expected = vec![
			MaterialItemType::Polymers,
			MaterialItemType::Optics,
			MaterialItemType::Metals,
			MaterialItemType::Superconductors,
			MaterialItemType::Nanomaterials,
		];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_bits_order_to_enums_consistency_1() {
		let bit_order = 0b_01_10_11_00;
		let enum_list = vec![NibbleType::X0, NibbleType::X2, NibbleType::X4, NibbleType::X6];

		let result = AvatarUtils::bits_order_to_enum(bit_order, 4, enum_list);
		let expected = vec![NibbleType::X2, NibbleType::X4, NibbleType::X6, NibbleType::X0];
		assert_eq!(result, expected);

		let bit_order_2 = 0b_01_11_00_10;
		let enum_list_2 = vec![NibbleType::X4, NibbleType::X5, NibbleType::X6, NibbleType::X7];

		let result_2 = AvatarUtils::bits_order_to_enum(bit_order_2, 4, enum_list_2);
		let expected_2 = vec![NibbleType::X5, NibbleType::X7, NibbleType::X4, NibbleType::X6];
		assert_eq!(result_2, expected_2);
	}

	#[test]
	fn test_bits_order_to_enums_consistency_2() {
		let bit_order = 0b_01_10_00_10;
		let enum_list = vec![PetType::FoxishDude, PetType::FireDino, PetType::GiantWoodStick];

		let result = AvatarUtils::bits_order_to_enum(bit_order, 4, enum_list);
		let expected = vec![
			PetType::FireDino,
			PetType::GiantWoodStick,
			PetType::FoxishDude,
			PetType::GiantWoodStick,
		];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_enum_to_bits_consistency_1() {
		let pattern = vec![NibbleType::X2, NibbleType::X4, NibbleType::X1, NibbleType::X3];
		let expected = 0b_00_01_11_10;

		assert_eq!(AvatarUtils::enums_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_to_bits_consistency_2() {
		let pattern = vec![PetType::FoxishDude, PetType::BigHybrid, PetType::GiantWoodStick];
		let expected = 0b_00_11_00_10;

		assert_eq!(AvatarUtils::enums_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_order_to_bits_consistency() {
		let pattern = vec![
			SlotType::LegBack,
			SlotType::Breast,
			SlotType::WeaponFront,
			SlotType::WeaponBack,
			SlotType::ArmBack,
		];
		#[allow(clippy::unusual_byte_groupings)]
		// We group by 3 because the output is grouped by 3
		let expected = 0b_010_000_011_100_001;

		assert_eq!(AvatarUtils::enums_order_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_to_bits_to_enum() {
		let pattern = vec![
			MaterialItemType::Polymers,
			MaterialItemType::Superconductors,
			MaterialItemType::Ceramics,
		];

		let expected = vec![
			MaterialItemType::Polymers,
			MaterialItemType::Ceramics,
			MaterialItemType::Superconductors,
		];

		let bits = AvatarUtils::enums_to_bits(&pattern);
		assert_eq!(bits, 0b_01_10_00_01);

		let enums = AvatarUtils::bits_to_enums::<MaterialItemType>(bits);
		assert_eq!(enums, expected);
	}

	#[test]
	fn test_create_pattern_consistency() {
		let base_seed = SlotType::Head.as_byte() as usize;
		let pattern = AvatarUtils::create_pattern::<NibbleType>(
			base_seed,
			SlotType::Breast.as_byte() as usize,
		);

		let expected = vec![NibbleType::X7, NibbleType::X5, NibbleType::X4, NibbleType::X3];

		assert_eq!(pattern, expected);
	}

	#[test]
	fn tests_pattern_and_order() {
		let base_seed = (PetType::FoxishDude.as_byte() + SlotType::Head.as_byte()) as usize;

		let pattern_1 = AvatarUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorBase.as_byte() as usize,
		);
		let p10 = AvatarUtils::enums_to_bits(&pattern_1);
		let p11 = AvatarUtils::enums_order_to_bits(&pattern_1);

		assert_eq!(p10, 0b_01_10_11_00);
		assert_eq!(p11, 0b_01_11_10_00);

		// Decode Blueprint
		let unordered_1 = AvatarUtils::bits_to_enums::<NibbleType>(p10);
		let pattern_1_check = AvatarUtils::bits_order_to_enum(p11, 4, unordered_1);
		assert_eq!(pattern_1_check, pattern_1);

		// Pattern number and enum number only match if they are according to the index in the list
		let unordered_material = AvatarUtils::bits_to_enums::<MaterialItemType>(p10);
		assert_eq!(
			AvatarUtils::bits_order_to_enum(p11, 4, unordered_material)[0],
			MaterialItemType::Optics
		);

		let test_set: Vec<(EquippableItemType, u32, u32)> = vec![
			(EquippableItemType::ArmorComponent1, 0b_11_01_10_00, 0b_01_11_00_10),
			(EquippableItemType::ArmorComponent2, 0b_01_01_01_01, 0b_01_11_10_00),
			(EquippableItemType::ArmorComponent3, 0b_01_10_01_10, 0b_01_10_11_00),
		];

		for (armor_component, enum_to_bits, enum_order_to_bits) in test_set {
			let pattern_base = AvatarUtils::create_pattern::<NibbleType>(
				base_seed,
				armor_component.as_byte() as usize,
			);
			let p_enum_to_bits = AvatarUtils::enums_to_bits(&pattern_base);
			let p_enum_order_to_bits = AvatarUtils::enums_order_to_bits(&pattern_base);
			assert_eq!(p_enum_to_bits, enum_to_bits);
			assert_eq!(p_enum_order_to_bits, enum_order_to_bits);
			// Decode Blueprint
			let unordered_base = AvatarUtils::bits_to_enums::<NibbleType>(p_enum_to_bits);
			let pattern_base_check =
				AvatarUtils::bits_order_to_enum(p_enum_order_to_bits, 4, unordered_base);
			assert_eq!(pattern_base_check, pattern_base);
		}
	}

	#[test]
	fn test_match_progress_array_consistency() {
		let empty_vec = Vec::<u32>::new();

		let arr_1 = [0x00; 11];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10; 11];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x00; 11];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10; 11];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00];
		let arr_2 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10];
		let arr_2 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x00];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x00, 0x11, 0x02, 0x13, 0x04, 0x15, 0x04, 0x13, 0x02, 0x11, 0x00];
		let arr_2 = [0x01, 0x01, 0x12, 0x13, 0x04, 0x04, 0x13, 0x12, 0x01, 0x01, 0x15];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 8];
		let expected_mirrors: Vec<u32> = vec![1, 3, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);
	}

	#[test]
	fn test_match_progress_array_consistency_multiple() {
		let empty_vec = Vec::<u32>::new();

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x12, 0x13, 0x14, 0x13, 0x14, 0x11, 0x22, 0x10, 0x14, 0x22, 0x11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 7];
		let expected_mirrors: Vec<u32> = vec![5];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x10, 0x10, 0x10, 0x14, 0x14, 0x13, 0x13, 0x12, 0x15, 0x14, 0x14];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x15, 0x10, 0x14, 0x13, 0x13, 0x11, 0x10, 0x14, 0x12, 0x20, 0x11];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 7, 8];
		let expected_mirrors: Vec<u32> = vec![5];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x11, 0x11, 0x11, 0x10, 0x15, 0x12, 0x11, 0x11, 0x13, 0x12, 0x14];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 2, 3, 6, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_match_progress_consistency_on_level() {
		let empty_vec = Vec::<u32>::new();

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x44, 0x44, 0x42, 0x45];
		let arr_2 = [0x41, 0x51, 0x52, 0x53, 0x44, 0x52, 0x45, 0x41, 0x40, 0x41, 0x43];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 4, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x44, 0x44, 0x42, 0x45];
		let arr_2 = [0x52, 0x41, 0x43, 0x41, 0x53, 0x45, 0x43, 0x44, 0x52, 0x43, 0x43];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x54, 0x44, 0x42, 0x53];
		let arr_2 = [0x52, 0x40, 0x43, 0x41, 0x53, 0x45, 0x41, 0x44, 0x52, 0x43, 0x43];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![9];
		let expected_mirrors: Vec<u32> = vec![7, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x30, 0x30, 0x30, 0x30];
		let arr_2 = [0x45, 0x45, 0x45, 0x45, 0x45, 0x35, 0x45, 0x31, 0x30, 0x45, 0x45];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![7];
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x31, 0x30, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34];
		let arr_2 = [0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x22, 0x22, 0x22];
		let (mirrors, matches) = AvatarUtils::match_progress_arrays(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![3];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);
	}
}
