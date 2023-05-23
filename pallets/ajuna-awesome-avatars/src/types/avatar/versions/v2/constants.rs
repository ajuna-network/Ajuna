use super::*;

/*
	   public const bool Obfuscate = false;
	   public const byte ProgressComponents = 11;
*/

pub(crate) const MIN_SACRIFICE: usize = 1;
pub(crate) const MAX_SACRIFICE: usize = 4;
pub(crate) const MAX_QUANTITY: u8 = 8;
pub(crate) const SCALING_FACTOR_PERC: u32 = 100;
pub(crate) const MAX_EQUIPPED_SLOTS: usize = 5;
pub(crate) const PROGRESS_VARIATIONS: u8 = 6;
pub(crate) const STACK_PROB_PERC: u32 = 5;
pub(crate) const PROGRESS_PROBABILITY_PERC: u32 = 15;
pub(crate) const BASE_PROGRESS_PROB_PERC: u32 = 20;
pub(crate) const COLOR_GLOW_SPARK: u32 = 75;
pub(crate) const SPARK_PROGRESS_PROB_PERC: u32 = 75;
pub(crate) const GLIMMER_FORGE_GLIMMER_USE: u8 = 1;
pub(crate) const GLIMMER_FORGE_MATERIAL_USE: u8 = 4;
pub(crate) const MAX_BYTE: u32 = u8::MAX as u32;

/// Probabilities for all PackType::Material options
pub(crate) const PACK_TYPE_MATERIAL_ITEM_PROBABILITIES: ProbabilitySlots<ItemType, 6> = [
	(ItemType::Pet, 150),
	(ItemType::Material, 700),
	(ItemType::Essence, 50),
	(ItemType::Equippable, 50),
	(ItemType::Blueprint, 0),
	(ItemType::Special, 50),
];

pub(crate) const PACK_TYPE_MATERIAL_PET_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<PetItemType, 3> =
	[(PetItemType::Pet, 0), (PetItemType::PetPart, 980), (PetItemType::Egg, 20)];

pub(crate) const PACK_TYPE_MATERIAL_MATERIAL_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	MaterialItemType,
	8,
> = [
	(MaterialItemType::Polymers, 125),
	(MaterialItemType::Electronics, 125),
	(MaterialItemType::PowerCells, 125),
	(MaterialItemType::Optics, 125),
	(MaterialItemType::Metals, 125),
	(MaterialItemType::Ceramics, 125),
	(MaterialItemType::Superconductors, 125),
	(MaterialItemType::Nanomaterials, 125),
];

pub(crate) const PACK_TYPE_MATERIAL_ESSENCE_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	EssenceItemType,
	5,
> = [
	(EssenceItemType::Glimmer, 800),
	(EssenceItemType::ColorSpark, 50),
	(EssenceItemType::GlowSpark, 50),
	(EssenceItemType::PaintFlask, 10),
	(EssenceItemType::ForceGlow, 10),
];

pub(crate) const PACK_TYPE_MATERIAL_EQUIPABLE_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	EquippableItemType,
	7,
> = [
	(EquippableItemType::ArmorBase, 970),
	(EquippableItemType::ArmorComponent1, 0),
	(EquippableItemType::ArmorComponent2, 0),
	(EquippableItemType::ArmorComponent3, 0),
	(EquippableItemType::WeaponVersion1, 10),
	(EquippableItemType::WeaponVersion2, 10),
	(EquippableItemType::WeaponVersion3, 10),
];

pub(crate) const PACK_TYPE_MATERIAL_BLUEPRINT_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	BlueprintItemType,
	1,
> = [(BlueprintItemType::Blueprint, 1000)];

pub(crate) const PACK_TYPE_MATERIAL_SPECIAL_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	SpecialItemType,
	2,
> = [(SpecialItemType::Dust, 990), (SpecialItemType::Unidentified, 10)];
// -----------------------------------------------

/// Probabilities for all PackType::Equipment options
pub(crate) const PACK_TYPE_EQUIPMENT_ITEM_PROBABILITIES: ProbabilitySlots<ItemType, 6> = [
	(ItemType::Pet, 100),
	(ItemType::Material, 350),
	(ItemType::Essence, 300),
	(ItemType::Equippable, 50),
	(ItemType::Blueprint, 0),
	(ItemType::Special, 200),
];

pub(crate) const PACK_TYPE_EQUIPMENT_PET_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<PetItemType, 3> =
	[(PetItemType::Pet, 20), (PetItemType::PetPart, 800), (PetItemType::Egg, 180)];

pub(crate) const PACK_TYPE_EQUIPMENT_MATERIAL_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	MaterialItemType,
	8,
> = [
	(MaterialItemType::Polymers, 125),
	(MaterialItemType::Electronics, 125),
	(MaterialItemType::PowerCells, 125),
	(MaterialItemType::Optics, 125),
	(MaterialItemType::Metals, 125),
	(MaterialItemType::Ceramics, 125),
	(MaterialItemType::Superconductors, 125),
	(MaterialItemType::Nanomaterials, 125),
];

pub(crate) const PACK_TYPE_EQUIPMENT_ESSENCE_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	EssenceItemType,
	5,
> = [
	(EssenceItemType::Glimmer, 550),
	(EssenceItemType::ColorSpark, 175),
	(EssenceItemType::GlowSpark, 175),
	(EssenceItemType::PaintFlask, 50),
	(EssenceItemType::ForceGlow, 50),
];

