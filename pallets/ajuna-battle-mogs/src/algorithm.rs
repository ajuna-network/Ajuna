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
			.then_some(input_generation_1.saturating_sub(input_generation_2))
			.unwrap_or_default();

		let input_rarity_1 = input_rarity_1 as u16;
		let input_rarity_2 = input_rarity_2 as u16;

		let rarity_diff = (input_rarity_2 > input_rarity_1)
			.then_some(input_rarity_2.saturating_sub(input_rarity_1))
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

				if prob < 256 {
					prob as u8
				} else {
					u8::MAX
				}
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

	pub fn segmenting(input_dna: [[u8; 32]; 2], block_hash: [u8; 32]) -> [[u8; 32]; 2] {
		let stats_segment = &input_dna[0];
		let visuals_segment = &input_dna[1];

		let _stats_segment_1 = &stats_segment[0..16]; // Unused for now
		let stats_segment_2 = &stats_segment[16..32];
		let visuals_segment_1 = &visuals_segment[0..16];
		let _visuals_segment_2 = &visuals_segment[16..32]; // Unused for now

		let stats_segment_2_1 = &stats_segment_2[0..8];
		let visuals_segment_1_1 = &visuals_segment_1[0..8];

		let mut output_stats: [u8; 32] = Default::default();
		let mut output_visuals: [u8; 32] = Default::default();

		let mut stats_byte: u8 = 0;
		let mut visuals_byte: u8 = 0;

		let mut mask_side = BitMaskSide::Left;

		for i in 0..64 {
			let bit_index = (i % 8) as u8;
			let byte_index = i / 8;
			let j = 63 - i;

			let stats_bit = Binary::get_bit_at(stats_segment_2_1[byte_index], bit_index);
			let visuals_bit = Binary::get_bit_at(visuals_segment_1_1[byte_index], bit_index);

			let block_hash_bit_1 = Binary::get_bit_at(block_hash[byte_index], bit_index);
			let block_hash_bit_2 = Binary::get_bit_at(block_hash[j / 8], (j % 8) as u8);

			let half_i = i / 2;
			let mut stats_half_byte: u8 = output_stats[half_i];
			let mut visuals_half_byte: u8 = output_visuals[half_i];

			let stats_segment_byte = stats_segment[half_i];
			let visuals_segment_byte = visuals_segment[half_i];

			if mask_side == BitMaskSide::Left {
				stats_byte = 0;
				visuals_byte = 0;
			}

			match (stats_bit, visuals_bit) {
				(true, false) => {
					if block_hash_bit_1 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, stats_segment_byte, mask_side); // A+ as 4
						stats_half_byte = Binary::add_one(stats_half_byte, mask_side);
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0x44, mask_side);
					} else if !block_hash_bit_2 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, stats_segment_byte, mask_side); // A as A
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xAA, mask_side);
					} else {
						stats_half_byte = Binary::copy_bits(
							stats_half_byte,
							stats_segment_byte ^ visuals_segment_byte,
							mask_side,
						); // A^B as 7
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0x77, mask_side);
					}
				},
				(false, true) => {
					if block_hash_bit_2 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, visuals_segment_byte, mask_side); // 8
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0x88, mask_side);
						stats_half_byte = Binary::add_one(stats_half_byte, mask_side);
					} else if !block_hash_bit_1 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, visuals_segment_byte, mask_side); // B
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xBB, mask_side);
					} else {
						stats_half_byte = Binary::copy_bits(
							stats_half_byte,
							visuals_segment_byte ^ stats_segment_byte,
							mask_side,
						); // A^B as 7
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0x77, mask_side);
					}
				},
				(false, false) => {
					if !block_hash_bit_1 && !block_hash_bit_2 {
						if !stats_bit & visuals_bit {
							stats_half_byte = Binary::copy_bits(
								stats_half_byte,
								stats_segment_byte & !visuals_segment_byte,
								mask_side,
							); // !b- as 1
							stats_half_byte = Binary::sub_one(stats_half_byte, mask_side);
							visuals_half_byte =
								Binary::copy_bits(visuals_half_byte, 0x11, mask_side);
						} else {
							stats_half_byte = Binary::copy_bits(
								stats_half_byte,
								!stats_segment_byte & visuals_segment_byte,
								mask_side,
							); // !a- as 0
							visuals_half_byte =
								Binary::copy_bits(visuals_half_byte, 0x00, mask_side);
							stats_half_byte = Binary::sub_one(stats_half_byte, mask_side);
						}
					} else if block_hash_bit_1 && block_hash_bit_2 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, !block_hash[i % 32], mask_side); // !blk as E
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xEE, mask_side);
					} else if block_hash_bit_1 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, stats_segment_byte, mask_side); // A
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xAA, mask_side);
					} else {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, visuals_segment_byte, mask_side); // B
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xBB, mask_side);
					}
				},
				(true, true) => {
					if block_hash_bit_1 && block_hash_bit_2 {
						stats_half_byte = Binary::copy_bits(
							stats_half_byte,
							stats_segment_byte | visuals_segment_byte,
							mask_side,
						); // |+ as C
						stats_half_byte = Binary::add_one(stats_half_byte, mask_side);
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xCC, mask_side);
					} else if !block_hash_bit_1 && !block_hash_bit_2 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, block_hash[i % 32], mask_side); // blk as F
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xFF, mask_side);
					} else if block_hash_bit_1 {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, stats_segment_byte, mask_side); // A
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xAA, mask_side);
					} else {
						stats_half_byte =
							Binary::copy_bits(stats_half_byte, visuals_segment_byte, mask_side); // B
						visuals_half_byte = Binary::copy_bits(visuals_half_byte, 0xBB, mask_side);
					}
				},
			}

			stats_byte = Binary::copy_bits(stats_byte, stats_half_byte, mask_side);
			visuals_byte = Binary::copy_bits(visuals_byte, visuals_half_byte, mask_side);

			// recombination
			if mask_side == BitMaskSide::Right {
				if stats_byte == 0xFF || stats_byte == 0x00 {
					stats_byte &= block_hash[i % 32];
					visuals_byte = 0x33;
				}
				output_stats[i / 2] = stats_byte;
				output_visuals[i / 2] = visuals_byte;
			}

			mask_side = mask_side.flip();
		}

		[output_stats, output_visuals]
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

