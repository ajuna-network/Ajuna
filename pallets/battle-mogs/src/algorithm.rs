use crate::{BreedType, MogwaiGeneration, RarityType};

use sp_std::{mem::MaybeUninit, ptr::copy_nonoverlapping};

struct Binary;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BitMaskSide {
	Left = 0,
	Right = 1,
}

impl BitMaskSide {
	pub fn flip(self) -> Self {
		match self {
			BitMaskSide::Left => Self::Right,
			BitMaskSide::Right => Self::Left,
		}
	}
}

impl Binary {
	#[inline]
	pub fn get_bit_at(input: u8, n: u8) -> bool {
		input >> n & 1 != 0
	}

	pub const LEFT_BITMASK: u8 = 0xF0;
	pub const RIGHT_BITMASK: u8 = 0x0F;

	pub const LEFT_UNIT: u8 = 0x10;
	pub const RIGHT_UNIT: u8 = 0x01;

	#[inline]
	pub fn copy_bits(from: u8, to: u8, side: BitMaskSide) -> u8 {
		from | (to &
			if side == BitMaskSide::Left { Self::LEFT_BITMASK } else { Self::RIGHT_BITMASK })
	}

	#[inline]
	pub fn add_one(num: u8, side: BitMaskSide) -> u8 {
		if side == BitMaskSide::Left {
			(num.saturating_add(Self::LEFT_UNIT) & Self::LEFT_BITMASK) | (num & Self::RIGHT_BITMASK)
		} else {
			((num | Self::LEFT_BITMASK).saturating_add(Self::RIGHT_UNIT) & Self::RIGHT_BITMASK) |
				(num & Self::LEFT_BITMASK)
		}
	}

	#[inline]
	pub fn sub_one(num: u8, side: BitMaskSide) -> u8 {
		if side == BitMaskSide::Left {
			(num.saturating_sub(Self::LEFT_UNIT) & Self::LEFT_BITMASK) | (num & Self::RIGHT_BITMASK)
		} else {
			((num & Self::RIGHT_BITMASK).saturating_sub(Self::RIGHT_UNIT) & Self::RIGHT_BITMASK) |
				(num & Self::LEFT_BITMASK)
		}
	}
}

pub struct Breeding;

impl Breeding {
	pub fn sacrifice(
		input_generation_1: MogwaiGeneration,
		input_rarity_1: RarityType,
		input_dna_1: &[[u8; 32]; 2],
		input_generation_2: MogwaiGeneration,
		input_rarity_2: RarityType,
		input_dna_2: &[[u8; 32]; 2],
	) -> MogwaiGeneration {
		let input_generation_1 = input_generation_1 as u16;
		let input_generation_2 = input_generation_2 as u16;

		let generation_diff = (input_generation_1 > input_generation_2)
			.then(|| input_generation_1 - input_generation_2)
			.unwrap_or_default();

		let input_rarity_1 = input_rarity_1 as u16;
		let input_rarity_2 = input_rarity_2 as u16;

		let rarity_diff = (input_rarity_2 > input_rarity_1)
			.then(|| input_rarity_2 - input_rarity_1)
			.unwrap_or_default();

		if rarity_diff != 0 && generation_diff != 0 {
			let max_generation = {
				let max_gen = ((generation_diff * 2) / ((rarity_diff + 1) * input_rarity_2)) + 1;

				if (input_generation_2 + max_gen) > 16 {
					16 - input_generation_2
				} else {
					max_gen
				}
			};

			let final_prob = {
				let prob = {
					let prob_aug = 10;
					let prob_rar = rarity_diff * 4;
					let prob_gen = generation_diff * 20;

					let prob = (256 / (input_rarity_2 + prob_rar)) + prob_aug;

					if prob_gen > prob_rar * 2 {
						prob + prob_gen - (prob_rar * 2)
					} else {
						prob
					}
				};

				(prob < 256).then(|| prob as u8).unwrap_or(u8::MAX)
			};

			let gen_add = input_generation_1 + input_generation_2;
			let pos1 = input_dna_1[0][((gen_add + input_rarity_2) % 32) as usize];
			let pos2 = input_dna_2[0][((gen_add + input_rarity_1) % 32) as usize];

			let val1 = input_dna_1[0][(pos2 % 32) as usize];
			let val2 = input_dna_2[0][(pos1 % 32) as usize];

			if val1 < final_prob && val2 < final_prob {
				MogwaiGeneration::coerce_from((val1 as u16 + val2 as u16) % max_generation + 1)
			} else {
				MogwaiGeneration::coerce_from(generation_diff)
			}
		} else {
			MogwaiGeneration::coerce_from(generation_diff)
		}
	}

