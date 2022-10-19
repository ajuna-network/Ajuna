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

#[derive(
	Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, TypeInfo, MaxEncodedLen,
)]
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
	Breeded = 1,
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

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum AchievementState {
	InProgress { current: u16, target: u16 },
	Completed,
}

impl AchievementState {
	pub fn new(target: u16) -> Self {
		Self::InProgress { current: Default::default(), target }
	}

	pub fn update(self, amount: u16) -> Self {
		match self {
			AchievementState::InProgress { current, target } => {
				let new_current = current + amount;

				if new_current >= target {
					Self::Completed
				} else {
					Self::InProgress { current: new_current, target }
				}
			},
			AchievementState::Completed => Self::Completed,
		}
	}
}

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum AccountAchievement {
	EggHatcher = 0,
	Sacrificer = 1,
	Morpheus = 2,
	LegendBreeder = 3,
	Promiscuous = 4,
	Buyer = 5,
	Seller = 6,
}

impl AccountAchievement {
	pub fn target_for(&self) -> u16 {
		match self {
			AccountAchievement::EggHatcher => 100,
			AccountAchievement::Sacrificer => 100,
			AccountAchievement::Morpheus => 100,
			AccountAchievement::LegendBreeder => 1,
			AccountAchievement::Promiscuous => 50,
			AccountAchievement::Buyer => 10,
			AccountAchievement::Seller => 100,
		}
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
			PhaseType::Breeded => price = 20 * MILLIMOGS,
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

pub struct Breeding;

impl Breeding {
	pub fn sacrifice(
		gen1: u32,
		rar1: u32,
		dna1: [[u8; 32]; 2],
		gen2: u32,
		rar2: u32,
		dna2: [[u8; 32]; 2],
	) -> u32 {
		let mut result_gen: u32 = 0;

		let mut gen_diff: u32 = 0;
		if gen1 > gen2 {
			gen_diff = gen1 - gen2;
		}

		let mut rarity_diff: u32 = 0;
		if rar2 > rar1 {
			rarity_diff = rar2 - rar1;
		}

		if rarity_diff == 0 || gen_diff == 0 {
			result_gen = gen_diff;
		} else {
			let mut max_gen: u32 = ((gen_diff * 2) / ((rarity_diff + 1) * rar2)) + 1;
			if (gen2 + max_gen) > 16 {
				max_gen = 16 - gen2;
			}

			let prob_aug: u32 = 10;
			let prob_rar: u32 = rarity_diff * 4;
			let prob_gen: u32 = gen_diff * 20;

			let mut prob: u32 = (256 / (rar2 + prob_rar)) + prob_aug;

			if prob_gen > prob_rar * 2 {
				prob += prob_gen - (prob_rar * 2);
			}

			let mut final_prob: u8 = 255;
			if prob < 256 {
				final_prob = prob as u8;
			}

			let gen_add = gen1 + gen2;
			let pos1: u8 = dna1[0][((gen_add + rar2) % 32) as usize];
			let pos2: u8 = dna2[0][((gen_add + rar1) % 32) as usize];

			let val1: u8 = dna1[0][(pos2 % 32) as usize];
			let val2: u8 = dna2[0][(pos1 % 32) as usize];

			if val1 < final_prob && val2 < final_prob {
				result_gen = (val1 as u32 + val2 as u32) % max_gen + 1;
			}
		}

		result_gen
	}

	pub fn morph(breed_type: BreedType, gen1: [u8; 16], gen2: [u8; 16]) -> [u8; 32] {
		let mut final_dna: [u8; 32] = [0; 32];

		let (ll, rr) = final_dna.split_at_mut(16);
		let (l1, l2) = ll.split_at_mut(8);
		let (r1, r2) = rr.split_at_mut(8);

		match breed_type {
			BreedType::DomDom => {
				l1.copy_from_slice(&gen1[..8]);
				l2.copy_from_slice(&gen1[8..16]);
				r1.copy_from_slice(&gen2[..8]);
				r2.copy_from_slice(&gen2[8..16]);
			},
			BreedType::DomRez => {
				l1.copy_from_slice(&gen1[..8]);
				l2.copy_from_slice(&gen1[8..16]);
				r1.copy_from_slice(&gen2[8..16]);
				r2.copy_from_slice(&gen2[..8]);
			},
			BreedType::RezDom => {
				l1.copy_from_slice(&gen1[8..16]);
				l2.copy_from_slice(&gen1[..8]);
				r1.copy_from_slice(&gen2[8..16]);
				r2.copy_from_slice(&gen2[..8]);
			},
			BreedType::RezRez => {
				l1.copy_from_slice(&gen1[8..16]);
				l2.copy_from_slice(&gen1[..8]);
				r1.copy_from_slice(&gen2[..8]);
				r2.copy_from_slice(&gen2[8..16]);
			},
		}
		return final_dna
	}

	pub fn pairing(breed_type: BreedType, gen1: [u8; 32], gen2: [u8; 32]) -> [[u8; 32]; 2] {
		let mut ll: [u8; 32] = [0u8; 32];
		let mut rr: [u8; 32] = [0u8; 32];

		let (l1, l2) = ll.split_at_mut(16);
		let (r1, r2) = rr.split_at_mut(16);

		match breed_type {
			BreedType::DomDom => {
				l1.copy_from_slice(&gen1[..16]);
				l2.copy_from_slice(&gen1[16..32]);
				r1.copy_from_slice(&gen2[..16]);
				r2.copy_from_slice(&gen2[16..32]);
			},
			BreedType::DomRez => {
				l1.copy_from_slice(&gen1[..16]);
				l2.copy_from_slice(&gen1[16..32]);
				r1.copy_from_slice(&gen2[16..32]);
				r2.copy_from_slice(&gen2[..16]);
			},
			BreedType::RezDom => {
				l1.copy_from_slice(&gen1[16..32]);
				l2.copy_from_slice(&gen1[..16]);
				r1.copy_from_slice(&gen2[16..32]);
				r2.copy_from_slice(&gen2[..16]);
			},
			BreedType::RezRez => {
				l1.copy_from_slice(&gen1[16..32]);
				l2.copy_from_slice(&gen1[..16]);
				r1.copy_from_slice(&gen2[..16]);
				r2.copy_from_slice(&gen2[16..32]);
			},
		}

		let mut result: [[u8; 32]; 2] = [[0u8; 32]; 2];
		result[0] = ll;
		result[1] = rr;

		return result
	}

	pub fn segmenting(gen: [[u8; 32]; 2], blk: [u8; 32]) -> [[u8; 32]; 2] {
		let a_sec = &gen[0][0..32];
		let b_sec = &gen[1][0..32];

		//let a_x = &gen[0 ..  8];
		let a_y = &gen[0][16..32];
		let b_x = &gen[1][0..16];
		//let b_y = &gen[24 .. 32];

		let a_c = &a_y[0..8];
		let b_c = &b_x[0..8];

		let mut dna: [u8; 32] = Default::default();
		let mut evo: [u8; 32] = Default::default();

		let mut full: u8 = 0;
		let mut mark: u8 = 0;

		for i in 0..64 {
			let bit_a = Binary::get_bit_at(a_c[i / 8], i as u8 % 8);
			let bit_b = Binary::get_bit_at(b_c[i / 8], i as u8 % 8);

			let p1: usize = i;
			let p2: usize = 63 - i;
			let blk_a = Binary::get_bit_at(blk[p1 / 8], p1 as u8 % 8);
			let blk_b = Binary::get_bit_at(blk[p2 / 8], p2 as u8 % 8);

			let mut half_byte: u8 = dna[i / 2];
			let mut mark_byte: u8 = evo[i / 2];

			let a_byte = a_sec[i / 2];
			let b_byte = b_sec[i / 2];
			let side = i % 2;

			if side == 0 {
				full = 0;
				mark = 0;
			}

			// 1 - 0
			if bit_a && !bit_b {
				if blk_a {
					half_byte = Binary::copy_bits(half_byte, a_byte, side); // A+ as 4
					half_byte = Binary::add_one(half_byte, side);
					mark_byte = Binary::copy_bits(mark_byte, 0x44, side);
				} else if !blk_b {
					half_byte = Binary::copy_bits(half_byte, a_byte, side); // A as A
					mark_byte = Binary::copy_bits(mark_byte, 0xAA, side);
				} else {
					half_byte = Binary::copy_bits(half_byte, a_byte ^ b_byte, side); // A^B as 7
					mark_byte = Binary::copy_bits(mark_byte, 0x77, side);
				}
			} else
			// 0 - 1
			if !bit_a && bit_b {
				if blk_b {
					half_byte = Binary::copy_bits(half_byte, b_byte, side); // 8
					mark_byte = Binary::copy_bits(mark_byte, 0x88, side);
					half_byte = Binary::add_one(half_byte, side);
				} else if !blk_a {
					half_byte = Binary::copy_bits(half_byte, b_byte, side); // B
					mark_byte = Binary::copy_bits(mark_byte, 0xBB, side);
				} else {
					half_byte = Binary::copy_bits(half_byte, b_byte ^ a_byte, side); // A^B as 7
					mark_byte = Binary::copy_bits(mark_byte, 0x77, side);
				}
			} else
			// 0 - 0
			if !bit_a && !bit_b {
				if !blk_a && !blk_b {
					if bit_a < bit_b {
						half_byte = Binary::copy_bits(half_byte, a_byte & !b_byte, side); // !b- as 1
						half_byte = Binary::sub_one(half_byte, side);
						mark_byte = Binary::copy_bits(mark_byte, 0x11, side);
					} else {
						half_byte = Binary::copy_bits(half_byte, !a_byte & b_byte, side); // !a- as 0
						mark_byte = Binary::copy_bits(mark_byte, 0x00, side);
						half_byte = Binary::sub_one(half_byte, side);
					}
				} else if blk_a && blk_b {
					half_byte = Binary::copy_bits(half_byte, !blk[i % 32], side); // !blk as E
					mark_byte = Binary::copy_bits(mark_byte, 0xEE, side);
				} else {
					if blk_a {
						half_byte = Binary::copy_bits(half_byte, a_byte, side); // A
						mark_byte = Binary::copy_bits(mark_byte, 0xAA, side);
					} else {
						half_byte = Binary::copy_bits(half_byte, b_byte, side); // B
						mark_byte = Binary::copy_bits(mark_byte, 0xBB, side);
					}
				}
			} else
			// 1 - 1
			{
				if blk_a && blk_b {
					half_byte = Binary::copy_bits(half_byte, a_byte | b_byte, side); // |+ as C
					half_byte = Binary::add_one(half_byte, side);
					mark_byte = Binary::copy_bits(mark_byte, 0xCC, side);
				} else if !blk_a && !blk_b {
					half_byte = Binary::copy_bits(half_byte, blk[i % 32], side); // blk as F
					mark_byte = Binary::copy_bits(mark_byte, 0xFF, side);
				} else {
					if blk_a {
						half_byte = Binary::copy_bits(half_byte, a_byte, side); // A
						mark_byte = Binary::copy_bits(mark_byte, 0xAA, side);
					} else {
						half_byte = Binary::copy_bits(half_byte, b_byte, side); // B
						mark_byte = Binary::copy_bits(mark_byte, 0xBB, side);
					}
				}
			}

			full = Binary::copy_bits(full, half_byte, side);
			mark = Binary::copy_bits(mark, mark_byte, side);

			// recombination
			if side == 1 {
				if full == 0xFF || full == 0x00 {
					full &= blk[i % 32];
					mark = 0x33;
				}
				dna[i / 2] = full;
				evo[i / 2] = mark;
			}
		}

		let mut result = [[0u8; 32]; 2];
		result[0] = dna;
		result[1] = evo;

		result
	}

	/// baking an egg will reroll on rarity with a certain probability
	pub fn bake(rarity: u8, blk: [u8; 32]) -> u8 {
		let prob: u16 = 250;

		let mut result = (rarity << 4) >> 4;
		let max_rarity = rarity >> 4;

		let mut rand: [u16; 16] = [0u16; 16];
		for i in 0..(max_rarity + 1) {
			let p = (i * 2) as usize;
			rand[i as usize] = (((blk[p] as u16) << 8) | blk[p + 1] as u16) % 1000;
		}

		if rand[max_rarity as usize] > prob {
			for i in 0..max_rarity {
				if rand[i as usize] > prob {
					result = i;
					break
				}
			}
		}

		result
	}
}

pub struct Generation {}

impl Generation {
	fn compute_next_gen_and_rarity(
		generation: &MogwaiGeneration,
		rarity: &RarityType,
		hash: &[u8; 6],
	) -> (RarityType, MogwaiGeneration) {
		let generation = *generation as u16;
		let rarity = (*rarity as u16).saturating_sub(1);

		let gen_multiplier =
			if generation >= rarity { (generation * 2) - rarity } else { generation * 2 };

		let mut out_rarity = MogwaiGeneration::default() as u16;
		let mut out_generation = generation;

		let rng_gen_1 = hash[0] as u16 + hash[1] as u16;
		let rng_gen_3 = hash[4] as u16 + hash[5] as u16;

		if (rng_gen_1 % gen_multiplier) == 0 {
			out_generation += 1;
			out_rarity = 1;
			let rng_gen_2 = hash[2] as u16 + hash[3] as u16;
			if (rng_gen_2 % gen_multiplier) < (out_generation / 2) {
				out_generation += 1;
				out_rarity = 2;
				if (rng_gen_3 % gen_multiplier) < (out_generation / 2) {
					out_generation += 1;
					out_rarity = 3;
				}
			}
		} else if (rng_gen_3 % gen_multiplier) == 0 {
			out_generation -= 1;
		}

		(RarityType::from(out_rarity), MogwaiGeneration::from(out_generation))
	}

	pub fn next_gen(
		gen_1: MogwaiGeneration,
		rar_1: RarityType,
		gen_2: MogwaiGeneration,
		rar_2: RarityType,
		random_hash: &[u8],
	) -> (RarityType, MogwaiGeneration, RarityType) {
		let mut resulting_gen = MogwaiGeneration::default();
		let mut resulting_rarity = RarityType::default();

		if random_hash.len() >= 12 {
			let base_rarity = (rar_1 as u16 + rar_2 as u16).saturating_sub(2) / 2;

			let slice = unsafe { &*(&random_hash[0..6] as *const [u8] as *const [u8; 6]) };
			let (out_rarity_1, out_gen_1) =
				Self::compute_next_gen_and_rarity(&gen_1, &rar_1, slice);

			let slice = unsafe { &*(&random_hash[6..12] as *const [u8] as *const [u8; 6]) };
			let (out_rarity_2, out_gen_2) =
				Self::compute_next_gen_and_rarity(&gen_2, &rar_2, slice);

			resulting_gen = MogwaiGeneration::coerce_from(
				(out_gen_1 as u16 + out_gen_2 as u16 + base_rarity) / 2,
			);

			resulting_rarity = RarityType::from(
				((out_rarity_1 as u16 + out_rarity_2 as u16 + ((rar_1 as u16 + rar_2 as u16) / 2)) /
					2) % 5,
			)
		}

		let max_rarity = RarityType::from((6 + ((rar_1 as u16 + rar_2 as u16) / 2 as u16) / 2) % 5);

		(resulting_rarity, resulting_gen, max_rarity)
	}
}

struct Binary {}

impl Binary {
	pub fn get_bit_at(input: u8, n: u8) -> bool {
		input & (1 << n) != 0
	}

	pub fn copy_bits(mut old: u8, mut new: u8, side: usize) -> u8 {
		if side == 0 {
			new = new & 0xF0;
		} else {
			new = new & 0x0F;
		}
		old |= new;
		old
	}

	pub fn add_one(mut old: u8, side: usize) -> u8 {
		let mut new = old.clone();
		if side == 0 {
			old = old & 0x0F;
			new >>= 4;
			new += 1;
			new <<= 4;
			if new == 0 {
				new = 0xF0;
			}
		} else {
			old = old & 0xF0;
			new = new & 0x0F;
			new += 1;
			new = new & 0x0F;
			if new == 0 {
				new = 0x0F;
			}
		}
		new |= old;
		new
	}

	pub fn sub_one(mut old: u8, side: usize) -> u8 {
		let mut new = old.clone();
		if side == 0 {
			old = old & 0x0F;
			new >>= 4;
			if new != 0 {
				new -= 1;
			}
			new <<= 4;
		} else {
			old = old & 0xF0;
			new = new & 0x0F;
			if new > 0 {
				new -= 1;
			}
			new = new & 0x0F;
		}
		new |= old;
		new
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
