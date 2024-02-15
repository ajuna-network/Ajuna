mod assemble;
mod breed;
mod build;
mod equip;
mod feed;
mod flask;
mod glimmer;
mod mate;
mod spark;
mod stack;
mod statue;
mod tinker;

use super::*;
pub(super) struct AvatarCombinator<T: Config>(pub PhantomData<T>);

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn combine_avatars_in(
		forge_type: ForgeType,
		season_id: SeasonId,
		_season: &SeasonOf<T>,
		leader: ForgeItem<T>,
		sacrifices: Vec<ForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		match forge_type {
			ForgeType::Stack => Self::stack_avatars(leader, sacrifices, season_id, hash_provider),
			ForgeType::Tinker => Self::tinker_avatars(leader, sacrifices, season_id),
			ForgeType::Build => Self::build_avatars(leader, sacrifices, season_id, hash_provider),
			ForgeType::Assemble => Self::assemble_avatars(leader, sacrifices, hash_provider),
			ForgeType::Breed => Self::breed_avatars(leader, sacrifices, hash_provider),
			ForgeType::Equip => Self::equip_avatars(leader, sacrifices),
			ForgeType::Mate => Self::mate_avatars(
				leader,
				sacrifices,
				season_id,
				hash_provider,
				<frame_system::Pallet<T>>::block_number(),
			),
			ForgeType::Feed => Self::feed_avatars(leader, sacrifices),
			ForgeType::Glimmer => {
				Self::glimmer_avatars(leader, sacrifices, season_id, hash_provider)
			},
			ForgeType::Spark => Self::spark_avatars(leader, sacrifices, hash_provider),
			ForgeType::Flask => Self::flask_avatars(leader, sacrifices, hash_provider),
			ForgeType::Statue => Self::statue_avatars(
				leader,
				sacrifices,
				season_id,
				hash_provider,
				<frame_system::Pallet<T>>::block_number(),
			),
			ForgeType::None => Self::forge_none(leader, sacrifices),
		}
	}

	fn forge_none(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		Ok((
			LeaderForgeOutput::Forged(input_leader, 0),
			input_sacrifices
				.into_iter()
				.map(|sacrifice| ForgeOutput::Forged(sacrifice, 0))
				.collect(),
		))
	}

	fn match_avatars(
		input_leader: ForgeItem<T>,
		sacrifices: Vec<ForgeItem<T>>,
		rarity_level: u8,
		hash_provider: &mut HashProvider<T, 32>,
	) -> (ForgeItem<T>, Vec<ForgeItem<T>>) {
		let (leader_id, mut leader) = input_leader;

		let mut matches: u8 = 0;
		let mut no_fit: u8 = 0;
		let mut matching_score = Vec::new();

		let mut leader_progress_array = AvatarUtils::read_progress_array(&leader);

		for sacrifice in sacrifices.iter() {
			let sacrifice_progress_array = AvatarUtils::read_progress_array(&sacrifice.1);

			if let Some(matched_indexes) = AvatarUtils::is_array_match(
				leader_progress_array,
				sacrifice_progress_array,
				rarity_level,
			) {
				matching_score.extend(matched_indexes);
				matches += 1;
			} else {
				no_fit += 1;
			}
		}

		if !matching_score.is_empty() {
			let rolls = matches + no_fit;

			let match_probability =
				(SCALING_FACTOR_PERC - BASE_PROGRESS_PROB_PERC) / MAX_SACRIFICE as u32;
			let probability_match = BASE_PROGRESS_PROB_PERC + (matches as u32 * match_probability);

			for i in 0..rolls as usize {
				let hash_index = hash_provider.hash[i] as usize;
				let random_hash = hash_provider.hash[hash_index % 32];

				if (random_hash as u32 * SCALING_FACTOR_PERC) <= (probability_match * MAX_BYTE) {
					let pos = matching_score[hash_index % matching_score.len()];

					leader_progress_array[pos as usize] += 0x10; // 16

					matching_score.retain(|item| *item != pos);
					if matching_score.is_empty() {
						break;
					}
				}
			}

			AvatarUtils::write_progress_array(&mut leader, leader_progress_array);
		}

		leader.souls += sacrifices.iter().map(|(_, sacrifice)| sacrifice.souls).sum::<SoulCount>();

		((leader_id, leader), sacrifices)
	}
}

#[cfg(test)]
mod match_test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_match_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let hash_bytes = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(hash_bytes);
			let unit_closure = |avatar| avatar;

			let test_set: Vec<([u8; 32], [u8; 32], usize, (usize, u8), [u8; 11])> = vec![
				(
					[
						0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					],
					[
						0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					],
					1,
					(0, 0x10),
					[0_u8; 11],
				),
				(
					[
						0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x10,
						0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x01, 0x00,
					],
					[
						0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11,
						0x11, 0x12, 0x13, 0x14, 0x01, 0x05, 0x00, 0x00,
					],
					1,
					(1, 0x20),
					[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x01, 0x00],
				),
				(
					[
						0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x10,
						0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00,
					],
					[
						0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x11,
						0x12, 0x13, 0x14, 0x15, 0x01, 0x05, 0x00, 0x00,
					],
					1,
					(1, 0x20),
					[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00],
				),
				(
					[
						0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x10,
						0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00,
					],
					[
						0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x10,
						0x10, 0x13, 0x14, 0x15, 0x01, 0x00, 0x00, 0x00,
					],
					1,
					(1, 0x20),
					[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00],
				),
				(
					[
						0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10, 0x10,
						0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00,
					],
					[
						0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11,
						0x11, 0x13, 0x14, 0x15, 0x01, 0x00, 0x00, 0x00,
					],
					1,
					(0, 0x10),
					[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00, 0x00],
				),
			];

			for (i, (dna_1, dna_2, sac_len, (top_bit_index, top_1_bit), progress_array_1)) in
				test_set.into_iter().enumerate()
			{
				let avatar_1 =
					create_random_avatar::<Test, _>(&ALICE, Some(dna_1), Some(unit_closure));
				let avatar_2 =
					create_random_avatar::<Test, _>(&ALICE, Some(dna_2), Some(unit_closure));

				let ((_, leader), sacrifices) = AvatarCombinator::<Test>::match_avatars(
					avatar_1,
					vec![avatar_2],
					0,
					&mut hash_provider,
				);

				assert_eq!(sacrifices.len(), sac_len, "Test matched_avatars len for case {}", i);

				if top_bit_index == 0 {
					assert_eq!(leader.dna[0], top_1_bit, "Test top_bit for case {}", i);
				} else {
					assert_eq!(
						sacrifices[top_bit_index - 1].1.dna[0],
						top_1_bit,
						"Test top_bit for case {}",
						i
					);
				}

				assert_eq!(
					AvatarUtils::read_progress_array(&leader),
					progress_array_1,
					"Test progress_array for case {}",
					i
				);
			}
		});
	}
}