	pub fn morph(
		breed_type: BreedType,
		left_source_dna: &[u8; 16],
		right_source_dna: &[u8; 16],
	) -> [u8; 32] {
		let mut final_dna: MaybeUninit<[u8; 32]> = MaybeUninit::uninit();

		let (left_indexes, right_indexes) = match breed_type {
			BreedType::DomDom => ((0..8, 8..16), (0..8, 8..16)),
			BreedType::DomRez => ((0..8, 8..16), (8..16, 0..8)),
			BreedType::RezDom => ((8..16, 0..8), (8..16, 0..8)),
			BreedType::RezRez => ((8..16, 0..8), (0..8, 8..16)),
		};

		unsafe {
			let dna_ptr = final_dna.as_mut_ptr() as *mut u8;

			copy_nonoverlapping(left_source_dna[left_indexes.0].as_ptr(), dna_ptr, 8);
			copy_nonoverlapping(left_source_dna[left_indexes.1].as_ptr(), dna_ptr.add(8), 8);
			copy_nonoverlapping(right_source_dna[right_indexes.0].as_ptr(), dna_ptr.add(16), 8);
			copy_nonoverlapping(right_source_dna[right_indexes.1].as_ptr(), dna_ptr.add(24), 8);

			final_dna.assume_init()
		}
	}

	pub fn pairing(
		breed_type: BreedType,
		left_source_dna: &[u8; 32],
		right_source_dna: &[u8; 32],
	) -> [[u8; 32]; 2] {
		let mut left_dna: MaybeUninit<[u8; 32]> = MaybeUninit::uninit();
		let mut right_dna: MaybeUninit<[u8; 32]> = MaybeUninit::uninit();

		let (left_indexes, right_indexes) = match breed_type {
			BreedType::DomDom => ((0..16, 16..32), (0..16, 16..16)),
			BreedType::DomRez => ((0..16, 16..32), (16..32, 0..16)),
			BreedType::RezDom => ((16..32, 0..16), (16..32, 0..16)),
			BreedType::RezRez => ((16..32, 0..16), (0..16, 16..32)),
		};

		unsafe {
			let l_dna_ptr = left_dna.as_mut_ptr() as *mut u8;
			let r_dna_ptr = right_dna.as_mut_ptr() as *mut u8;

			copy_nonoverlapping(left_source_dna[left_indexes.0].as_ptr(), l_dna_ptr, 16);
			copy_nonoverlapping(left_source_dna[left_indexes.1].as_ptr(), l_dna_ptr.add(16), 16);
			copy_nonoverlapping(right_source_dna[right_indexes.0].as_ptr(), r_dna_ptr, 16);
			copy_nonoverlapping(right_source_dna[right_indexes.1].as_ptr(), r_dna_ptr.add(16), 16);

			[left_dna.assume_init(), right_dna.assume_init()]
		}
	}

