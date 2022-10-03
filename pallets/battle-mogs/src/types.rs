/*
 _______ __                       _______         __
|   _   |__|.--.--.-----.---.-.  |    |  |.-----.|  |_.
|       |  ||  |  |     |  _  |  |       ||  -__||   _|.--.
|___|___|  ||_____|__|__|___._|  |__|____||_____||____||__|
	   |___|
 .............<-::]] Ajuna Network (ajuna.io) [[::->.............
+-----------------------------------------------------------------
| This file is part of the BattleMogs project from Ajuna Network.
¦-----------------------------------------------------------------
| Copyright (c) 2022 BloGa Tech AG
| Copyright (c) 2020 DOT Mog Team (darkfriend77 & metastar77)
¦-----------------------------------------------------------------
| Authors: darkfriend77
| License: GNU Affero General Public License v3.0
+-----------------------------------------------------------------
*/
use codec::MaxEncodedLen;
use frame_support::codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct MogwaiStruct<Hash, BlockNumber, Balance, MogwaiGeneration, PhaseType> {
	pub id: Hash,
	pub dna: [[u8; 32]; 2],
	//	pub state: u32,
	//  pub level: u32,
	pub genesis: BlockNumber,
	pub intrinsic: Balance,
	pub generation: MogwaiGeneration,
	pub rarity: u8,
	pub phase: PhaseType,
}

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum MogwaiGeneration {
	First = 1,
	Second = 2,
	Third = 3,
	Fourth = 4,
	Fifth = 5,
	Sixth = 6,
	Seventh = 7,
	Eighth = 8,
	Ninth = 9,
	Tenth = 10,
	Eleventh = 11,
	Twelfth = 12,
	Thirteenth = 13,
	Fourteenth = 14,
	Fifteenth = 15,
	Sixteenth = 16,
}

impl MogwaiGeneration {
	pub fn coerce_from(num: u16) -> Self {
		match num {
			0 => Self::First,
			1..=16 => Self::from(num),
			_ => Self::Sixteenth,
		}
	}
}

impl Default for MogwaiGeneration {
	fn default() -> Self {
		Self::First
	}
}

impl From<u8> for MogwaiGeneration {
	fn from(num: u8) -> Self {
		MogwaiGeneration::from(num as u16)
	}
}

impl From<u16> for MogwaiGeneration {
	fn from(num: u16) -> Self {
		match num {
			1 => MogwaiGeneration::First,
			2 => MogwaiGeneration::Second,
			3 => MogwaiGeneration::Third,
			4 => MogwaiGeneration::Fourth,
			5 => MogwaiGeneration::Fifth,
			6 => MogwaiGeneration::Sixth,
			7 => MogwaiGeneration::Seventh,
			8 => MogwaiGeneration::Eighth,
			9 => MogwaiGeneration::Ninth,
			10 => MogwaiGeneration::Tenth,
			11 => MogwaiGeneration::Eleventh,
			12 => MogwaiGeneration::Twelfth,
			13 => MogwaiGeneration::Thirteenth,
			14 => MogwaiGeneration::Fourteenth,
			15 => MogwaiGeneration::Fifteenth,
			16 => MogwaiGeneration::Sixteenth,
			_ => MogwaiGeneration::First,
		}
	}
}

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, TypeInfo)]
pub enum BreedType {
	DomDom = 0,
	DomRez = 1,
	RezDom = 2,
	RezRez = 3,
}

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum RarityType {
	Common = 0,
	Uncommon = 1,
	Rare = 2,
	Epic = 3,
	Legendary = 4,
	Mythical = 5,
}

impl Default for RarityType {
	fn default() -> Self {
		Self::Common
	}
}

impl From<u8> for RarityType {
	fn from(num: u8) -> Self {
		RarityType::from(num as u16)
	}
}

impl From<u16> for RarityType {
	fn from(num: u16) -> Self {
		match num {
			0 => RarityType::Common,
			1 => RarityType::Uncommon,
			2 => RarityType::Rare,
			3 => RarityType::Epic,
			4 => RarityType::Legendary,
			5 => RarityType::Mythical,
			_ => RarityType::Common,
		}
	}
}

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum PhaseType {
	None = 0,
	Bred = 1,
	Hatched = 2,
	Matured = 3,
	Mastered = 4,
	Exalted = 5,
}

impl Default for PhaseType {
	fn default() -> Self {
		Self::None
	}
}