pub(crate) const PACK_TYPE_EQUIPMENT_EQUIPABLE_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	EquippableItemType,
	7,
> = [
	(EquippableItemType::ArmorBase, 10),
	(EquippableItemType::ArmorComponent1, 250),
	(EquippableItemType::ArmorComponent2, 250),
	(EquippableItemType::ArmorComponent3, 250),
	(EquippableItemType::WeaponVersion1, 80),
	(EquippableItemType::WeaponVersion2, 80),
	(EquippableItemType::WeaponVersion3, 80),
];

pub(crate) const PACK_TYPE_EQUIPMENT_BLUEPRINT_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	BlueprintItemType,
	1,
> = [(BlueprintItemType::Blueprint, 1000)];

pub(crate) const PACK_TYPE_EQUIPMENT_SPECIAL_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	SpecialItemType,
	2,
> = [(SpecialItemType::Dust, 990), (SpecialItemType::Unidentified, 10)];
// -----------------------------------------------

/// Probabilities for all PackType::Special options
pub(crate) const PACK_TYPE_SPECIAL_ITEM_PROBABILITIES: ProbabilitySlots<ItemType, 6> = [
	(ItemType::Pet, 300),
	(ItemType::Material, 350),
	(ItemType::Essence, 100),
	(ItemType::Equippable, 50),
	(ItemType::Blueprint, 0),
	(ItemType::Special, 200),
];

pub(crate) const PACK_TYPE_SPECIAL_PET_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<PetItemType, 3> =
	[(PetItemType::Pet, 10), (PetItemType::PetPart, 390), (PetItemType::Egg, 600)];

pub(crate) const PACK_TYPE_SPECIAL_MATERIAL_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	MaterialItemType,
	8,
> = [
	(MaterialItemType::Polymers, 125),
	(MaterialItemType::Electronics, 125),
	(MaterialItemType::PowerCells, 125),
	(MaterialItemType::Optics, 125),
	(MaterialItemType::Metals, 125),
	(MaterialItemType::Ceramics, 125),
	(MaterialItemType::Superconductors, 125),
	(MaterialItemType::Nanomaterials, 125),
];

pub(crate) const PACK_TYPE_SPECIAL_ESSENCE_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	EssenceItemType,
	5,
> = [
	(EssenceItemType::Glimmer, 800),
	(EssenceItemType::ColorSpark, 75),
	(EssenceItemType::GlowSpark, 75),
	(EssenceItemType::PaintFlask, 25),
	(EssenceItemType::ForceGlow, 25),
];

pub(crate) const PACK_TYPE_SPECIAL_EQUIPABLE_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	EquippableItemType,
	7,
> = [
	(EquippableItemType::ArmorBase, 10),
	(EquippableItemType::ArmorComponent1, 250),
	(EquippableItemType::ArmorComponent2, 250),
	(EquippableItemType::ArmorComponent3, 250),
	(EquippableItemType::WeaponVersion1, 80),
	(EquippableItemType::WeaponVersion2, 80),
	(EquippableItemType::WeaponVersion3, 80),
];

pub(crate) const PACK_TYPE_SPECIAL_BLUEPRINT_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	BlueprintItemType,
	1,
> = [(BlueprintItemType::Blueprint, 1000)];

pub(crate) const PACK_TYPE_SPECIAL_SPECIAL_ITEM_TYPE_PROBABILITIES: ProbabilitySlots<
	SpecialItemType,
	2,
> = [(SpecialItemType::Dust, 990), (SpecialItemType::Unidentified, 10)];
// -----------------------------------------------

/// Probabilities for equipment slots
pub(crate) const ARMOR_SLOT_PROBABILITIES: ProbabilitySlots<SlotType, 6> = [
	(SlotType::Head, 170),
	(SlotType::Breast, 170),
	(SlotType::ArmFront, 165),
	(SlotType::ArmBack, 165),
	(SlotType::LegFront, 165),
	(SlotType::LegBack, 165),
];

pub(crate) const WEAPON_SLOT_PROBABILITIES: ProbabilitySlots<SlotType, 2> =
	[(SlotType::WeaponFront, 500), (SlotType::WeaponBack, 500)];

/// Probabilities for pet type
pub(crate) const PET_TYPE_PROBABILITIES: ProbabilitySlots<PetType, 7> = [
	(PetType::TankyBullwog, 150),
	(PetType::FoxishDude, 150),
	(PetType::WierdFerry, 150),
	(PetType::FireDino, 150),
	(PetType::BigHybrid, 150),
	(PetType::GiantWoodStick, 150),
	(PetType::CrazyDude, 100),
];

/// Probabilities for equipment type
pub(crate) const EQUIPMENT_TYPE_PROBABILITIES: ProbabilitySlots<EquippableItemType, 7> = [
	(EquippableItemType::ArmorBase, 250),
	(EquippableItemType::ArmorComponent1, 150),
	(EquippableItemType::ArmorComponent3, 150),
	(EquippableItemType::ArmorComponent3, 150),
	(EquippableItemType::WeaponVersion1, 100),
	(EquippableItemType::WeaponVersion2, 100),
	(EquippableItemType::WeaponVersion3, 100),
];
