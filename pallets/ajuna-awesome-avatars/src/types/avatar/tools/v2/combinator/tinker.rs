use super::*;

impl<'a, T> AvatarCombinator<'a, T>
where
	T: Config,
{
	pub(super) fn tinker_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let mut output_sacrifices = Vec::with_capacity(0);
		let mut consumed_leader = false;

		let sacrifice_pattern = input_sacrifices
			.iter()
			.map(|(_, sacrifice)| {
				AvatarUtils::read_attribute_as::<MaterialItemType>(
					sacrifice,
					AvatarAttributes::ItemSubType,
				)
			})
			.collect::<Vec<MaterialItemType>>();

		let pattern_flags = AvatarUtils::read_full_spec_bytes(&leader)
			.chunks_exact(2)
			.take(4)
			.map(|chunk| {
				sacrifice_pattern ==
					AvatarUtils::bits_order_to_enum(
						chunk[1] as u32,
						4,
						AvatarUtils::bits_to_enums::<MaterialItemType>(chunk[0] as u32),
					)
			})
			.collect::<Vec<_>>();

		let mut soul_points = 0;

		let all_patterns_match = pattern_flags.iter().any(|pattern| *pattern);
		let can_use_leader = AvatarUtils::can_use_avatar(&leader, 1);
		let can_use_all_sacrifices = input_sacrifices
			.iter()
			.all(|(_, sacrifice)| AvatarUtils::can_use_avatar(sacrifice, 1));

		if all_patterns_match && can_use_leader && can_use_all_sacrifices {
			let mut success = true;

			let (use_result, consumed, out_soul_points) = AvatarUtils::use_avatar(&mut leader, 1);

			success &= use_result;
			consumed_leader = consumed;
			soul_points += out_soul_points;

			for (sacrifice_id, mut sacrifice) in input_sacrifices.into_iter() {
				let (use_result, _, out_soul_points) = AvatarUtils::use_avatar(&mut sacrifice, 1);
				success &= use_result;
				soul_points += out_soul_points;

				let sacrifice_output =
					if AvatarUtils::read_attribute(&sacrifice, AvatarAttributes::Quantity) == 0 {
						ForgeOutput::Consumed(sacrifice_id)
					} else {
						ForgeOutput::Forged((sacrifice_id, sacrifice), 0)
					};

				output_sacrifices.push(sacrifice_output);
			}

			if !success || soul_points > u8::MAX as SoulCount {
				// https://github.com/ajuna-network/Ajuna.AAA.Season2/blob/master/Ajuna.AAA.Season2/Game.cs#L877
				todo!()
			}

			let equipable_item_type = {
				if pattern_flags[0] {
					EquipableItemType::ArmorBase
				} else if pattern_flags[1] {
					EquipableItemType::ArmorComponent1
				} else if pattern_flags[2] {
					EquipableItemType::ArmorComponent2
				} else if pattern_flags[3] {
					EquipableItemType::ArmorComponent3
				} else {
					// https://github.com/ajuna-network/Ajuna.AAA.Season2/blob/master/Ajuna.AAA.Season2/Game.cs#L899
					todo!()
				}
			};

			let pet_type =
				AvatarUtils::read_attribute_as::<PetType>(&leader, AvatarAttributes::ClassType2);

			let slot_type =
				AvatarUtils::read_attribute_as::<SlotType>(&leader, AvatarAttributes::ClassType1);

			let dna =
				AvatarMinterV2::<T>(PhantomData).generate_base_avatar_dna(hash_provider, 6)?;
			let generated_blueprint = AvatarBuilder::with_dna(season_id, dna)
				.into_blueprint(
					BlueprintItemType::Blueprint,
					pet_type,
					slot_type,
					equipable_item_type,
					sacrifice_pattern,
					soul_points as SoulCount,
				)
				.build();

			output_sacrifices.push(ForgeOutput::Minted(generated_blueprint));
		} else {
			// TODO: Incomplete
			output_sacrifices.extend(
				input_sacrifices.into_iter().map(|sacrifice| ForgeOutput::Forged(sacrifice, 0)),
			);
		}

		let leader_output = if consumed_leader {
			LeaderForgeOutput::Consumed(leader_id)
		} else {
			LeaderForgeOutput::Forged((leader_id, leader), 0)
		};

		Ok((leader_output, output_sacrifices))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_tinker_success_no_materials_left() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquipableItemType::ArmorBase.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::Optics,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::PowerCells
				]
			);

			let pet_part_input_1 = create_random_pet_part(&ALICE, pet_type, slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, pattern[0].clone(), 1);
			let material_input_2 = create_random_material(&ALICE, pattern[1].clone(), 1);
			let material_input_3 = create_random_material(&ALICE, pattern[2].clone(), 1);
			let material_input_4 = create_random_material(&ALICE, pattern[3].clone(), 1);

			let total_soul_points = pet_part_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 5);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Consumed(_) = leader_output {
				let minted_blueprint = sacrifice_output
					.into_iter()
					.filter(|output| is_minted(output))
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					assert!(AvatarUtils::has_attribute_with_value(
						&avatar,
						AvatarAttributes::ItemType,
						ItemType::Blueprint
					));
					assert_eq!(
						AvatarUtils::read_spec_byte(&avatar, AvatarSpecBytes::SpecByte3),
						EquipableItemType::ArmorBase.as_byte()
					);
					assert_eq!(
						AvatarUtils::read_attribute(&avatar, AvatarAttributes::Quantity),
						total_soul_points as u8
					);
					assert_eq!(avatar.souls, total_soul_points);
				} else {
					panic!("ForgeOutput of blueprint should have been Minted!");
				}
			} else {
				panic!("LeaderForgeOutput should have been Consumed!");
			}
		})
	}

	#[test]
	fn test_tinker_success_some_materials_left() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquipableItemType::ArmorBase.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::Optics,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::PowerCells
				]
			);

			let pet_part_input_1 = create_random_pet_part(&ALICE, pet_type, slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, pattern[0].clone(), 2);
			let material_input_2 = create_random_material(&ALICE, pattern[1].clone(), 1);
			let material_input_3 = create_random_material(&ALICE, pattern[2].clone(), 2);
			let material_input_4 = create_random_material(&ALICE, pattern[3].clone(), 1);

			let total_soul_points = pet_part_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 7);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			assert_eq!(
				sacrifice_output
					.iter()
					.map(|output| {
						match output {
							ForgeOutput::Forged((_, avatar), _) => avatar.souls,
							ForgeOutput::Minted(avatar) => avatar.souls,
							_ => 0,
						}
					})
					.sum::<SoulCount>(),
				total_soul_points
			);

			if let LeaderForgeOutput::Consumed(_) = leader_output {
				let minted_blueprint = sacrifice_output
					.into_iter()
					.filter(|output| is_minted(output))
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					assert!(AvatarUtils::has_attribute_with_value(
						&avatar,
						AvatarAttributes::ItemType,
						ItemType::Blueprint
					));
				} else {
					panic!("ForgeOutput of blueprint should have been Minted!");
				}
			} else {
				panic!("LeaderForgeOutput should have been Consumed!");
			}
		})
	}

	#[test]
	fn test_tinker_success_all_materials_left() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquipableItemType::ArmorBase.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::Optics,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::PowerCells
				]
			);

			let pet_part_input_1 = create_random_pet_part(&ALICE, pet_type, slot_type, 2);
			let material_input_1 = create_random_material(&ALICE, pattern[0].clone(), 2);
			let material_input_2 = create_random_material(&ALICE, pattern[1].clone(), 2);
			let material_input_3 = create_random_material(&ALICE, pattern[2].clone(), 2);
			let material_input_4 = create_random_material(&ALICE, pattern[3].clone(), 2);

			let total_soul_points = pet_part_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 10);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
				assert_eq!(AvatarUtils::read_attribute(&leader, AvatarAttributes::Quantity), 1);
			} else {
				panic!("LeaderForgeOutput should have been Forged!");
			}

			let minted_blueprint = sacrifice_output
				.into_iter()
				.filter(|output| is_minted(output))
				.collect::<Vec<ForgeOutput<Test>>>()
				.pop()
				.expect("Should have 1 element!");

			if let ForgeOutput::Minted(avatar) = minted_blueprint {
				assert!(AvatarUtils::has_attribute_with_value(
					&avatar,
					AvatarAttributes::ItemType,
					ItemType::Blueprint
				));

				assert_eq!(AvatarUtils::read_attribute(&avatar, AvatarAttributes::Quantity), 5);
			} else {
				panic!("ForgeOutput of blueprint should have been Minted!");
			}
		})
	}

	#[test]
	fn test_tinker_failure_wrong_material_order() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquipableItemType::ArmorBase.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::Optics,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::PowerCells
				]
			);

			let pet_part_input_1 = create_random_pet_part(&ALICE, pet_type, slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, pattern[0].clone(), 1);
			let material_input_2 = create_random_material(&ALICE, pattern[2].clone(), 1);
			let material_input_3 = create_random_material(&ALICE, pattern[1].clone(), 1);
			let material_input_4 = create_random_material(&ALICE, pattern[3].clone(), 1);

			let total_soul_points = pet_part_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 5);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
				let leader_quantity =
					AvatarUtils::read_attribute(&leader, AvatarAttributes::Quantity) as u32;
				assert_eq!(leader_quantity, 1);

				assert_eq!(
					sacrifice_output
						.iter()
						.map(|output| {
							match output {
								ForgeOutput::Forged((_, avatar), _) =>
									AvatarUtils::read_attribute(avatar, AvatarAttributes::Quantity)
										as u32,
								_ => 0,
							}
						})
						.sum::<u32>(),
					(total_soul_points - leader_quantity)
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!");
			}
		});
	}

	#[test]
	fn test_tinker_failure_wrong_material() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquipableItemType::ArmorBase.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::Optics,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::PowerCells
				]
			);

			let pet_part_input_1 = create_random_pet_part(&ALICE, pet_type, slot_type, 1);

			let material_input_1 = create_random_material(&ALICE, MaterialItemType::Metals, 1);
			let material_input_2 = create_random_material(&ALICE, MaterialItemType::Ceramics, 1);
			let material_input_3 =
				create_random_material(&ALICE, MaterialItemType::Superconductors, 1);
			let material_input_4 = create_random_material(&ALICE, MaterialItemType::Electronics, 1);

			let total_soul_points = pet_part_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 5);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
				let leader_quantity =
					AvatarUtils::read_attribute(&leader, AvatarAttributes::Quantity) as u32;
				assert_eq!(leader_quantity, 1);

				assert_eq!(
					sacrifice_output
						.iter()
						.map(|output| {
							match output {
								ForgeOutput::Forged((_, avatar), _) =>
									AvatarUtils::read_attribute(avatar, AvatarAttributes::Quantity)
										as u32,
								_ => 0,
							}
						})
						.sum::<u32>(),
					(total_soul_points - leader_quantity)
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!");
			}
		});
	}

	#[test]
	fn test_tinker_success_on_other_pattern() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_type = PetType::TankyBullwog;
			let slot_type = SlotType::Breast;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquipableItemType::ArmorComponent3.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::PowerCells,
					MaterialItemType::Ceramics,
					MaterialItemType::Superconductors,
					MaterialItemType::Electronics
				]
			);

			let pet_part_input_1 = create_random_pet_part(&ALICE, pet_type, slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, pattern[0].clone(), 1);
			let material_input_2 = create_random_material(&ALICE, pattern[1].clone(), 1);
			let material_input_3 = create_random_material(&ALICE, pattern[2].clone(), 1);
			let material_input_4 = create_random_material(&ALICE, pattern[3].clone(), 1);

			let total_soul_points = pet_part_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 5);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Consumed(_) = leader_output {
				let minted_blueprint = sacrifice_output
					.into_iter()
					.filter(|output| is_minted(output))
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					assert!(AvatarUtils::has_attribute_with_value(
						&avatar,
						AvatarAttributes::ItemType,
						ItemType::Blueprint
					));
					assert_eq!(
						AvatarUtils::read_spec_byte(&avatar, AvatarSpecBytes::SpecByte3),
						EquipableItemType::ArmorComponent3.as_byte()
					);
					assert_eq!(
						AvatarUtils::read_attribute(&avatar, AvatarAttributes::Quantity),
						total_soul_points as u8
					);
					assert_eq!(avatar.souls, total_soul_points);

					let avatar_dna = avatar.dna.as_slice();
					let expected_dna =
						[0x51, 0x21, 0x13, 0x05, 0x00, 0x66, 0x6C, 0x04, 0x01, 0x01, 0x01, 0x01];

					assert_eq!(&avatar_dna[0..12], &expected_dna);
				} else {
					panic!("ForgeOutput of blueprint should have been Minted!");
				}
			} else {
				panic!("LeaderForgeOutput should have been Consumed!");
			}
		});
	}
}
