use super::{constants::*, types::*, ByteType};
use crate::{
	types::{Avatar, Dna, DnaEncoding, SeasonId, SoulCount},
	ByteConvertible, Config, Force, Ranged, RarityTier,
};
//use frame_support::traits::Len;
use sp_runtime::{traits::Hash, SaturatedConversion};
use sp_std::{
	cmp::Ordering,
	marker::PhantomData,
	ops::{Div, Rem},
	vec::Vec,
};

#[derive(Clone)]
pub(crate) struct WrappedAvatar {
	inner: Avatar,
}

impl WrappedAvatar {
	pub fn new(avatar: Avatar) -> Self {
		Self { inner: avatar }
	}

	pub fn unwrap(self) -> Avatar {
		self.inner
	}

	pub fn get_dna(&self) -> &Dna {
		&self.inner.dna
	}

	pub fn get_souls(&self) -> SoulCount {
		self.inner.souls
	}

	pub fn set_souls(&mut self, souls: SoulCount) {
		self.inner.souls = souls;
	}

	pub fn add_souls(&mut self, souls: SoulCount) {
		self.inner.souls += souls;
	}

	pub fn dec_souls(&mut self, souls: SoulCount) {
		self.inner.souls -= souls;
	}

	pub fn can_use(&self, quantity: u8) -> bool {
		self.get_quantity() >= quantity
	}

	pub fn use_avatar(&mut self, quantity: u8) -> (bool, bool, SoulCount) {
		let current_qty = self.get_quantity();

		if current_qty < quantity {
			return (false, false, 0)
		}

		let new_qty = current_qty - quantity;
		self.set_quantity(new_qty);

		let (avatar_consumed, output_soul_points) = if new_qty == 0 {
			let soul_points = self.get_souls();
			self.set_souls(0);
			(true, soul_points)
		} else {
			let diff = self.get_custom_type_1::<u8>().saturating_mul(quantity) as SoulCount;
			self.set_souls(self.get_souls().saturating_sub(diff));
			(false, diff)
		};

		(true, avatar_consumed, output_soul_points)
	}

	// TODO: Improve return type to [[u8; 3]; 10] if possible
	pub fn spec_byte_split_ten(&self) -> Vec<Vec<u8>> {
		self.get_specs()
			.into_iter()
			.flat_map(|entry| [entry >> 4, entry & 0x0F])
			.take(30)
			.collect::<Vec<u8>>()
			.chunks_exact(3)
			.map(|item| item.into())
			.collect::<Vec<Vec<u8>>>()
	}

	pub fn spec_byte_split_ten_count(&self) -> usize {
		self.spec_byte_split_ten()
			.into_iter()
			.filter(|segment| segment.iter().sum::<u8>() > 0)
			.count()
	}

	pub fn get_item_type(&self) -> ItemType {
		DnaUtils::read_attribute(&self.inner, AvatarAttr::ItemType)
	}

	pub fn set_item_type(&mut self, item_type: ItemType) {
		DnaUtils::write_attribute(&mut self.inner, AvatarAttr::ItemType, &item_type)
	}

	pub fn same_item_type(&self, other: &WrappedAvatar) -> bool {
		self.get_item_type().cmp(&other.get_item_type()).is_eq()
	}

