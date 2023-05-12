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

mod pet;

pub use pet::*;

use super::*;

pub const PROB_SCALING_FACTOR: u32 = 1_000;
pub const MAX_QUANTITY: u16 = 8;
pub const MAX_INITIAL_SOUL: u8 = 99;

pub const SCALING_FACTOR_PERC: u8 = 100;
pub const PROGRESS_PROB_PERC: u8 = 15;
pub const PROGRESS_VARIATIONS: u8 = 6;

#[derive(Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo, Clone, Default, PartialEq)]
pub enum MintPack {
	#[default]
	Material,
	Equipment,
	Special,
}

type Prob = u32;

impl MintPack {
	pub(super) const fn probs(&self) -> [(Item, Prob); 6] {
		match self {
			Self::Material => [
				(Item::Pet, 150),
				(Item::Material, 700),
				(Item::Essence, 50),
				(Item::Equippable, 50),
				(Item::Blueprint, 0),
				(Item::Special, 50),
			],
			Self::Equipment => [
				(Item::Pet, 100),
				(Item::Material, 350),
				(Item::Essence, 300),
				(Item::Equippable, 50),
				(Item::Blueprint, 0),
				(Item::Special, 200),
			],
			Self::Special => [
				(Item::Pet, 300),
				(Item::Material, 350),
				(Item::Essence, 100),
				(Item::Equippable, 50),
				(Item::Blueprint, 0),
				(Item::Special, 200),
			],
		}
	}
}

#[derive(Default)]
pub enum Rarity {
	#[default]
	Common,
	Uncommon,
	Rare,
	Epic,
	Legendary,
	Mythical,
}

impl From<&Rarity> for u8 {
	fn from(value: &Rarity) -> Self {
		match value {
			Rarity::Common => 1,
			Rarity::Uncommon => 2,
			Rarity::Rare => 3,
			Rarity::Epic => 4,
			Rarity::Legendary => 5,
			Rarity::Mythical => 6,
		}
	}
}

impl Rarity {
	pub(crate) fn upgrade(&self) -> &Self {
		match self {
			Self::Common => &Self::Uncommon,
			Self::Uncommon => &Self::Rare,
			Self::Rare => &Self::Epic,
			Self::Epic => &Self::Legendary,
			Self::Legendary => &Self::Mythical,
			Self::Mythical => &Self::Mythical,
		}
	}
}

pub enum Item {
	Pet,
	Material,
	Essence,
	Equippable,
	Blueprint,
	Special,
}

impl From<Item> for u8 {
	fn from(value: Item) -> Self {
		match value {
			Item::Pet => 1,
			Item::Material => 2,
			Item::Essence => 3,
			Item::Equippable => 4,
			Item::Blueprint => 5,
			Item::Special => 6,
		}
	}
}

enum ByteOption {
	Full,
	High,
	Low,
}

#[derive(Clone, Default)]
pub struct DnaStrand([u8; 32]);

impl From<DnaStrand> for Dna {
	fn from(value: DnaStrand) -> Self {
		Self::truncate_from(value.0.to_vec())
	}
}

impl From<Dna> for DnaStrand {
	fn from(value: Dna) -> Self {
		let mut dna = [0_u8; 32];
		dna.copy_from_slice(&value.into_inner()[..32]);
		Self(dna)
	}
}

fn get_byte(x: u8, byte_option: ByteOption) -> u8 {
	match byte_option {
		ByteOption::Full => x,
		ByteOption::High => x >> 4,
		ByteOption::Low => x & 0x0F,
	}
}

fn set_byte(x: &mut u8, byte_option: ByteOption, value: u8) {
	*x = match byte_option {
		ByteOption::Full => value,
		ByteOption::High => *x & 0x0F | (value << 4),
		ByteOption::Low => *x & 0xF0 | value,
	};
}

impl DnaStrand {
	fn get(&self, byte_option: ByteOption, at: usize) -> Result<u8, ()> {
		if at > self.0.len() {
			return Err(())
		}
		Ok(get_byte(self.0[at], byte_option))
	}

