use super::*;

impl<'a, T> AvatarCombinator<'a, T>
where
	T: Config,
{
	pub(super) fn breed_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (mut input_leader, matching_sacrifices, consumed_sacrifices, non_matching_sacrifices) =
			Self::match_avatars(input_leader, input_sacrifices, hash_provider);

		let rarity_type = RarityType::from_byte(AvatarUtils::read_lowest_progress_byte(
			&AvatarUtils::read_progress_array(&input_leader.1),
			ByteType::High,
		));

		let is_leader_legendary = rarity_type == RarityType::Legendary;
		let is_leader_egg = AvatarUtils::has_attribute_set_with_values(
			&input_leader.1,
			&[
				(AvatarAttributes::ItemType, ItemType::Pet.as_byte()),
				(AvatarAttributes::ItemSubType, PetItemType::Egg.as_byte()),
			],
		);
		let pet_variation =
			AvatarUtils::read_attribute(&input_leader.1, AvatarAttributes::CustomType2);

		if is_leader_legendary && is_leader_egg && pet_variation > 0 {
			let pet_type_list = AvatarUtils::bits_to_enums::<PetType>(pet_variation as u32);
			let pet_type = &pet_type_list[hash_provider.hash[0] as usize % pet_type_list.len()];

			AvatarUtils::write_typed_attribute(
				&mut input_leader.1,
				AvatarAttributes::ClassType2,
				pet_type.clone(),
			);

			AvatarUtils::write_typed_attribute(
				&mut input_leader.1,
				AvatarAttributes::ItemSubType,
				PetItemType::Pet,
			);
		}

		AvatarUtils::write_typed_attribute(
			&mut input_leader.1,
			AvatarAttributes::RarityType,
			rarity_type,
		);

		let output_vec: Vec<ForgeOutput<T>> = non_matching_sacrifices
			.into_iter()
			.map(|sacrifice| ForgeOutput::Forged(sacrifice, 0))
			.chain(
				consumed_sacrifices
					.into_iter()
					.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id)),
			)
			.chain(
				matching_sacrifices
					.into_iter()
					.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id)),
			)
			.collect();

		Ok((LeaderForgeOutput::Forged(input_leader, 0), output_vec))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_breed_egg_prep_1() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let hash_base = [
				[
					0x13, 0x00, 0x04, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42, 0x40, 0x40, 0x44, 0x43,
					0x42, 0x41, 0x44, 0x44, 0x42, 0x45,
				],
				[
					0x13, 0x00, 0x04, 0x01, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41, 0x51, 0x52, 0x53, 0x44,
					0x52, 0x45, 0x41, 0x40, 0x41, 0x43,
				],
				[
					0x13, 0x00, 0x04, 0x01, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x52, 0x41, 0x43, 0x41, 0x53,
					0x45, 0x43, 0x44, 0x52, 0x43, 0x43,
				],
				[
					0x13, 0x00, 0x04, 0x01, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42, 0x43, 0x43, 0x44, 0x42,
					0x44, 0x54, 0x45, 0x41, 0x45, 0x40,
				],
				[
					0x13, 0x00, 0x04, 0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43, 0x43, 0x52, 0x41, 0x42,
					0x42, 0x40, 0x45, 0x43, 0x52, 0x44,
				],
			];

			let unit_closure = |avatar| avatar;

			let mut avatar_set = (0..5)
				.into_iter()
				.map(|i| {
					create_random_avatar::<Test, _>(&ALICE, Some(hash_base[i]), Some(unit_closure))
				})
				.collect::<Vec<_>>();

			let sacrifices = avatar_set.split_off(1);
			let leader = avatar_set.pop().unwrap();

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::breed_avatars(leader, sacrifices, &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 3);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				let expected_dna = [
					0x13, 0x00, 0x04, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x52, 0x40, 0x40, 0x44, 0x53,
					0x42, 0x51, 0x54, 0x44, 0x42, 0x45,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_breed_egg_prep_2() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x9A, 0x6D, 0x5D, 0x62, 0x1B, 0x32, 0xFF, 0x42, 0x32, 0x46, 0x62, 0x15, 0xBB, 0x51,
				0xE9, 0x37, 0xDB, 0xB0, 0xBC, 0x0F, 0xB0, 0x4C, 0xFF, 0x14, 0x40, 0x99, 0xEF, 0x6C,
				0x23, 0xAF, 0xCF, 0x4E,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let progress_arrays = [
				[0x43, 0x44, 0x41, 0x43, 0x45, 0x44, 0x44, 0x41, 0x43, 0x41, 0x43],
				[0x53, 0x40, 0x41, 0x41, 0x43, 0x44, 0x50, 0x42, 0x45, 0x40, 0x41],
				[0x45, 0x44, 0x50, 0x45, 0x43, 0x43, 0x45, 0x43, 0x43, 0x41, 0x40],
				[0x43, 0x43, 0x40, 0x41, 0x52, 0x45, 0x41, 0x40, 0x53, 0x42, 0x44],
				[0x43, 0x40, 0x44, 0x43, 0x41, 0x45, 0x44, 0x44, 0x44, 0x45, 0x42],
			];

			let mut egg_set = (0..5)
				.into_iter()
				.map(|i| {
					let soul_points = ((progress_arrays[i][2] | progress_arrays[i][6]) % 99) + 1;

					create_random_egg(
						None,
						&ALICE,
						RarityType::Epic,
						0b0000_1111,
						soul_points as SoulCount,
						progress_arrays[i],
					)
				})
				.collect::<Vec<_>>();

			let sacrifices = egg_set.split_off(1);
			let leader = egg_set.pop().unwrap();

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::breed_avatars(leader, sacrifices, &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 2);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				let expected_progress_array =
					[0x43, 0x44, 0x51, 0x43, 0x45, 0x44, 0x44, 0x51, 0x43, 0x41, 0x43];
				assert_eq!(AvatarUtils::read_progress_array(&avatar), expected_progress_array);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_breed_egg_prep_3() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let progress_arrays = [
				[0x54, 0x55, 0x43, 0x50, 0x50, 0x41, 0x41, 0x54, 0x54, 0x43, 0x52],
				[0x42, 0x55, 0x42, 0x50, 0x43, 0x43, 0x45, 0x45, 0x44, 0x50, 0x42],
				[0x44, 0x40, 0x44, 0x44, 0x53, 0x41, 0x40, 0x40, 0x54, 0x43, 0x45],
				[0x42, 0x41, 0x44, 0x40, 0x53, 0x41, 0x43, 0x44, 0x42, 0x42, 0x42],
				[0x54, 0x43, 0x44, 0x42, 0x45, 0x42, 0x41, 0x44, 0x40, 0x51, 0x41],
			];

			let mut egg_set = (0..5)
				.into_iter()
				.map(|i| {
					let soul_points = ((progress_arrays[i][2] | progress_arrays[i][6]) % 99) + 1;

					create_random_egg(
						None,
						&ALICE,
						RarityType::Epic,
						0b0000_1111,
						soul_points as SoulCount,
						progress_arrays[i],
					)
				})
				.collect::<Vec<_>>();

			let sacrifices = egg_set.split_off(1);
			let leader = egg_set.pop().unwrap();

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::breed_avatars(leader, sacrifices, &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				let expected_progress_array =
					[0x54, 0x55, 0x53, 0x50, 0x50, 0x51, 0x51, 0x54, 0x54, 0x53, 0x52];
				assert_eq!(AvatarUtils::read_progress_array(&avatar), expected_progress_array);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	// TODO: Need original test rewrite
	/*#[test]
	fn test_breed_egg() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let rarity_type = RarityType::Epic;

			let mut egg_set = (0..5)
				.into_iter()
				.map(|i| {
					let soul_points = (i + 1) * 10;
					let pet_type =
						SlotRoller::<Test>::roll_on(&PET_TYPE_PROBABILITIES, &mut hash_provider);
					let progress_array = AvatarUtils::generate_progress_bytes(
						rarity_type,
						SCALING_FACTOR_PERC,
						PROGRESS_PROBABILITY_PERC,
						[i; 11],
					);

					create_random_egg(
						None,
						&ALICE,
						rarity_type,
						pet_type,
						soul_points as SoulCount,
						progress_array,
					)
				})
				.collect::<Vec<_>>();

			let total_soul_points =
				egg_set.iter().map(|(_, avatar)| avatar.souls).sum::<SoulCount>();
			assert_eq!(total_soul_points, 150);

			let sacrifices = egg_set.split_off(1);
			let leader = egg_set.pop().unwrap();

			assert_eq!(AvatarUtils::read_attribute(&leader.1, AvatarAttributes::CustomType2), 2);
			let expected_progress_array =
				[0x41, 0x40, 0x45, 0x43, 0x40, 0x53, 0x41, 0x42, 0x42, 0x52, 0x44];
			assert_eq!(AvatarUtils::read_progress_array(&leader.1), expected_progress_array);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::breed_avatars(
				leader,
				sacrifices,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(total_soul_points, avatar.souls);

				let expected_progress_array =
					[0x41, 0x40, 0x45, 0x43, 0x40, 0x53, 0x41, 0x42, 0x42, 0x52, 0x44];
				assert_eq!(AvatarUtils::read_progress_array(&avatar), expected_progress_array);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}

			// MORE
		});
	}*/
}