	pub fn get_item_sub_type<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::ItemSubType)
	}

	pub fn set_item_sub_type<T>(&mut self, item_sub_type: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::ItemSubType, &item_sub_type)
	}

	pub fn same_item_sub_type(&self, other: &WrappedAvatar) -> bool {
		self.get_item_sub_type::<u8>().cmp(&other.get_item_sub_type::<u8>()).is_eq()
	}

	pub fn get_class_type_1<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::ClassType1)
	}

	pub fn set_class_type_1<T>(&mut self, class_type_1: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::ClassType1, &class_type_1)
	}

	pub fn same_class_type_1(&self, other: &WrappedAvatar) -> bool {
		self.get_class_type_1::<u8>().cmp(&other.get_class_type_1::<u8>()).is_eq()
	}

	pub fn get_class_type_2<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::ClassType2)
	}

	pub fn set_class_type_2<T>(&mut self, class_type_2: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::ClassType2, &class_type_2)
	}

	pub fn same_class_type_2(&self, other: &WrappedAvatar) -> bool {
		self.get_class_type_2::<u8>().cmp(&other.get_class_type_2::<u8>()).is_eq()
	}

	pub fn get_custom_type_1<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::CustomType1)
	}

	pub fn set_custom_type_1<T>(&mut self, custom_type_1: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::CustomType1, &custom_type_1)
	}

	pub fn same_custom_type_1(&self, other: &WrappedAvatar) -> bool {
		self.get_custom_type_1::<u8>().cmp(&other.get_custom_type_1::<u8>()).is_eq()
	}

	pub fn get_custom_type_2<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::CustomType2)
	}

	pub fn set_custom_type_2<T>(&mut self, custom_type_1: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::CustomType2, &custom_type_1)
	}

	pub fn same_custom_type_2(&self, other: &WrappedAvatar) -> bool {
		self.get_custom_type_2::<u8>().cmp(&other.get_custom_type_2::<u8>()).is_eq()
	}

	pub fn get_rarity(&self) -> RarityTier {
		DnaUtils::read_attribute(&self.inner, AvatarAttr::RarityTier)
	}

	pub fn set_rarity(&mut self, rarity: RarityTier) {
		DnaUtils::write_attribute::<RarityTier>(&mut self.inner, AvatarAttr::RarityTier, &rarity)
	}

	pub fn same_rarity(&self, other: &WrappedAvatar) -> bool {
		self.get_rarity().cmp(&other.get_rarity()).is_eq()
	}

	pub fn get_quantity(&self) -> u8 {
		DnaUtils::read_attribute_raw(&self.inner, AvatarAttr::Quantity)
	}

	pub fn set_quantity(&mut self, quantity: u8) {
		DnaUtils::write_attribute_raw(&mut self.inner, AvatarAttr::Quantity, quantity)
	}

	pub fn same_quantity(&self, other: &WrappedAvatar) -> bool {
		self.get_quantity().cmp(&other.get_quantity()).is_eq()
	}

	pub fn get_spec<T>(&self, spec_index: SpecIdx) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_spec::<T>(&self.inner, spec_index)
	}

	pub fn set_spec(&mut self, spec_index: SpecIdx, value: u8) {
		DnaUtils::write_spec(&mut self.inner, spec_index, value);
	}

	pub fn get_specs(&self) -> [u8; 16] {
		DnaUtils::read_specs(&self.inner)
	}

	pub fn set_specs(&mut self, spec_bytes: [u8; 16]) {
		DnaUtils::write_specs(&mut self.inner, spec_bytes)
	}

	pub fn same_specs(&self, other: &WrappedAvatar) -> bool {
		self.get_specs() == other.get_specs()
	}

	pub fn same_spec_at(&self, other: &WrappedAvatar, position: SpecIdx) -> bool {
		DnaUtils::read_spec_raw(&self.inner, position) ==
			DnaUtils::read_spec_raw(&other.inner, position)
	}

	pub fn get_progress(&self) -> [u8; 11] {
		DnaUtils::read_progress(&self.inner)
	}

	pub fn set_progress(&mut self, progress_array: [u8; 11]) {
		DnaUtils::write_progress(&mut self.inner, progress_array)
	}

	pub fn same_progress(&self, other: &WrappedAvatar) -> bool {
		self.get_progress() == other.get_progress()
	}

	pub fn has_type(&self, item_type: ItemType) -> bool {
		self.get_item_type() == item_type
	}

	pub fn has_subtype<T>(&self, item_sub_type: T) -> bool
	where
		T: ByteConvertible + Eq,
	{
		self.get_item_sub_type::<T>() == item_sub_type
	}

	pub fn has_full_type<T>(&self, item_type: ItemType, item_sub_type: T) -> bool
	where
		T: ByteConvertible + Eq,
	{
		self.has_type(item_type) && self.has_subtype(item_sub_type)
	}

	pub fn has_zeroed_class_types(&self) -> bool {
		self.get_class_type_1::<u8>() == 0 && self.get_class_type_2::<u8>() == 0
	}

	pub fn same_full_type(&self, other: &WrappedAvatar) -> bool {
		self.same_item_type(other) && self.same_item_sub_type(other)
	}

	pub fn same_full_and_class_types(&self, other: &WrappedAvatar) -> bool {
		self.same_full_type(other) && self.same_class_type_1(other) && self.same_class_type_2(other)
	}

	pub fn same_assemble_version(&self, other: &WrappedAvatar) -> bool {
		self.same_item_type(other) && self.same_class_type_1(other) && self.same_class_type_2(other)
	}
}

#[derive(Copy, Clone)]
pub enum AvatarAttr {
	ItemType,
	ItemSubType,
	ClassType1,
	ClassType2,
	CustomType1,
	CustomType2,
	RarityTier,
	Quantity,
}

#[derive(Copy, Clone)]
pub enum SpecIdx {
	Byte1,
	Byte2,
	Byte3,
	Byte4,
	Byte5,
	Byte6,
	Byte7,
	Byte8,
	#[allow(dead_code)]
	Byte9,
	#[allow(dead_code)]
	Byte10,
	#[allow(dead_code)]
	Byte11,
	#[allow(dead_code)]
	Byte12,
	#[allow(dead_code)]
	Byte13,
	#[allow(dead_code)]
	Byte14,
	#[allow(dead_code)]
	Byte15,
	#[allow(dead_code)]
	Byte16,
}

#[derive(Default)]
pub(crate) struct AvatarBuilder {
	inner: Avatar,
}

impl AvatarBuilder {
	pub fn with_dna(season_id: SeasonId, dna: Dna) -> Self {
		Self { inner: Avatar { season_id, encoding: DnaEncoding::V2, dna, souls: 0 } }
	}

	pub fn with_base_avatar(avatar: Avatar) -> Self {
		Self { inner: avatar }
	}

	pub fn with_attribute<T>(self, attribute: AvatarAttr, value: &T) -> Self
	where
		T: ByteConvertible,
	{
		self.with_attribute_raw(attribute, value.as_byte())
	}