	pub fn segmenting(gen: [[u8; 32]; 2], blk: [u8; 32]) -> [[u8; 32]; 2] {
		let a_sec = &gen[0][0..32];
		let b_sec = &gen[1][0..32];

		//let a_x = &gen[0 ..  8];
		let a_y = &a_sec[16..32];
		let b_x = &b_sec[0..16];
		//let b_y = &gen[24 .. 32];

		let a_c = &a_y[0..8];
		let b_c = &b_x[0..8];

		let mut dna: [u8; 32] = Default::default();
		let mut evo: [u8; 32] = Default::default();

		let mut full: u8 = 0;
		let mut mark: u8 = 0;

		let mut mask_side = BitMaskSide::Left;

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

			if mask_side == BitMaskSide::Left {
				full = 0;
				mark = 0;
			}

			match (bit_a, bit_b) {
				(true, false) => {
					if blk_a {
						half_byte = Binary::copy_bits(half_byte, a_byte, mask_side); // A+ as 4
						half_byte = Binary::add_one(half_byte, mask_side);
						mark_byte = Binary::copy_bits(mark_byte, 0x44, mask_side);
					} else if !blk_b {
						half_byte = Binary::copy_bits(half_byte, a_byte, mask_side); // A as A
						mark_byte = Binary::copy_bits(mark_byte, 0xAA, mask_side);
					} else {
						half_byte = Binary::copy_bits(half_byte, a_byte ^ b_byte, mask_side); // A^B as 7
						mark_byte = Binary::copy_bits(mark_byte, 0x77, mask_side);
					}
				},
				(false, true) => {
					if blk_b {
						half_byte = Binary::copy_bits(half_byte, b_byte, mask_side); // 8
						mark_byte = Binary::copy_bits(mark_byte, 0x88, mask_side);
						half_byte = Binary::add_one(half_byte, mask_side);
					} else if !blk_a {
						half_byte = Binary::copy_bits(half_byte, b_byte, mask_side); // B
						mark_byte = Binary::copy_bits(mark_byte, 0xBB, mask_side);
					} else {
						half_byte = Binary::copy_bits(half_byte, b_byte ^ a_byte, mask_side); // A^B as 7
						mark_byte = Binary::copy_bits(mark_byte, 0x77, mask_side);
					}
				},
				(false, false) => {
					if !blk_a && !blk_b {
						if bit_a < bit_b {
							half_byte = Binary::copy_bits(half_byte, a_byte & !b_byte, mask_side); // !b- as 1
							half_byte = Binary::sub_one(half_byte, mask_side);
							mark_byte = Binary::copy_bits(mark_byte, 0x11, mask_side);
						} else {
							half_byte = Binary::copy_bits(half_byte, !a_byte & b_byte, mask_side); // !a- as 0
							mark_byte = Binary::copy_bits(mark_byte, 0x00, mask_side);
							half_byte = Binary::sub_one(half_byte, mask_side);
						}
					} else if blk_a && blk_b {
						half_byte = Binary::copy_bits(half_byte, !blk[i % 32], mask_side); // !blk as E
						mark_byte = Binary::copy_bits(mark_byte, 0xEE, mask_side);
					} else {
						if blk_a {
							half_byte = Binary::copy_bits(half_byte, a_byte, mask_side); // A
							mark_byte = Binary::copy_bits(mark_byte, 0xAA, mask_side);
						} else {
							half_byte = Binary::copy_bits(half_byte, b_byte, mask_side); // B
							mark_byte = Binary::copy_bits(mark_byte, 0xBB, mask_side);
						}
					}
				},
				(true, true) => {
					if blk_a && blk_b {
						half_byte = Binary::copy_bits(half_byte, a_byte | b_byte, mask_side); // |+ as C
						half_byte = Binary::add_one(half_byte, mask_side);
						mark_byte = Binary::copy_bits(mark_byte, 0xCC, mask_side);
					} else if !blk_a && !blk_b {
						half_byte = Binary::copy_bits(half_byte, blk[i % 32], mask_side); // blk as F
						mark_byte = Binary::copy_bits(mark_byte, 0xFF, mask_side);
					} else {
						if blk_a {
							half_byte = Binary::copy_bits(half_byte, a_byte, mask_side); // A
							mark_byte = Binary::copy_bits(mark_byte, 0xAA, mask_side);
						} else {
							half_byte = Binary::copy_bits(half_byte, b_byte, mask_side); // B
							mark_byte = Binary::copy_bits(mark_byte, 0xBB, mask_side);
						}
					}
				},
			}

			full = Binary::copy_bits(full, half_byte, mask_side);
			mark = Binary::copy_bits(mark, mark_byte, mask_side);

			// recombination
			if mask_side == BitMaskSide::Right {
				if full == 0xFF || full == 0x00 {
					full &= blk[i % 32];
					mark = 0x33;
				}
				dna[i / 2] = full;
				evo[i / 2] = mark;
			}

			mask_side = mask_side.flip();
		}

		[dna, evo]
	}

	pub fn bake(rarity: RarityType, blk: [u8; 32]) -> RarityType {
		let prob: u16 = 250;

		let rarity = rarity as u8;

		let mut result = rarity & Binary::RIGHT_BITMASK;
		let max_rarity = rarity >> 4;

		let mut rand: [u16; 16] = Default::default();
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

		RarityType::from(result)
	}
}

pub struct Generation;

impl Generation {
	fn compute_next_generation_and_rarity(
		generation: MogwaiGeneration,
		rarity: RarityType,
		hash: &[u8; 6],
	) -> (RarityType, MogwaiGeneration) {
		let generation = generation as u16;
		let rarity = (rarity as u16).saturating_sub(1);

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
		input_generation_1: MogwaiGeneration,
		input_rarity_1: RarityType,
		input_generation_2: MogwaiGeneration,
		input_rarity_2: RarityType,
		random_hash: &[u8],
	) -> (RarityType, MogwaiGeneration, RarityType) {
		let mut resulting_gen = MogwaiGeneration::default();
		let mut resulting_rarity = RarityType::default();

		if random_hash.len() >= 12 {
			let base_rarity = (input_rarity_1 as u16 + input_rarity_2 as u16).saturating_sub(2) / 2;

			let slice = unsafe { &*(&random_hash[0..6] as *const [u8] as *const [u8; 6]) };
			let (out_rarity_1, out_gen_1) =
				Self::compute_next_generation_and_rarity(input_generation_1, input_rarity_1, slice);

			let slice = unsafe { &*(&random_hash[6..12] as *const [u8] as *const [u8; 6]) };
			let (out_rarity_2, out_gen_2) =
				Self::compute_next_generation_and_rarity(input_generation_2, input_rarity_2, slice);

			resulting_gen = MogwaiGeneration::coerce_from(
				(out_gen_1 as u16 + out_gen_2 as u16 + base_rarity) / 2,
			);

			resulting_rarity = RarityType::from(
				((out_rarity_1 as u16 +
					out_rarity_2 as u16 + ((input_rarity_1 as u16 + input_rarity_2 as u16) / 2)) /
					2) % 5,
			)
		}

		let max_rarity = RarityType::from(
			(6 + ((input_rarity_1 as u16 + input_rarity_2 as u16) / 2 as u16) / 2) % 5,
		);

		(resulting_rarity, resulting_gen, max_rarity)
	}
}