pub type Balance = u128;
pub const MILLIMOGS: Balance = 1_000_000_000;
pub const DMOGS: Balance = 1_000 * MILLIMOGS;

#[derive(Encode, Decode, Copy, Clone, PartialEq, TypeInfo)]
pub enum FeeType {
	Default = 0,
	Remove = 1,
}

impl Default for FeeType {
	fn default() -> Self {
		Self::Default
	}
}

pub struct Pricing;
impl Pricing {
	pub fn config_update_price(index: u8, value: u8) -> Balance {
		let price: Balance;
		match index {
			// Config max. Mogwais in account
			1 => price = Self::config_max_mogwais(value),
			_ => price = 0,
		}
		price
	}
	fn config_max_mogwais(value: u8) -> Balance {
		let price: Balance;
		match value {
			1 => price = 5 * DMOGS,
			2 => price = 10 * DMOGS,
			3 => price = 20 * DMOGS,
			_ => price = 0 * DMOGS,
		}
		price
	}
	pub fn fee_price(fee: FeeType) -> Balance {
		let price: Balance;
		match fee {
			FeeType::Default => price = 1 * MILLIMOGS,
			FeeType::Remove => price = 50 * MILLIMOGS,
		}

		price
	}
	pub fn intrinsic_return(phase: PhaseType) -> Balance {
		let price: Balance;

		match phase {
			PhaseType::None => price = 0 * MILLIMOGS,
			PhaseType::Bred => price = 20 * MILLIMOGS,
			PhaseType::Hatched => price = 5 * MILLIMOGS,
			PhaseType::Matured => price = 3 * MILLIMOGS,
			PhaseType::Mastered => price = 2 * MILLIMOGS,
			PhaseType::Exalted => price = 1 * MILLIMOGS,
		}

		price
	}
	pub fn pairing(rarity1: u8, rarity2: u8) -> Balance {
		let price: Balance;
		match rarity1 as u32 + rarity2 as u32 {
			0 => price = 10 * MILLIMOGS,
			1 => price = 100 * MILLIMOGS,
			2 => price = 200 * MILLIMOGS,
			3 => price = 300 * MILLIMOGS,
			4 => price = 400 * MILLIMOGS,
			5 => price = 500 * MILLIMOGS,
			6 => price = 1000 * MILLIMOGS,
			7 => price = 1500 * MILLIMOGS,
			8 => price = 2000 * MILLIMOGS,
			_ => price = 10000 * MILLIMOGS,
		}

		price
	}
}

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum GameConfigType {
	Activated = 0,
	MaxMogwaisInAccount = 1,
	MaxStashSize = 2,
	AccountNaming = 3,
}

impl Default for GameConfigType {
	fn default() -> Self {
		Self::Activated
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct GameConfig {
	pub parameters: [u8; GameConfig::PARAM_COUNT],
}

impl GameConfig {
	pub const PARAM_COUNT: usize = 10;

	pub fn new() -> Self {
		let parameters = [0; GameConfig::PARAM_COUNT];

		return GameConfig { parameters }
	}
	pub fn config_value(index: u8, value: u8) -> u32 {
		let result: u32;
		match index {
			// MaxMogwaisInAccount
			1 => match value {
				0 => result = 6,
				1 => result = 12,
				2 => result = 18,
				3 => result = 24,
				_ => result = 0,
			},
			_ => result = 0,
		}
		result
	}
	pub fn verify_update(index: u8, value: u8, update_value_opt: Option<u8>) -> u8 {
		let mut result: u8;
		match index {
			// MaxMogwaisInAccount
			1 => match value {
				0 => result = 1,
				1 => result = 2,
				2 => result = 3,
				_ => result = 0,
			},
			_ => result = 0,
		}
		// don't allow bad requests
		if update_value_opt.is_some() && result != update_value_opt.unwrap() {
			result = 0;
		}
		result
	}
}

#[derive(Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum GameEventType {
	Default = 0,
	Hatch = 1,
}

impl Default for GameEventType {
	fn default() -> Self {
		Self::Default
	}
}

impl GameEventType {
	pub fn time_till(game_type: GameEventType) -> u16 {
		match game_type {
			GameEventType::Hatch => 100,
			GameEventType::Default => 0,
		}
	}

	pub fn duration(game_type: GameEventType) -> u16 {
		match game_type {
			GameEventType::Hatch => 0,
			GameEventType::Default => 0,
		}
	}
}