	pub fn with_attribute_raw(mut self, attribute: AvatarAttr, value: u8) -> Self {
		DnaUtils::write_attribute_raw(&mut self.inner, attribute, value);
		self
	}

	pub fn with_spec_byte_raw(mut self, spec_byte: SpecIdx, value: u8) -> Self {
		DnaUtils::write_spec(&mut self.inner, spec_byte, value);
		self
	}

	pub fn with_spec_bytes(mut self, spec_bytes: [u8; 16]) -> Self {
		DnaUtils::write_specs(&mut self.inner, spec_bytes);
		self
	}

	pub fn with_soul_count(mut self, soul_count: SoulCount) -> Self {
		self.inner.souls = soul_count;
		self
	}

	pub fn with_progress_array(mut self, progress_array: [u8; 11]) -> Self {
		DnaUtils::write_progress(&mut self.inner, progress_array);
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
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::Pet)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Legendary)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_attribute_raw(AvatarAttr::CustomType2, pet_variation)
			.with_spec_bytes(spec_bytes)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_generic_pet_part(self, slot_types: &[SlotType], quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		let spec_bytes = {
			let mut spec_bytes = DnaUtils::read_specs(&self.inner);

			for slot_index in slot_types.iter().map(|slot_type| slot_type.as_byte() as usize) {
				spec_bytes[slot_index] = spec_bytes[slot_index].saturating_add(1);
			}

			spec_bytes
		};

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::PetPart)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &custom_type_1)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_bytes(spec_bytes)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	#[cfg(test)]
	pub fn into_pet_part(self, pet_type: &PetType, slot_type: &SlotType, quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
		let base_0 = DnaUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorBase.as_byte() as usize,
		);
		let comp_1 = DnaUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent1.as_byte() as usize,
		);
		let comp_2 = DnaUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent2.as_byte() as usize,
		);
		let comp_3 = DnaUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent3.as_byte() as usize,
		);

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::PetPart)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &custom_type_1)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(SpecIdx::Byte1, DnaUtils::enums_to_bits(&base_0) as u8)
			.with_spec_byte_raw(SpecIdx::Byte2, DnaUtils::enums_order_to_bits(&base_0) as u8)
			.with_spec_byte_raw(SpecIdx::Byte3, DnaUtils::enums_to_bits(&comp_1) as u8)
			.with_spec_byte_raw(SpecIdx::Byte4, DnaUtils::enums_order_to_bits(&comp_1) as u8)
			.with_spec_byte_raw(SpecIdx::Byte5, DnaUtils::enums_to_bits(&comp_2) as u8)
			.with_spec_byte_raw(SpecIdx::Byte6, DnaUtils::enums_order_to_bits(&comp_2) as u8)
			.with_spec_byte_raw(SpecIdx::Byte7, DnaUtils::enums_to_bits(&comp_3) as u8)
			.with_spec_byte_raw(SpecIdx::Byte8, DnaUtils::enums_order_to_bits(&comp_3) as u8)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_egg(
		self,
		rarity: &RarityTier,
		pet_variation: u8,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::Egg)
			// Unused
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, rarity)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_attribute_raw(AvatarAttr::CustomType2, pet_variation)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_material(self, material_type: &MaterialItemType, quantity: u8) -> Self {
		let sp_ratio = match *material_type {
			MaterialItemType::Ceramics | MaterialItemType::Electronics => HexType::X1,
			MaterialItemType::PowerCells | MaterialItemType::Polymers => HexType::X2,
			MaterialItemType::Superconductors | MaterialItemType::Metals => HexType::X3,
			MaterialItemType::Optics | MaterialItemType::Nanomaterials => HexType::X4,
		};

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Material)
			.with_attribute(AvatarAttr::ItemSubType, material_type)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &sp_ratio)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Common)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_soul_count(quantity as SoulCount * sp_ratio as SoulCount)
	}

	pub fn into_glimmer(self, quantity: u8) -> Self {
		let custom_type_1 = HexType::from_byte(GLIMMER_SP);

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::Glimmer)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &custom_type_1)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_color_spark(
		self,
		color_pair: &(ColorType, ColorType),
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::ColorSpark)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, color_pair.0.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte2, color_pair.1.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_glow_spark(
		self,
		force: &Force,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::GlowSpark)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, force.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte2, 0)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
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

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::PaintFlask)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, color_bytes)
			.with_spec_byte_raw(SpecIdx::Byte2, 0b0000_1000)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_glow_flask(
		self,
		force: &Force,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::GlowFlask)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, force.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte2, 0)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
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
		if equippable_type.is_empty() || equippable_type.iter().any(|equip| !equip.is_armor()) {
			return Err(())
		}

		let (armor_assemble_progress, color_flag) = {
			let mut color_flag = 0b0000_0000;
			let mut progress = DnaUtils::enums_to_bits(equippable_type) as u8;

			if color_pair.0 != ColorType::Null && color_pair.1 != ColorType::Null {
				color_flag = 0b0000_1000;
				progress |= ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
					((color_pair.1.as_byte().saturating_sub(1)) << 4)
			}

			(progress, color_flag)
		};

		// Guaranteed to work because of check above
		let first_equippable = equippable_type.first().unwrap();

		let progress_array = DnaUtils::generate_progress(
			rarity,
			SCALING_FACTOR_PERC,
			Some(PROGRESS_PROBABILITY_PERC),
			hash_provider,
		);

		Ok(self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Equippable)
			.with_attribute(AvatarAttr::ItemSubType, first_equippable)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, rarity)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(SpecIdx::Byte1, armor_assemble_progress)
			.with_spec_byte_raw(SpecIdx::Byte2, force.as_byte() | color_flag)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
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
		if !equippable_type.is_weapon() {
			return Err(())
		}

		let (weapon_info, color_flag) = {
			let mut color_flag = 0b0000_0000;
			let mut info = DnaUtils::enums_to_bits(&[*equippable_type]) as u8 >> 4;

			if color_pair.0 != ColorType::Null && color_pair.1 != ColorType::Null {
				color_flag = 0b0000_1000;
				info |= ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
					((color_pair.1.as_byte().saturating_sub(1)) << 4)
			}

			(info, color_flag)
		};

		let rarity = RarityTier::Legendary;

		let progress_array =
			DnaUtils::generate_progress(&rarity, SCALING_FACTOR_PERC, None, hash_provider);

		Ok(self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Equippable)
			.with_attribute(AvatarAttr::ItemSubType, equippable_type)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &rarity)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(SpecIdx::Byte1, weapon_info)
			.with_spec_byte_raw(SpecIdx::Byte2, force.as_byte() | color_flag)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
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
		quantity: u8,
	) -> Self {
		// TODO: add a quantity algorithm
		// - base 8 - 16 and
		// - components 6 - 12
		let mat_req1 = 1;
		let mat_req2 = 1;
		let mat_req3 = 1;
		let mat_req4 = 1;

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Blueprint)
			.with_attribute(AvatarAttr::ItemSubType, blueprint_type)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X1)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(SpecIdx::Byte1, DnaUtils::enums_to_bits(pattern) as u8)
			.with_spec_byte_raw(SpecIdx::Byte2, DnaUtils::enums_order_to_bits(pattern) as u8)
			.with_spec_byte_raw(SpecIdx::Byte3, equippable_item_type.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte4, mat_req1)
			.with_spec_byte_raw(SpecIdx::Byte5, mat_req2)
			.with_spec_byte_raw(SpecIdx::Byte6, mat_req3)
			.with_spec_byte_raw(SpecIdx::Byte7, mat_req4)
			.with_soul_count(quantity as SoulCount)
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

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Special)
			.with_attribute(AvatarAttr::ItemSubType, &SpecialItemType::Unidentified)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Legendary)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, git_info)
			.with_spec_byte_raw(SpecIdx::Byte2, force.as_byte())
			.with_soul_count(soul_points)
	}

	pub fn into_dust(self, soul_points: SoulCount) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Special)
			.with_attribute(AvatarAttr::ItemSubType, &SpecialItemType::Dust)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X1)
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Common)
			.with_attribute_raw(AvatarAttr::Quantity, soul_points as u8)
			.with_soul_count(soul_points)
	}

	pub fn into_toolbox(self, soul_points: SoulCount) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Special)
			.with_attribute(AvatarAttr::ItemSubType, &SpecialItemType::ToolBox)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_progress_array([0xBB; 11])
			.with_soul_count(soul_points)
	}

	pub fn build(self) -> Avatar {
		self.inner
	}

	pub fn build_wrapped(self) -> WrappedAvatar {
		WrappedAvatar::new(self.inner)
	}
}