#[cfg(test)]
mod test {
	use super::*;

	mod segmenting {
		use super::*;

		#[test]
		fn segmenting_consistency_test_1() {
			let dna: [[u8; 32]; 2] = [[0; 32]; 2];
			let hash: [u8; 32] = [0; 32];

			let expected_output = [[0; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_2() {
			let dna: [[u8; 32]; 2] = [[0; 32]; 2];
			let hash: [u8; 32] = [0xFF; 32];

			let expected_output = [[0; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_3() {
			let dna: [[u8; 32]; 2] = [[0xFF; 32]; 2];
			let hash: [u8; 32] = [0; 32];

			let expected_output = [[0; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_4() {
			let dna: [[u8; 32]; 2] = [[0xFF; 32]; 2];
			let hash: [u8; 32] = [0xFF; 32];

			let expected_output = [[0xFF; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_5() {
			let dna: [[u8; 32]; 2] = [[0xFF; 32], [0; 32]];
			let hash: [u8; 32] = [0; 32];

			let expected_output = [[0; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_6() {
			let dna: [[u8; 32]; 2] = [[0xFF; 32], [0; 32]];
			let hash: [u8; 32] = [0xFF; 32];

			let expected_output = [[0xFF; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_7() {
			let dna: [[u8; 32]; 2] = [[0; 32], [0xFF; 32]];
			let hash: [u8; 32] = [0; 32];

			let expected_output = [[0; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_consistency_test_8() {
			let dna: [[u8; 32]; 2] = [[0; 32], [0xFF; 32]];
			let hash: [u8; 32] = [0xFF; 32];

			let expected_output = [[0xFF; 32], [0x33; 32]];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}

		#[test]
		fn segmenting_mogwai_test_1() {
			let dna = [
				[
					0x93, 0x9D, 0xCA, 0xDD, 0x21, 0xB0, 0x15, 0x09, 0xD9, 0x8E, 0x69, 0x35, 0xA3,
					0xED, 0xA0, 0x5C, 0xC5, 0x2B, 0x91, 0x9F, 0x68, 0x34, 0xBE, 0x48, 0x2D, 0x0A,
					0x58, 0x23, 0xA3, 0xB2, 0x41, 0xCD,
				],
				[
					0x0A, 0xC0, 0xEF, 0x2F, 0xF6, 0xF5, 0x23, 0xBE, 0xEA, 0xAE, 0x29, 0x53, 0x27,
					0x45, 0xAD, 0x27, 0x46, 0x34, 0x30, 0xA8, 0x36, 0xEF, 0xCA, 0x51, 0x7B, 0xCE,
					0xC9, 0x51, 0x79, 0xD9, 0xC0, 0x0A,
				],
			];

			let hash = [
				0x91, 0x2B, 0xCA, 0x9F, 0x21, 0xB0, 0x48, 0x09, 0x34, 0x8E, 0x69, 0x35, 0xA3, 0xED,
				0x23, 0x5C, 0xC5, 0xA0, 0x91, 0x9F, 0x68, 0x8E, 0x0A, 0x48, 0x2D, 0x0A, 0x58, 0x23,
				0xA3, 0x68, 0x41, 0xCD,
			];

			let expected_output = [
				[
					0xAA, 0x90, 0xD4, 0xDE, 0x32, 0x31, 0x26, 0xBE, 0xE3, 0xBF, 0x69, 0x65, 0xA3,
					0xFD, 0xBD, 0x1D, 0x66, 0x3F, 0x42, 0xA9, 0x46, 0x6A, 0xFE, 0x69, 0x70, 0xCB,
					0x9E, 0xF2, 0xD9, 0xDC, 0xC0, 0xCB,
				],
				[
					0x4B, 0xAB, 0xE0, 0xA4, 0x44, 0x04, 0xB4, 0xBB, 0xB7, 0x88, 0xAB, 0x7A, 0xAA,
					0xCA, 0x4B, 0x04, 0xEB, 0xB7, 0x8C, 0xB8, 0x8B, 0xF0, 0xCA, 0x87, 0xBF, 0x74,
					0x7F, 0xE7, 0xEB, 0xBC, 0xBB, 0xA8,
				],
			];
			let output_dna = Breeding::segmenting(dna, hash);

			assert_eq!(output_dna, expected_output);
		}
	}
}