	fn item_type(&mut self, value: u8) -> Result<u8, ()> {
		self.get(ByteOption::High, 0)
	}
	fn sub_item_type(&self) -> Result<u8, ()> {
		self.get(ByteOption::Low, 0)
	}
	fn class_1(&self) -> Result<u8, ()> {
		self.get(ByteOption::High, 1)
	}
	fn class_2(&self) -> Result<u8, ()> {
		self.get(ByteOption::Low, 1)
	}
	fn custom_1(&self) -> Result<u8, ()> {
		self.get(ByteOption::High, 2)
	}
	fn rarity(&self) -> Result<u8, ()> {
		self.get(ByteOption::Low, 2)
	}
	fn quantity(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 3)
	}
	fn custom_2(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 4)
	}
	fn spec_byte_1(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 5)
	}
	fn spec_byte_2(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 6)
	}
	fn spec_byte_3(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 7)
	}
	fn spec_byte_4(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 8)
	}
	fn spec_byte_5(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 9)
	}
	fn spec_byte_6(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 10)
	}
	fn spec_byte_7(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 11)
	}
	fn spec_byte_8(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 12)
	}
	fn spec_byte_9(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 13)
	}
	fn spec_byte_10(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 14)
	}
	fn spec_byte_11(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 15)
	}
	fn spec_byte_12(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 16)
	}
	fn spec_byte_13(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 17)
	}
	fn spec_byte_14(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 18)
	}
	fn spec_byte_15(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 19)
	}
	fn spec_byte_16(&self) -> Result<u8, ()> {
		self.get(ByteOption::Full, 20)
	}

	fn set(&mut self, byte_option: ByteOption, at: usize, value: u8) -> Result<&mut Self, ()> {
		if at > self.0.len() || (matches!(byte_option, ByteOption::Full) && value > 15) {
			return Err(())
		}
		set_byte(&mut self.0[at], byte_option, value);
		Ok(self)
	}
	fn set_item_type(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::High, 0, value)
	}
	fn set_sub_item_type(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Low, 0, value)
	}
	fn set_class_1(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::High, 1, value)
	}
	fn set_class_2(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Low, 1, value)
	}
	fn set_custom_1(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::High, 2, value)
	}
	fn set_rarity(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Low, 2, value)
	}
	fn set_quantity(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 3, value)
	}
	fn set_custom_2(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 4, value)
	}
	fn set_spec_byte_1(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 5, value)
	}
	fn set_spec_byte_2(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 6, value)
	}
	fn set_spec_byte_3(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 7, value)
	}
	fn set_spec_byte_4(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 8, value)
	}
	fn set_spec_byte_5(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 9, value)
	}
	fn set_spec_byte_6(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 10, value)
	}
	fn set_spec_byte_7(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 11, value)
	}
	fn set_spec_byte_8(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 12, value)
	}
	fn set_spec_byte_9(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 13, value)
	}
	fn set_spec_byte_10(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 14, value)
	}
	fn set_spec_byte_11(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 15, value)
	}
	fn set_spec_byte_12(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 16, value)
	}
	fn set_spec_byte_13(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 17, value)
	}
	fn set_spec_byte_14(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 18, value)
	}
	fn set_spec_byte_15(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 19, value)
	}
	fn set_spec_byte_16(&mut self, value: u8) -> Result<&mut Self, ()> {
		self.set(ByteOption::Full, 20, value)
	}
	fn set_spec_bytes(&mut self, values: [u8; 16]) -> Result<&mut Self, ()> {
		self.set_spec_byte_1(values[0])?
			.set_spec_byte_2(values[1])?
			.set_spec_byte_3(values[2])?
			.set_spec_byte_4(values[3])?
			.set_spec_byte_5(values[4])?
			.set_spec_byte_6(values[5])?
			.set_spec_byte_7(values[6])?
			.set_spec_byte_8(values[7])?
			.set_spec_byte_9(values[8])?
			.set_spec_byte_10(values[9])?
			.set_spec_byte_11(values[10])?
			.set_spec_byte_12(values[11])?
			.set_spec_byte_13(values[12])?
			.set_spec_byte_14(values[13])?
			.set_spec_byte_15(values[14])?
			.set_spec_byte_16(values[15])
	}
	fn set_progress_bytes(
		&mut self,
		values: [u8; 11],
		rarity: &Rarity,
		probability: u8,
	) -> Result<&mut Self, ()> {
		values.into_iter().enumerate().for_each(|(i, value)| {
			let rarity = if (value as u16 * SCALING_FACTOR_PERC as u16) <
				(probability as u16 * u8::MAX as u16)
			{
				rarity.upgrade()
			} else {
				rarity
			};

			let variation = value % PROGRESS_VARIATIONS;
			set_byte(&mut self.0[21 + i], ByteOption::Low, variation);
			set_byte(&mut self.0[21 + i], ByteOption::High, rarity.into());
		});

		// Last progress is never upgraded (ex. avoid legendary eggs).
		set_byte(&mut self.0[self.0.len() - 1], ByteOption::High, rarity.into());

		Ok(self)
	}
}

#[derive(Default)]
pub enum ArmorSlot {
	#[default]
	Head = 1,
	Breast = 2,
	ArmFront = 3,
	ArmBack = 4,
	LegFront = 5,
	LegBack = 6,
	Tail = 7,
}

impl From<u16> for ArmorSlot {
	fn from(value: u16) -> Self {
		match value {
			value if value == 0 => Self::Head,
			value if value == 1 => Self::Breast,
			value if value == 2 => Self::ArmFront,
			value if value == 3 => Self::ArmBack,
			value if value == 4 => Self::LegFront,
			value if value == 5 => Self::LegBack,
			value if value == 6 => Self::Tail,
			_ => Self::default(),
		}
	}
}

#[derive(Default)]
pub enum WeaponSlot {
	#[default]
	Front = 8,
	Back = 9,
}

impl From<u16> for WeaponSlot {
	fn from(value: u16) -> Self {
		match value {
			value if value == 0 => Self::Front,
			value if value == 1 => Self::Back,
			_ => Self::default(),
		}
	}
}

#[derive(Default)]
pub enum Color {
	#[default]
	None,
	A,
	B,
	C,
	D,
}

impl From<u8> for Color {
	fn from(value: u8) -> Self {
		match value {
			value if value == 0 => Self::None,
			value if value == 1 => Self::A,
			value if value == 2 => Self::B,
			value if value == 3 => Self::C,
			value if value == 4 => Self::D,
			_ => Self::default(),
		}
	}
}