/// Struct to wrap DNA interactions with Avatars from V2 upwards.
/// Don't use with Avatars with V1.
pub(crate) struct DnaUtils;

impl DnaUtils {
	fn read_strand(avatar: &Avatar, position: usize, byte_type: ByteType) -> u8 {
		Self::read_at(avatar.dna.as_slice(), position, byte_type)
	}

	fn write_strand(avatar: &mut Avatar, position: usize, byte_type: ByteType, value: u8) {
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

	fn read_at(dna: &[u8], position: usize, byte_type: ByteType) -> u8 {
		match byte_type {
			ByteType::Full => dna[position],
			ByteType::High => Self::high_nibble_of(dna[position]),
			ByteType::Low => Self::low_nibble_of(dna[position]),
		}
	}

	fn write_at(dna: &mut [u8], position: usize, byte_type: ByteType, value: u8) {
		match byte_type {
			ByteType::Full => dna[position] = value,
			ByteType::High =>
				dna[position] = (dna[position] & (ByteType::High as u8)) | (value << 4),
			ByteType::Low =>
				dna[position] =
					(dna[position] & (ByteType::Low as u8)) | (value & (ByteType::High as u8)),
		}
	}

	pub fn high_nibble_of(byte: u8) -> u8 {
		byte >> 4
	}

	pub fn low_nibble_of(byte: u8) -> u8 {
		byte & 0x0F
	}

	pub fn read_attribute<T>(avatar: &Avatar, attribute: AvatarAttr) -> T
	where
		T: ByteConvertible,
	{
		T::from_byte(Self::read_attribute_raw(avatar, attribute))
	}

	pub fn read_attribute_raw(avatar: &Avatar, attribute: AvatarAttr) -> u8 {
		match attribute {
			AvatarAttr::ItemType => Self::read_strand(avatar, 0, ByteType::High),
			AvatarAttr::ItemSubType => Self::read_strand(avatar, 0, ByteType::Low),
			AvatarAttr::ClassType1 => Self::read_strand(avatar, 1, ByteType::High),
			AvatarAttr::ClassType2 => Self::read_strand(avatar, 1, ByteType::Low),
			AvatarAttr::CustomType1 => Self::read_strand(avatar, 2, ByteType::High),
			AvatarAttr::CustomType2 => Self::read_strand(avatar, 4, ByteType::Full),
			AvatarAttr::RarityTier => Self::read_strand(avatar, 2, ByteType::Low),
			AvatarAttr::Quantity => Self::read_strand(avatar, 3, ByteType::Full),
		}
	}

	pub fn write_attribute<T>(avatar: &mut Avatar, attribute: AvatarAttr, value: &T)
	where
		T: ByteConvertible,
	{
		Self::write_attribute_raw(avatar, attribute, value.as_byte())
	}

	pub fn write_attribute_raw(avatar: &mut Avatar, attribute: AvatarAttr, value: u8) {
		match attribute {
			AvatarAttr::ItemType => Self::write_strand(avatar, 0, ByteType::High, value),
			AvatarAttr::ItemSubType => Self::write_strand(avatar, 0, ByteType::Low, value),
			AvatarAttr::ClassType1 => Self::write_strand(avatar, 1, ByteType::High, value),
			AvatarAttr::ClassType2 => Self::write_strand(avatar, 1, ByteType::Low, value),
			AvatarAttr::CustomType1 => Self::write_strand(avatar, 2, ByteType::High, value),
			AvatarAttr::CustomType2 => Self::write_strand(avatar, 4, ByteType::Full, value),
			AvatarAttr::RarityTier => Self::write_strand(avatar, 2, ByteType::Low, value),
			AvatarAttr::Quantity => Self::write_strand(avatar, 3, ByteType::Full, value),
		}
	}

	pub fn read_specs(avatar: &Avatar) -> [u8; 16] {
		let mut out = [0; 16];
		out.copy_from_slice(&avatar.dna[5..21]);
		out
	}

	pub fn read_spec_raw(avatar: &Avatar, index: SpecIdx) -> u8 {
		match index {
			SpecIdx::Byte1 => Self::read_strand(avatar, 5, ByteType::Full),
			SpecIdx::Byte2 => Self::read_strand(avatar, 6, ByteType::Full),
			SpecIdx::Byte3 => Self::read_strand(avatar, 7, ByteType::Full),
			SpecIdx::Byte4 => Self::read_strand(avatar, 8, ByteType::Full),
			SpecIdx::Byte5 => Self::read_strand(avatar, 9, ByteType::Full),
			SpecIdx::Byte6 => Self::read_strand(avatar, 10, ByteType::Full),
			SpecIdx::Byte7 => Self::read_strand(avatar, 11, ByteType::Full),
			SpecIdx::Byte8 => Self::read_strand(avatar, 12, ByteType::Full),
			SpecIdx::Byte9 => Self::read_strand(avatar, 13, ByteType::Full),
			SpecIdx::Byte10 => Self::read_strand(avatar, 14, ByteType::Full),
			SpecIdx::Byte11 => Self::read_strand(avatar, 15, ByteType::Full),
			SpecIdx::Byte12 => Self::read_strand(avatar, 16, ByteType::Full),
			SpecIdx::Byte13 => Self::read_strand(avatar, 17, ByteType::Full),
			SpecIdx::Byte14 => Self::read_strand(avatar, 18, ByteType::Full),
			SpecIdx::Byte15 => Self::read_strand(avatar, 19, ByteType::Full),
			SpecIdx::Byte16 => Self::read_strand(avatar, 20, ByteType::Full),
		}
	}

	pub fn read_spec<T>(avatar: &Avatar, spec_byte: SpecIdx) -> T
	where
		T: ByteConvertible,
	{
		T::from_byte(Self::read_spec_raw(avatar, spec_byte))
	}

	pub fn write_specs(avatar: &mut Avatar, value: [u8; 16]) {
		(avatar.dna[5..21]).copy_from_slice(&value);
	}

	pub fn write_spec(avatar: &mut Avatar, spec_byte: SpecIdx, value: u8) {
		match spec_byte {
			SpecIdx::Byte1 => Self::write_strand(avatar, 5, ByteType::Full, value),
			SpecIdx::Byte2 => Self::write_strand(avatar, 6, ByteType::Full, value),
			SpecIdx::Byte3 => Self::write_strand(avatar, 7, ByteType::Full, value),
			SpecIdx::Byte4 => Self::write_strand(avatar, 8, ByteType::Full, value),
			SpecIdx::Byte5 => Self::write_strand(avatar, 9, ByteType::Full, value),
			SpecIdx::Byte6 => Self::write_strand(avatar, 10, ByteType::Full, value),
			SpecIdx::Byte7 => Self::write_strand(avatar, 11, ByteType::Full, value),
			SpecIdx::Byte8 => Self::write_strand(avatar, 12, ByteType::Full, value),
			SpecIdx::Byte9 => Self::write_strand(avatar, 13, ByteType::Full, value),
			SpecIdx::Byte10 => Self::write_strand(avatar, 14, ByteType::Full, value),
			SpecIdx::Byte11 => Self::write_strand(avatar, 15, ByteType::Full, value),
			SpecIdx::Byte12 => Self::write_strand(avatar, 16, ByteType::Full, value),
			SpecIdx::Byte13 => Self::write_strand(avatar, 17, ByteType::Full, value),
			SpecIdx::Byte14 => Self::write_strand(avatar, 18, ByteType::Full, value),
			SpecIdx::Byte15 => Self::write_strand(avatar, 19, ByteType::Full, value),
			SpecIdx::Byte16 => Self::write_strand(avatar, 20, ByteType::Full, value),
		}
	}

	pub fn read_progress(avatar: &Avatar) -> [u8; 11] {
		let mut out = [0; 11];
		out.copy_from_slice(&avatar.dna[21..32]);
		out
	}

	pub fn write_progress(avatar: &mut Avatar, value: [u8; 11]) {
		(avatar.dna[21..32]).copy_from_slice(&value);
	}

	pub fn is_progress_match(
		array_1: [u8; 11],
		array_2: [u8; 11],
		rarity_level: u8,
	) -> Option<Vec<u32>> {
		let (mirror, matches) = Self::match_progress(array_1, array_2, rarity_level);
		let match_count = matches.len() as u32;
		let mirror_count = mirror.len() as u32;

		(match_count > 0 && (((match_count * 2) + mirror_count) >= 6)).then_some(matches)
	}

	pub fn match_progress(
		array_1: [u8; 11],
		array_2: [u8; 11],
		rarity_level: u8,
	) -> (Vec<u32>, Vec<u32>) {
		let mut matches = Vec::<u32>::new();
		let mut mirrors = Vec::<u32>::new();

		let lowest_1 = Self::lowest_progress_byte(&array_1, ByteType::High);
		let lowest_2 = Self::lowest_progress_byte(&array_2, ByteType::High);

		if lowest_1 > lowest_2 {
			return (mirrors, matches)
		}

		for i in 0..array_1.len() {
			let rarity_1 = Self::read_at(&array_1, i, ByteType::High);
			let variation_1 = Self::read_at(&array_1, i, ByteType::Low);

			let rarity_2 = Self::read_at(&array_2, i, ByteType::High);
			let variation_2 = Self::read_at(&array_2, i, ByteType::Low);

			let have_same_rarity = rarity_1 == rarity_2 || rarity_2 == 0x0B;
			let is_maxed = rarity_1 > lowest_1 || lowest_1 == RarityTier::Legendary.as_byte();
			let byte_match = Self::match_progress_byte(variation_1, variation_2);

			if have_same_rarity &&
				!is_maxed && (rarity_1 < rarity_level || variation_2 == 0x0B || byte_match)
			{
				matches.push(i as u32);
			} else if is_maxed && ((variation_1 == variation_2) || variation_2 == 0x0B) {
				mirrors.push(i as u32);
			}
		}

		(mirrors, matches)
	}

	fn match_progress_byte(byte_1: u8, byte_2: u8) -> bool {
		let diff = if byte_1 >= byte_2 { byte_1 - byte_2 } else { byte_2 - byte_1 };
		diff == 1 || diff == (PROGRESS_VARIATIONS - 1)
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

	pub fn generate_progress<T: Config>(
		rarity: &RarityTier,
		scale_factor: u32,
		probability: Option<u32>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> [u8; 11] {
		let mut progress_bytes = [0; 11];

		let prob_value = probability.unwrap_or_default();

		for i in 0..progress_bytes.len() {
			let random_value = hash_provider.next();

			// Upcast random_value
			let new_rarity =
				if (random_value as u32).saturating_mul(scale_factor) < (prob_value * MAX_BYTE) {
					rarity.upgrade().as_byte()
				} else {
					rarity.as_byte()
				};

			Self::write_at(&mut progress_bytes, i, ByteType::High, new_rarity);
			Self::write_at(
				&mut progress_bytes,
				i,
				ByteType::Low,
				random_value % PROGRESS_VARIATIONS,
			);
		}

		Self::write_at(&mut progress_bytes, 10, ByteType::High, rarity.as_byte());

		progress_bytes
	}

	pub fn lowest_progress_byte(progress_bytes: &[u8; 11], byte_type: ByteType) -> u8 {
		let mut result = u8::MAX;

		for i in 0..progress_bytes.len() {
			let value = Self::read_at(progress_bytes, i, byte_type);
			if result > value {
				result = value;
			}
		}

		result
	}

	#[cfg(test)]
	pub fn lowest_progress_indexes(progress_bytes: &[u8; 11], byte_type: ByteType) -> Vec<usize> {
		let mut lowest = u8::MAX;

		let mut result = Vec::new();

		for i in 0..progress_bytes.len() {
			let value = Self::read_at(progress_bytes, i, byte_type);

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

	pub fn indexes_of_max(byte_array: &[u8]) -> Vec<usize> {
		let mut max_value = 0;
		let mut max_indexes = Vec::new();

		for (i, byte) in byte_array.iter().enumerate() {
			match byte.cmp(&max_value) {
				Ordering::Greater => {
					max_value = *byte;
					max_indexes.clear();
					max_indexes.push(i);
				},
				Ordering::Equal => {
					max_indexes.push(i);
				},
				Ordering::Less => continue,
			}
		}

		max_indexes
	}

	pub fn current_period<T: Config>(
		current_phase: u32,
		total_phases: u32,
		block_number: T::BlockNumber,
	) -> u32 {
		block_number
			.div(current_phase.into())
			.rem(total_phases.into())
			.saturated_into::<u32>()
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

	pub fn full_hash(&self, mutate_seed: usize) -> T::Hash {
		let mut full_hash = self.hash;

		for (i, hash) in full_hash.iter_mut().enumerate() {
			*hash = self.hash[(i + mutate_seed) % N];
		}

		T::Hashing::hash(&full_hash)
	}

	pub fn next(&mut self) -> u8 {
		<Self as Iterator>::next(self).unwrap_or_default()
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
	use crate::mock::*;
	use hex;

	#[test]
	fn test_bits_to_enums_consistency_1() {
		let bits = 0b_01_01_01_01;

		let result = DnaUtils::bits_to_enums::<NibbleType>(bits);
		let expected = vec![NibbleType::X0, NibbleType::X2, NibbleType::X4, NibbleType::X6];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_bits_to_enums_consistency_2() {
		let bits = 0b_11_01_10_01;

		let result = DnaUtils::bits_to_enums::<MaterialItemType>(bits);
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

		let result = DnaUtils::bits_order_to_enum(bit_order, 4, enum_list);
		let expected = vec![NibbleType::X2, NibbleType::X4, NibbleType::X6, NibbleType::X0];
		assert_eq!(result, expected);

		let bit_order_2 = 0b_01_11_00_10;
		let enum_list_2 = vec![NibbleType::X4, NibbleType::X5, NibbleType::X6, NibbleType::X7];

		let result_2 = DnaUtils::bits_order_to_enum(bit_order_2, 4, enum_list_2);
		let expected_2 = vec![NibbleType::X5, NibbleType::X7, NibbleType::X4, NibbleType::X6];
		assert_eq!(result_2, expected_2);
	}

	#[test]
	fn test_bits_order_to_enums_consistency_2() {
		let bit_order = 0b_01_10_00_10;
		let enum_list = vec![PetType::FoxishDude, PetType::FireDino, PetType::GiantWoodStick];

		let result = DnaUtils::bits_order_to_enum(bit_order, 4, enum_list);
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

		assert_eq!(DnaUtils::enums_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_to_bits_consistency_2() {
		let pattern = vec![PetType::FoxishDude, PetType::BigHybrid, PetType::GiantWoodStick];
		let expected = 0b_00_11_00_10;

		assert_eq!(DnaUtils::enums_to_bits(&pattern), expected);
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

		assert_eq!(DnaUtils::enums_order_to_bits(&pattern), expected);
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

		let bits = DnaUtils::enums_to_bits(&pattern);
		assert_eq!(bits, 0b_01_10_00_01);

		let enums = DnaUtils::bits_to_enums::<MaterialItemType>(bits);
		assert_eq!(enums, expected);
	}

	#[test]
	fn test_create_pattern_consistency() {
		let base_seed = SlotType::Head.as_byte() as usize;
		let pattern =
			DnaUtils::create_pattern::<NibbleType>(base_seed, SlotType::Breast.as_byte() as usize);

		let expected = vec![NibbleType::X7, NibbleType::X5, NibbleType::X4, NibbleType::X3];

		assert_eq!(pattern, expected);
	}

	#[test]
	fn tests_pattern_and_order() {
		let base_seed = (PetType::FoxishDude.as_byte() + SlotType::Head.as_byte()) as usize;

		let pattern_1 = DnaUtils::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorBase.as_byte() as usize,
		);
		let p10 = DnaUtils::enums_to_bits(&pattern_1);
		let p11 = DnaUtils::enums_order_to_bits(&pattern_1);

		assert_eq!(p10, 0b_01_10_11_00);
		assert_eq!(p11, 0b_01_11_10_00);

		// Decode Blueprint
		let unordered_1 = DnaUtils::bits_to_enums::<NibbleType>(p10);
		let pattern_1_check = DnaUtils::bits_order_to_enum(p11, 4, unordered_1);
		assert_eq!(pattern_1_check, pattern_1);

		// Pattern number and enum number only match if they are according to the index in the list
		let unordered_material = DnaUtils::bits_to_enums::<MaterialItemType>(p10);
		assert_eq!(
			DnaUtils::bits_order_to_enum(p11, 4, unordered_material)[0],
			MaterialItemType::Optics
		);

		let test_set: Vec<(EquippableItemType, u32, u32)> = vec![
			(EquippableItemType::ArmorComponent1, 0b_11_01_10_00, 0b_01_11_00_10),
			(EquippableItemType::ArmorComponent2, 0b_01_01_01_01, 0b_01_11_10_00),
			(EquippableItemType::ArmorComponent3, 0b_01_10_01_10, 0b_01_10_11_00),
		];

		for (armor_component, enum_to_bits, enum_order_to_bits) in test_set {
			let pattern_base = DnaUtils::create_pattern::<NibbleType>(
				base_seed,
				armor_component.as_byte() as usize,
			);
			let p_enum_to_bits = DnaUtils::enums_to_bits(&pattern_base);
			let p_enum_order_to_bits = DnaUtils::enums_order_to_bits(&pattern_base);
			assert_eq!(p_enum_to_bits, enum_to_bits);
			assert_eq!(p_enum_order_to_bits, enum_order_to_bits);
			// Decode Blueprint
			let unordered_base = DnaUtils::bits_to_enums::<NibbleType>(p_enum_to_bits);
			let pattern_base_check =
				DnaUtils::bits_order_to_enum(p_enum_order_to_bits, 4, unordered_base);
			assert_eq!(pattern_base_check, pattern_base);
		}
	}

	#[test]
	fn test_match_progress_array_consistency() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1 = [0x00; 11];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10; 11];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x00; 11];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10; 11];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00];
		let arr_2 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10];
		let arr_2 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x00];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x00, 0x11, 0x02, 0x13, 0x04, 0x15, 0x04, 0x13, 0x02, 0x11, 0x00];
		let arr_2 = [0x01, 0x01, 0x12, 0x13, 0x04, 0x04, 0x13, 0x12, 0x01, 0x01, 0x15];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 8];
		let expected_mirrors: Vec<u32> = vec![1, 3, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);
	}

