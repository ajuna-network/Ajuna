use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn tinker_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		season_id: SeasonId,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let mut output_sacrifices = Vec::with_capacity(0);
		let mut consumed_leader = false;

		let sacrifice_pattern = input_sacrifices
			.iter()
			.map(|(_, sacrifice)| sacrifice.get_item_sub_type::<MaterialItemType>())
			.collect::<Vec<MaterialItemType>>();

		let pattern_flags = leader
			.get_specs()
			.chunks_exact(2)
			.take(4)
			.map(|chunk| {
				sacrifice_pattern ==
					DnaUtils::bits_order_to_enum(
						chunk[1] as u32,
						4,
						DnaUtils::bits_to_enums::<MaterialItemType>(chunk[0] as u32),
					)
			})
			.collect::<Vec<_>>();

		let mut soul_points = 0;

		let any_pattern_match = pattern_flags.iter().any(|pattern| *pattern);
		let can_use_leader = leader.can_use(1);
		let can_use_all_sacrifices =
			input_sacrifices.iter().all(|(_, sacrifice)| sacrifice.can_use(1));

		if any_pattern_match && can_use_leader && can_use_all_sacrifices {
			let mut success = true;

			let (use_result, consumed, out_soul_points) = leader.use_avatar(1);

			success &= use_result;
			consumed_leader = consumed;
			soul_points += out_soul_points;

			for (sacrifice_id, mut sacrifice) in input_sacrifices.into_iter() {
				let (use_result, avatar_consumed, out_soul_points) = sacrifice.use_avatar(1);
				success &= use_result;
				soul_points += out_soul_points;

				let sacrifice_output = if avatar_consumed {
					ForgeOutput::Consumed(sacrifice_id)
				} else {
					ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0)
				};

				output_sacrifices.push(sacrifice_output);
			}

			if !success || soul_points > u8::MAX as SoulCount {
				// https://github.com/ajuna-network/Ajuna.AAA.Season2/blob/master/Ajuna.AAA.Season2/Game.cs#L877
				todo!()
			}

			let equippable_item_type = {
				if pattern_flags[0] {
					EquippableItemType::ArmorBase
				} else if pattern_flags[1] {
					EquippableItemType::ArmorComponent1
				} else if pattern_flags[2] {
					EquippableItemType::ArmorComponent2
				} else if pattern_flags[3] {
					EquippableItemType::ArmorComponent3
				} else {
					// https://github.com/ajuna-network/Ajuna.AAA.Season2/blob/master/Ajuna.AAA.Season2/Game.cs#L899
					todo!()
				}
			};

			let pet_type = leader.get_class_type_2::<PetType>();
			let slot_type = leader.get_class_type_1::<SlotType>();

			let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
			let generated_blueprint = AvatarBuilder::with_dna(season_id, dna)
				.into_blueprint(
					&BlueprintItemType::Blueprint,
					&pet_type,
					&slot_type,
					&equippable_item_type,
					&sacrifice_pattern,
					soul_points as u8,
				)
				.build();

			output_sacrifices.push(ForgeOutput::Minted(generated_blueprint));
		} else {
			// TODO: Incomplete
			output_sacrifices.extend(input_sacrifices.into_iter().map(
				|(sacrifice_id, sacrifice)| {
					ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0)
				},
			));
		}

		let leader_output = if consumed_leader {
			LeaderForgeOutput::Consumed(leader_id)
		} else {
			LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0)
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

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquippableItemType::ArmorBase.as_byte() as usize,
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

			let pet_part_input_1 = create_random_pet_part(&ALICE, &pet_type, &slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let total_soul_points = pet_part_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 11);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Consumed(_) = leader_output {
				let minted_blueprint = sacrifice_output
					.into_iter()
					.filter(is_minted)
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_item_type(), ItemType::Blueprint);
					assert_eq!(
						wrapped.get_spec::<EquippableItemType>(SpecIdx::Byte3),
						EquippableItemType::ArmorBase
					);
					assert_eq!(wrapped.get_quantity(), total_soul_points as u8);
					assert_eq!(wrapped.get_souls(), total_soul_points);
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

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquippableItemType::ArmorBase.as_byte() as usize,
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

			let pet_part_input_1 = create_random_pet_part(&ALICE, &pet_type, &slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, &pattern[0], 2);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 2);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let total_soul_points = pet_part_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 16);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
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
					.filter(is_minted)
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_item_type(), ItemType::Blueprint);
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

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquippableItemType::ArmorBase.as_byte() as usize,
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

			let pet_part_input_1 = create_random_pet_part(&ALICE, &pet_type, &slot_type, 2);
			let material_input_1 = create_random_material(&ALICE, &pattern[0], 2);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 2);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 2);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 2);

			let total_soul_points = pet_part_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 22);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
				assert_eq!(DnaUtils::read_attribute_raw(&leader, AvatarAttr::Quantity), 1);
			} else {
				panic!("LeaderForgeOutput should have been Forged!");
			}

			let minted_blueprint = sacrifice_output
				.into_iter()
				.filter(is_minted)
				.collect::<Vec<ForgeOutput<Test>>>()
				.pop()
				.expect("Should have 1 element!");

			if let ForgeOutput::Minted(avatar) = minted_blueprint {
				let wrapped = WrappedAvatar::new(avatar);
				assert_eq!(wrapped.get_item_type(), ItemType::Blueprint);
				assert_eq!(wrapped.get_quantity(), 11);
			} else {
				panic!("ForgeOutput of blueprint should have been Minted!");
			}
		})
	}

	#[test]
	fn test_tinker_failure_wrong_material_order() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquippableItemType::ArmorBase.as_byte() as usize,
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

			let pet_part_input_1 = create_random_pet_part(&ALICE, &pet_type, &slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let total_soul_points = pet_part_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 11);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
				let leader_quantity =
					DnaUtils::read_attribute_raw(&leader, AvatarAttr::Quantity) as u32;
				assert_eq!(leader_quantity, 1);

				assert_eq!(
					sacrifice_output
						.iter()
						.map(|output| {
							match output {
								ForgeOutput::Forged((_, avatar), _) =>
									DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity)
										as u32,
								_ => 0,
							}
						})
						.sum::<u32>(),
					4
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

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquippableItemType::ArmorBase.as_byte() as usize,
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

			let pet_part_input_1 = create_random_pet_part(&ALICE, &pet_type, &slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, &MaterialItemType::Metals, 1);
			let material_input_2 = create_random_material(&ALICE, &MaterialItemType::Ceramics, 1);
			let material_input_3 =
				create_random_material(&ALICE, &MaterialItemType::Superconductors, 1);
			let material_input_4 =
				create_random_material(&ALICE, &MaterialItemType::Electronics, 1);

			let total_soul_points = pet_part_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 9);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, leader), _) = leader_output {
				let leader_quantity =
					DnaUtils::read_attribute_raw(&leader, AvatarAttr::Quantity) as u32;
				assert_eq!(leader_quantity, 1);

				assert_eq!(
					sacrifice_output
						.iter()
						.map(|output| {
							match output {
								ForgeOutput::Forged((_, avatar), _) =>
									DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity)
										as u32,
								_ => 0,
							}
						})
						.sum::<u32>(),
					4
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

			let pet_type = PetType::TankyBullwog;
			let slot_type = SlotType::Breast;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				EquippableItemType::ArmorComponent3.as_byte() as usize,
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

			let pet_part_input_1 = create_random_pet_part(&ALICE, &pet_type, &slot_type, 1);
			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let total_soul_points = pet_part_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 8);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				pet_part_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Consumed(_) = leader_output {
				let minted_blueprint = sacrifice_output
					.into_iter()
					.filter(is_minted)
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_item_type(), ItemType::Blueprint);
					assert_eq!(
						wrapped.get_spec::<EquippableItemType>(SpecIdx::Byte3),
						EquippableItemType::ArmorComponent3
					);
					assert_eq!(wrapped.get_quantity(), total_soul_points as u8);
					assert_eq!(wrapped.get_souls(), total_soul_points);

					let avatar_dna = wrapped.get_dna().as_slice();
					let expected_dna =
						[0x51, 0x21, 0x13, 0x08, 0x00, 0x66, 0x6C, 0x04, 0x01, 0x01, 0x01, 0x01];

					assert_eq!(&avatar_dna[0..12], &expected_dna);
				} else {
					panic!("ForgeOutput of blueprint should have been Minted!");
				}
			} else {
				panic!("LeaderForgeOutput should have been Consumed!");
			}
		});
	}

	#[test]
	fn test_tinker_success_on_other_pattern_2() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;

			let unit_fn = |avatar: Avatar| {
				let mut avatar = avatar;
				avatar.souls = 1;
				WrappedAvatar::new(avatar)
			};

			let leader = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x12, 0x42, 0x12, 0x05, 0x00, 0x96, 0x78, 0xd1, 0xc6, 0x59, 0x4e, 0x1e, 0x8d,
					0x52, 0x38, 0x55, 0xac, 0x4e, 0xee, 0x09, 0x20, 0xd6, 0x36, 0xc0, 0x9a, 0xd2,
					0x08, 0x86, 0xb9, 0xbb, 0xf7, 0xe0,
				]),
				Some(unit_fn),
			);

			let sac_1 = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x23, 0x00, 0x11, 0x24, 0x00, 0x40, 0x3a, 0xdb, 0x6e, 0xf2, 0x37, 0x75, 0xd1,
					0x2c, 0xe5, 0x73, 0x2d, 0x29, 0xce, 0x16, 0xeb, 0xe8, 0x35, 0x01, 0x12, 0x5e,
					0x5c, 0xbf, 0x33, 0x4a, 0x3e, 0x33,
				]),
				Some(unit_fn),
			);

			let sac_2 = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x28, 0x00, 0x11, 0x29, 0x00, 0xcd, 0x8c, 0x45, 0x36, 0x52, 0xc5, 0xd9, 0x2f,
					0x4d, 0x5e, 0x3c, 0x69, 0x07, 0x11, 0xfa, 0xb6, 0x32, 0x9e, 0xa9, 0x94, 0x85,
					0xc1, 0x43, 0x83, 0xe6, 0x87, 0xff,
				]),
				Some(unit_fn),
			);

			let sac_3 = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x25, 0x00, 0x11, 0x1f, 0x00, 0x57, 0x2e, 0xf0, 0xa4, 0x02, 0xef, 0x9e, 0x59,
					0x78, 0xf0, 0x9d, 0xf5, 0x37, 0x71, 0xe5, 0x3a, 0x25, 0x3c, 0x1b, 0xed, 0xe2,
					0x9e, 0xc9, 0xe9, 0xbf, 0x7c, 0xa0,
				]),
				Some(unit_fn),
			);

			let sac_4 = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x22, 0x00, 0x11, 0x21, 0x00, 0xe8, 0x01, 0x9d, 0xdf, 0xc2, 0x99, 0xe6, 0x5d,
					0x30, 0xbb, 0x22, 0x05, 0xcf, 0x20, 0x0a, 0xdd, 0x2e, 0xb4, 0x70, 0x40, 0x14,
					0xd2, 0x03, 0x14, 0x35, 0xed, 0xc4,
				]),
				Some(unit_fn),
			);

			let total_soul_points =
				leader.1.get_souls() +
					sac_1.1.get_souls() + sac_2.1.get_souls() +
					sac_3.1.get_souls() + sac_4.1.get_souls();
			assert_eq!(total_soul_points, 5);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::tinker_avatars(
				leader,
				vec![sac_1, sac_2, sac_3, sac_4],
				season_id,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, _), _) = leader_output {
				let minted_blueprint = sacrifice_output
					.into_iter()
					.filter(is_minted)
					.collect::<Vec<ForgeOutput<Test>>>()
					.pop()
					.expect("Should have 1 element!");

				if let ForgeOutput::Minted(avatar) = minted_blueprint {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_item_type(), ItemType::Blueprint);
					assert_eq!(wrapped.get_souls(), total_soul_points);
				} else {
					panic!("ForgeOutput of blueprint should have been Minted!");
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!");
			}
		});
	}
}
