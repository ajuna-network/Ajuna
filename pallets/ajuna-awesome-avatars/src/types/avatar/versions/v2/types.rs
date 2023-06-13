use crate::{ByteConvertible, Ranged};
use sp_std::ops::Range;

#[derive(Clone, Default)]
pub(crate) enum ByteType {
	#[default]
	Full = 0b1111_1111,
	High = 0b0000_1111,
	Low = 0b1111_0000,
}

impl ByteConvertible for ByteType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0xFF => Self::Full,
			0x0F => Self::High,
			0xF0 => Self::Low,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) enum HexType {
	#[default]
	X0 = 0b0000,
	X1 = 0b0001,
	X2 = 0b0010,
	X3 = 0b0011,
	X4 = 0b0100,
	X5 = 0b0101,
	X6 = 0b0110,
	X7 = 0b0111,
	X8 = 0b1000,
	X9 = 0b1001,
	XA = 0b1010,
	XB = 0b1011,
	XC = 0b1100,
	XD = 0b1101,
	XE = 0b1110,
	XF = 0b1111,
}

impl ByteConvertible for HexType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0x0 => Self::X0,
			0x1 => Self::X1,
			0x2 => Self::X2,
			0x3 => Self::X3,
			0x4 => Self::X4,
			0x5 => Self::X5,
			0x6 => Self::X6,
			0x7 => Self::X7,
			0x8 => Self::X8,
			0x9 => Self::X9,
			0xA => Self::XA,
			0xB => Self::XB,
			0xC => Self::XC,
			0xD => Self::XD,
			0xE => Self::XE,
			0xF => Self::XF,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NibbleType {
	#[default]
	X0 = 0b0000,
	X1 = 0b0001,
	X2 = 0b0010,
	X3 = 0b0011,
	X4 = 0b0100,
	X5 = 0b0101,
	X6 = 0b0110,
	X7 = 0b0111,
}

impl ByteConvertible for NibbleType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0b0000 => Self::X0,
			0b0001 => Self::X1,
			0b0010 => Self::X2,
			0b0011 => Self::X3,
			0b0100 => Self::X4,
			0b0101 => Self::X5,
			0b0110 => Self::X6,
			0b0111 => Self::X7,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl Ranged for NibbleType {
	fn range() -> Range<usize> {
		0..8
	}
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) enum ItemType {
	#[default]
	Pet = 1,
	Material = 2,
	Essence = 3,
	Equippable = 4,
	Blueprint = 5,
	Special = 6,
}

impl ByteConvertible for ItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Pet,
			2 => Self::Material,
			3 => Self::Essence,
			4 => Self::Equippable,
			5 => Self::Blueprint,
			6 => Self::Special,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum PetItemType {
	#[default]
	Pet = 1,
	PetPart = 2,
	Egg = 3,
}

impl ByteConvertible for PetItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Pet,
			2 => Self::PetPart,
			3 => Self::Egg,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub(crate) enum EquippableItemType {
	#[default]
	ArmorBase = 1,
	ArmorComponent1 = 2,
	ArmorComponent2 = 3,
	ArmorComponent3 = 4,
	WeaponVersion1 = 5,
	WeaponVersion2 = 6,
	WeaponVersion3 = 7,
}

impl ByteConvertible for EquippableItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::ArmorBase,
			2 => Self::ArmorComponent1,
			3 => Self::ArmorComponent2,
			4 => Self::ArmorComponent3,
			5 => Self::WeaponVersion1,
			6 => Self::WeaponVersion2,
			7 => Self::WeaponVersion3,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for EquippableItemType {
	fn range() -> Range<usize> {
		1..8
	}
}

impl EquippableItemType {
	pub fn is_armor(item: EquippableItemType) -> bool {
		item == EquippableItemType::ArmorBase ||
			item == EquippableItemType::ArmorComponent1 ||
			item == EquippableItemType::ArmorComponent2 ||
			item == EquippableItemType::ArmorComponent3
	}

