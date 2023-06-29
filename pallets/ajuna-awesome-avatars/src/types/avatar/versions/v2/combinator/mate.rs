use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn mate_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
		_block_number: T::BlockNumber,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		if input_sacrifices.len() != 1 {
			return Ok((
				LeaderForgeOutput::Forged((input_leader.0, input_leader.1.unwrap()), 0),
				input_sacrifices
					.into_iter()
					.map(|(id, sacrifice)| ForgeOutput::Forged((id, sacrifice.unwrap()), 0))
					.collect(),
			))
		}

		let (leader_id, mut leader) = input_leader;
		let (partner_id, mut partner) = {
			let mut input_sacrifices = input_sacrifices;
			input_sacrifices.pop().unwrap()
		};

		if leader.spec_byte_split_ten_count() < MAX_EQUIPPED_SLOTS {
			return Ok((
				LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0),
				vec![ForgeOutput::Forged((partner_id, partner.unwrap()), 0)],
			))
		}
		if partner.spec_byte_split_ten_count() < MAX_EQUIPPED_SLOTS {
			leader.add_souls(partner.get_souls());

			return Ok((
				LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0),
				vec![ForgeOutput::Consumed(partner_id)],
			))
		}

		let (mirrors, _) = DnaUtils::match_progress(
			leader.get_progress(),
			partner.get_progress(),
			RarityTier::Common.as_byte(),
		);

		if mirrors.len() < 4 {
			leader.add_souls(partner.get_souls());

			return Ok((
				LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0),
				vec![ForgeOutput::Consumed(partner_id)],
			))
		}

		let leader_pet_type =
			DnaUtils::enums_to_bits(&[leader.get_class_type_2::<PetType>()]) as u8;

		let leader_pet_variation = leader.get_custom_type_2::<u8>();
		let partner_pet_variation = partner.get_custom_type_2::<u8>();

		let legendary_egg_flag = ((hash_provider.hash[0] | hash_provider.hash[1]) == 0x7F) &&
			((leader_pet_variation + partner_pet_variation) % 42) == 0;

		let random_pet_variation = hash_provider.hash[0] & hash_provider.hash[1] & 0b0111_1111;

		let pet_variation = if !legendary_egg_flag {
			(leader_pet_variation ^ partner_pet_variation) | leader_pet_type | random_pet_variation
		} else {
			0x00
		};

		let soul_points = (hash_provider.hash[0] | hash_provider.hash[1] & 0x7F) as SoulCount;

		let (leader_output, other_output, any_survived) = if partner.get_souls() > soul_points {
			partner.dec_souls(soul_points);
			(
				LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0),
				vec![ForgeOutput::Forged((partner_id, partner.unwrap()), 0)],
				true,
			)
		} else if leader.get_souls() + partner.get_souls() > soul_points {
			leader.dec_souls(soul_points - partner.get_souls());
			(
				LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0),
				vec![ForgeOutput::Consumed(partner_id)],
				true,
			)
		} else {
			let generated_dust = {
				let dna = MinterV2::<T>::generate_empty_dna::<32>()?;

				AvatarBuilder::with_dna(season_id, dna)
					.into_dust(leader.get_souls() + partner.get_souls())
					.build()
			};
			(
				LeaderForgeOutput::Consumed(leader_id),
				vec![ForgeOutput::Consumed(partner_id), ForgeOutput::Minted(generated_dust)],
				false,
			)
		};

		let additional_output = any_survived
			.then(|| {
				let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
				let progress_array = DnaUtils::generate_progress(
					&RarityTier::Rare,
					SCALING_FACTOR_PERC,
					Some(PROGRESS_PROBABILITY_PERC),
					hash_provider,
				);
				let generated_egg = AvatarBuilder::with_dna(season_id, dna)
					.into_egg(&RarityTier::Rare, pet_variation, soul_points, progress_array)
					.build();
				Ok::<_, DispatchError>(ForgeOutput::Minted(generated_egg))
			})
			.transpose()?;

		Ok((leader_output, other_output.into_iter().chain(additional_output.into_iter()).collect()))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;
	use sp_std::collections::btree_map::BTreeMap;

	#[test]
	fn test_mate_extra_sacrifices() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let leader =
				create_random_pet(&ALICE, &PetType::BigHybrid, 0b0001_1001, [0; 16], [0; 11], 1000);
			let partner =
				create_random_pet(&ALICE, &PetType::CrazyDude, 0b0101_0011, [0; 16], [0; 11], 1000);
			let extra_partner =
				create_random_pet(&ALICE, &PetType::CrazyDude, 0b0101_0011, [0; 16], [0; 11], 1000);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
				leader,
				vec![partner, extra_partner],
				0,
				&mut hash_provider,
				1,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 2);

			assert!(is_leader_forged(&leader_output));
		});
	}

	#[test]
	fn test_mate_all_equipped_slots_leader() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let leader =
				create_random_pet(&ALICE, &PetType::BigHybrid, 0b0001_1001, [0; 16], [0; 11], 1000);
			let partner =
				create_random_pet(&ALICE, &PetType::CrazyDude, 0b0101_0011, [0; 16], [0; 11], 1000);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
				leader,
				vec![partner],
				0,
				&mut hash_provider,
				1,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);

			assert!(is_leader_forged(&leader_output));
		});
	}

	#[test]
	fn test_mate_all_equipped_slots_partner() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let leader = create_random_pet(
				&ALICE,
				&PetType::BigHybrid,
				0b0001_1001,
				[0xFF; 16],
				[0; 11],
				1000,
			);
			let partner =
				create_random_pet(&ALICE, &PetType::CrazyDude, 0b0101_0011, [0; 16], [0; 11], 1000);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
				leader,
				vec![partner],
				0,
				&mut hash_provider,
				1,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);

			assert!(is_leader_forged(&leader_output));
		});
	}

	#[test]
	fn test_mate() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let partner_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x54, 0x50, 0x50, 0x53, 0x55, 0x53, 0x51];
			let partner_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let leader = create_random_pet(
				&ALICE,
				&PetType::BigHybrid,
				0b0001_1001,
				leader_spec_bytes,
				leader_progress_array,
				1000,
			);
			let partner = create_random_pet(
				&ALICE,
				&PetType::CrazyDude,
				0b0101_0011,
				partner_spec_bytes,
				partner_progress_array,
				1000,
			);

			let expected_dna = [
				0x11, 0x05, 0x05, 0x01, 0x19, 0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x09, 0x75, 0x97, 0x50, 0x00, 0x00, 0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54,
				0x51, 0x53, 0x55, 0x53,
			];
			assert_eq!(leader.1.get_dna().as_slice(), &expected_dna);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
				leader,
				vec![partner],
				0,
				&mut hash_provider,
				1,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 1000);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}

			if let ForgeOutput::Forged((_, avatar), _) = &sacrifice_output[0] {
				assert_eq!(avatar.souls, 878);
			} else {
				panic!("ForgeOutput for first output should have been Forged!")
			}

			if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
				assert_eq!(avatar.souls, 122);
				assert_eq!(
					DnaUtils::read_attribute::<PetItemType>(avatar, AvatarAttr::ItemSubType),
					PetItemType::Egg
				);
				assert_eq!(
					DnaUtils::read_attribute_raw(avatar, AvatarAttr::CustomType2),
					0b0101_1010
				);
			} else {
				panic!("ForgeOutput for second output should have been Minted!")
			}
		});
	}

	#[test]
	fn test_mate_egg_distribution_1() {
		ExtBuilder::default().build().execute_with(|| {
			let hash = Pallet::<Test>::random_hash(b"mate_dist_1", &ALICE);
			let mut hash_provider: HashProvider<Test, 32> = HashProvider::new(&hash);

			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let partner_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x54, 0x50, 0x50, 0x53, 0x55, 0x53, 0x51];
			let partner_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let loop_count = 100_000_u32;
			let mut distribution_map = BTreeMap::new();

			for i in 0..loop_count {
				let leader = create_random_pet(
					&ALICE,
					&PetType::FoxishDude,
					(hash_provider.next() % 128) + 1,
					leader_spec_bytes,
					leader_progress_array,
					1000,
				);
				let partner = create_random_pet(
					&ALICE,
					&PetType::FoxishDude,
					(hash_provider.next() % 128) + 1,
					partner_spec_bytes,
					partner_progress_array,
					1000,
				);

				let (_, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
					leader,
					vec![partner],
					0,
					&mut hash_provider,
					1,
				)
				.expect("Should succeed in forging");

				assert_eq!(sacrifice_output.len(), 2);
				assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
				assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					assert_eq!(
						DnaUtils::read_attribute::<PetItemType>(avatar, AvatarAttr::ItemSubType),
						PetItemType::Egg
					);

					let custom_type_2 =
						DnaUtils::read_attribute_raw(avatar, AvatarAttr::CustomType2);

					distribution_map
						.entry(custom_type_2)
						.and_modify(|value| *value += 1)
						.or_insert(1_u32);
				} else {
					panic!("ForgeOutput for second output should have been Minted!")
				}

				if i % 1000 == 999 {
					let hash_text = format!("loop_{:#07X}", i);
					let hash = Pallet::<Test>::random_hash(hash_text.as_bytes(), &ALICE);
					hash_provider = HashProvider::new(&hash);
				}
			}

			assert_eq!(distribution_map.get(&0b0000_0000).unwrap(), &94_u32);

			assert_eq!(distribution_map.get(&0b0000_0011).unwrap(), &468_u32);
			assert_eq!(distribution_map.get(&0b0000_0111).unwrap(), &439_u32);
			assert_eq!(distribution_map.get(&0b0000_1111).unwrap(), &1495_u32);
			assert_eq!(distribution_map.get(&0b0001_1111).unwrap(), &2281_u32);
			assert_eq!(distribution_map.get(&0b0011_1111).unwrap(), &3324_u32);
			assert_eq!(distribution_map.get(&0b0111_1110).unwrap(), &2816_u32);
			assert_eq!(distribution_map.get(&0b0111_1111).unwrap(), &4751_u32);
		});
	}

	#[test]
	fn test_mate_egg_distribution_2() {
		ExtBuilder::default().build().execute_with(|| {
			let hash = Pallet::<Test>::random_hash(b"mate_dist_2", &ALICE);
			let mut hash_provider: HashProvider<Test, 32> = HashProvider::new(&hash);

			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];
			let mut leader_pet = 0b0000_1111_u8;

			let partner_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x54, 0x50, 0x50, 0x53, 0x55, 0x53, 0x51];
			let partner_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];
			let mut partner_pet = 0b0000_1111_u8;

			let loop_count = 100_000_u32;
			let mut distribution_map = BTreeMap::new();
			let mut match_map = BTreeMap::new();

			for i in 0..loop_count {
				let leader = create_random_pet(
					&ALICE,
					&PetType::FoxishDude,
					leader_pet,
					leader_spec_bytes,
					leader_progress_array,
					1000,
				);
				let partner = create_random_pet(
					&ALICE,
					&PetType::FoxishDude,
					partner_pet,
					partner_spec_bytes,
					partner_progress_array,
					1000,
				);

				let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
					leader,
					vec![partner],
					0,
					&mut hash_provider,
					1,
				)
				.expect("Should succeed in forging");

				assert_eq!(sacrifice_output.len(), 2);
				assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
				assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					assert_eq!(
						DnaUtils::read_attribute::<PetItemType>(avatar, AvatarAttr::ItemSubType),
						PetItemType::Egg
					);

					let custom_type_2 =
						DnaUtils::read_attribute_raw(avatar, AvatarAttr::CustomType2);

					distribution_map
						.entry(custom_type_2)
						.and_modify(|value| *value += 1)
						.or_insert(1_u32);

					if custom_type_2 == 0b0000_0000 {
						let leader_custom_type_2 =
							if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
								DnaUtils::read_attribute_raw(&leader, AvatarAttr::CustomType2)
							} else {
								panic!("ForgeOutput for leader should have been Minted!")
							};

						let partner_custom_type_2 =
							if let ForgeOutput::Forged((_, partner), _) = &sacrifice_output[0] {
								DnaUtils::read_attribute_raw(partner, AvatarAttr::CustomType2)
							} else {
								panic!("ForgeOutput for leader should have been Minted!")
							};

						match_map
							.entry((leader_custom_type_2, partner_custom_type_2))
							.and_modify(|value| *value += 1)
							.or_insert(1_u32);
					}

					let keys_no_0 = distribution_map
						.iter()
						.filter(|(key, _)| **key > 0)
						.map(|(key, _)| *key)
						.collect::<Vec<_>>();

					leader_pet = keys_no_0[hash_provider.next() as usize % keys_no_0.len()];
					partner_pet = keys_no_0[hash_provider.next() as usize % keys_no_0.len()];
				} else {
					panic!("ForgeOutput for second output should have been Minted!")
				}

				if i % 1000 == 999 {
					let hash_text = format!("loop_{:#07X}", i);
					let hash = Pallet::<Test>::random_hash(hash_text.as_bytes(), &ALICE);
					hash_provider = HashProvider::new(&hash);
				}
			}

			assert_eq!(distribution_map.get(&0b0000_0000).unwrap(), &62);
		});
	}

	#[test]
	fn test_mate_egg_distribution_3() {
		ExtBuilder::default().build().execute_with(|| {
			let hash = Pallet::<Test>::random_hash(b"mate_dist_3", &ALICE);
			let mut hash_provider: HashProvider<Test, 32> = HashProvider::new(&hash);

			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];
			let leader_pet = 0b0010_0111_u8;

			let partner_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x54, 0x50, 0x50, 0x53, 0x55, 0x53, 0x51];
			let partner_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];
			let partner_pet = 0b0000_0011_u8;

			let loop_count = 100_000_u32;
			let mut distribution_map = BTreeMap::new();
			let mut match_map = BTreeMap::new();

			for i in 0..loop_count {
				let leader = create_random_pet(
					&ALICE,
					&PetType::FoxishDude,
					leader_pet,
					leader_spec_bytes,
					leader_progress_array,
					1000,
				);
				let partner = create_random_pet(
					&ALICE,
					&PetType::FoxishDude,
					partner_pet,
					partner_spec_bytes,
					partner_progress_array,
					1000,
				);

				let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
					leader,
					vec![partner],
					0,
					&mut hash_provider,
					1,
				)
				.expect("Should succeed in forging");

				assert_eq!(sacrifice_output.len(), 2);
				assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
				assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					assert_eq!(
						DnaUtils::read_attribute::<PetItemType>(avatar, AvatarAttr::ItemSubType),
						PetItemType::Egg
					);

					let custom_type_2 =
						DnaUtils::read_attribute_raw(avatar, AvatarAttr::CustomType2);

					distribution_map
						.entry(custom_type_2)
						.and_modify(|value| *value += 1)
						.or_insert(1_u32);

					if custom_type_2 == 0b0000_0000 {
						let leader_custom_type_2 =
							if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
								DnaUtils::read_attribute_raw(&leader, AvatarAttr::CustomType2)
							} else {
								panic!("ForgeOutput for leader should have been Minted!")
							};

						let partner_custom_type_2 =
							if let ForgeOutput::Forged((_, partner), _) = &sacrifice_output[0] {
								DnaUtils::read_attribute_raw(partner, AvatarAttr::CustomType2)
							} else {
								panic!("ForgeOutput for leader should have been Minted!")
							};

						match_map
							.entry((leader_custom_type_2, partner_custom_type_2))
							.and_modify(|value| *value += 1)
							.or_insert(1_u32);
					}
				} else {
					panic!("ForgeOutput for second output should have been Minted!")
				}

				if i % 1000 == 999 {
					let hash_text = format!("loop_{:#07X}", i);
					let hash = Pallet::<Test>::random_hash(hash_text.as_bytes(), &ALICE);
					hash_provider = HashProvider::new(&hash);
				}
			}

			assert_eq!(distribution_map.get(&0b0000_0000).unwrap(), &3000);
		});
	}
}