	#[test]
	fn test_match_progress_array_consistency_multiple() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x12, 0x13, 0x14, 0x13, 0x14, 0x11, 0x22, 0x10, 0x14, 0x22, 0x11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 7];
		let expected_mirrors: Vec<u32> = vec![5];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x10, 0x10, 0x10, 0x14, 0x14, 0x13, 0x13, 0x12, 0x15, 0x14, 0x14];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x15, 0x10, 0x14, 0x13, 0x13, 0x11, 0x10, 0x14, 0x12, 0x20, 0x11];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 7, 8];
		let expected_mirrors: Vec<u32> = vec![5];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x11, 0x11, 0x11, 0x10, 0x15, 0x12, 0x11, 0x11, 0x13, 0x12, 0x14];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 2, 3, 6, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_match_progress_consistency_on_level() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x44, 0x44, 0x42, 0x45];
		let arr_2 = [0x41, 0x51, 0x52, 0x53, 0x44, 0x52, 0x45, 0x41, 0x40, 0x41, 0x43];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 4, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x44, 0x44, 0x42, 0x45];
		let arr_2 = [0x52, 0x41, 0x43, 0x41, 0x53, 0x45, 0x43, 0x44, 0x52, 0x43, 0x43];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x54, 0x44, 0x42, 0x53];
		let arr_2 = [0x52, 0x40, 0x43, 0x41, 0x53, 0x45, 0x41, 0x44, 0x52, 0x43, 0x43];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![9];
		let expected_mirrors: Vec<u32> = vec![7, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x30, 0x30, 0x30, 0x30];
		let arr_2 = [0x45, 0x45, 0x45, 0x45, 0x45, 0x35, 0x45, 0x31, 0x30, 0x45, 0x45];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![7];
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x31, 0x30, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34];
		let arr_2 = [0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x22, 0x22, 0x22];
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_match_progress_consistency_hex() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1: [u8; 11] =
			hex::decode("3130353330333132323234").expect("Decode").try_into().unwrap();
		let arr_2: [u8; 11] =
			hex::decode("2121353424332322222222").expect("Decode").try_into().unwrap();
		let (mirrors, matches) = DnaUtils::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_indexes_of_max() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(DnaUtils::indexes_of_max(&[0, 2, 1, 1]), vec![1]);
			assert_eq!(DnaUtils::indexes_of_max(&[9, 5, 3, 9, 7, 2, 1]), vec![0, 3]);
			assert_eq!(DnaUtils::indexes_of_max(&[0, 0, 0, 0, 0]), vec![0, 1, 2, 3, 4]);
			assert_eq!(DnaUtils::indexes_of_max(&[1, 4, 9, 2, 3, 11, 10, 11, 0, 1]), vec![5, 7]);
		});
	}

	#[test]
	fn test_current_period() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 0), 0);
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 9), 0);
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 10), 1);
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 19), 1);
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 130), 13);
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 139), 13);
			assert_eq!(DnaUtils::current_period::<Test>(10, 14, 140), 0);
		});
	}
}