	pub fn is_weapon(item: EquippableItemType) -> bool {
		item == EquippableItemType::WeaponVersion1 ||
			item == EquippableItemType::WeaponVersion2 ||
			item == EquippableItemType::WeaponVersion3
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PetType {
	#[default]
	TankyBullwog = 1,
	FoxishDude = 2,
	WierdFerry = 3,
	FireDino = 4,
	BigHybrid = 5,
	GiantWoodStick = 6,
	CrazyDude = 7,
}

impl ByteConvertible for PetType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::TankyBullwog,
			2 => Self::FoxishDude,
			3 => Self::WierdFerry,
			4 => Self::FireDino,
			5 => Self::BigHybrid,
			6 => Self::GiantWoodStick,
			7 => Self::CrazyDude,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl Ranged for PetType {
	fn range() -> Range<usize> {
		1..8
	}
}

#[derive(Clone, Debug, Default)]
pub(crate) enum PetPartType {
	#[default]
	Horns = 1,
	Furs = 2,
	Wings = 3,
	Scales = 4,
	Claws = 5,
	Sticks = 6,
	Eyes = 7,
}

impl ByteConvertible for PetPartType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Horns,
			2 => Self::Furs,
			3 => Self::Wings,
			4 => Self::Scales,
			5 => Self::Claws,
			6 => Self::Sticks,
			7 => Self::Eyes,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SlotType {
	#[default]
	Head = 1,
	Breast = 2,
	ArmFront = 3,
	ArmBack = 4,
	LegFront = 5,
	LegBack = 6,
	Tail = 7,
	WeaponFront = 8,
	WeaponBack = 9,
}

impl ByteConvertible for SlotType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Head,
			2 => Self::Breast,
			3 => Self::ArmFront,
			4 => Self::ArmBack,
			5 => Self::LegFront,
			6 => Self::LegBack,
			7 => Self::Tail,
			8 => Self::WeaponFront,
			9 => Self::WeaponBack,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl Ranged for SlotType {
	fn range() -> Range<usize> {
		1..10
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MaterialItemType {
	#[default]
	Polymers = 1,
	Electronics = 2,
	PowerCells = 3,
	Optics = 4,
	Metals = 5,
	Ceramics = 6,
	Superconductors = 7,
	Nanomaterials = 8,
}

impl ByteConvertible for MaterialItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Polymers,
			2 => Self::Electronics,
			3 => Self::PowerCells,
			4 => Self::Optics,
			5 => Self::Metals,
			6 => Self::Ceramics,
			7 => Self::Superconductors,
			8 => Self::Nanomaterials,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl Ranged for MaterialItemType {
	fn range() -> Range<usize> {
		1..9
	}
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum EssenceItemType {
	#[default]
	Glimmer = 1,
	ColorSpark = 2,
	GlowSpark = 3,
	PaintFlask = 4,
	GlowFlask = 5,
}

impl ByteConvertible for EssenceItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Glimmer,
			2 => Self::ColorSpark,
			3 => Self::GlowSpark,
			4 => Self::PaintFlask,
			5 => Self::GlowFlask,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum BlueprintItemType {
	#[default]
	Blueprint = 1,
}

impl ByteConvertible for BlueprintItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Blueprint,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum ColorType {
	#[default]
	None = 0,
	ColorA = 1,
	ColorB = 2,
	ColorC = 3,
	ColorD = 4,
}

impl ByteConvertible for ColorType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0 => Self::None,
			1 => Self::ColorA,
			2 => Self::ColorB,
			3 => Self::ColorC,
			4 => Self::ColorD,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}

impl Ranged for ColorType {
	fn range() -> Range<usize> {
		0..5
	}
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum SpecialItemType {
	#[default]
	Dust = 1,
	Unidentified = 2,
	Fragment = 3,
	ToolBox = 4,
}

impl ByteConvertible for SpecialItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Dust,
			2 => Self::Unidentified,
			3 => Self::Fragment,
			4 => Self::ToolBox,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		self.clone() as u8
	}
}
