use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn build_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let mut output_sacrifices = Vec::with_capacity(input_sacrifices.len());

		let leader_spec_bytes = leader.get_specs();
		let unord_1 = DnaUtils::bits_to_enums::<MaterialItemType>(leader_spec_bytes[0] as u32);
		let pat_1 = DnaUtils::bits_order_to_enum(leader_spec_bytes[1] as u32, 4, unord_1);

		let quantities = [
			leader_spec_bytes[3],
			leader_spec_bytes[4],
			leader_spec_bytes[5],
			leader_spec_bytes[6],
		];

		let sacrifice_pattern = input_sacrifices
			.iter()
			.map(|(_, sacrifice)| sacrifice.get_item_sub_type::<MaterialItemType>())
			.collect::<Vec<MaterialItemType>>();

		let level = {
			let mut lvl = 0;
			for i in 1..6 {
				if input_sacrifices
					.iter()
					.enumerate()
					.all(|(index, (_, sacrifice))| sacrifice.can_use(quantities[index] * i))
				{
					lvl = i;
				}
			}
			lvl
		};

		let mut soul_points = 0 as SoulCount;

		let mut leader_consumed = false;

		if sacrifice_pattern == pat_1 && leader.can_use(1) && level > 0 {
			let (_, consumed, out_souls) = leader.use_avatar(1);
			soul_points += out_souls;
			leader_consumed = consumed;

			for (i, (sacrifice_id, mut sacrifice)) in input_sacrifices.into_iter().enumerate() {
				let (_, avatar_consumed, out_soul_points) =
					sacrifice.use_avatar(quantities[i] * level);
				soul_points += out_soul_points;

				let sacrifice_output = if avatar_consumed {
					ForgeOutput::Consumed(sacrifice_id)
				} else {
					ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0)
				};

				output_sacrifices.push(sacrifice_output);
			}

			let max_build = 6_usize;
			let mut build_prop = SCALING_FACTOR_PERC;

			let mut generated_equippables = Vec::with_capacity(3);

			for i in 0..max_build {
				if soul_points > 0 &&
					(build_prop * MAX_BYTE >= hash_provider.hash[i] as u32 * SCALING_FACTOR_PERC)
				{
					let pet_type = leader.get_class_type_2::<PetType>();
					let slot_type = leader.get_class_type_1::<SlotType>();
					let equippable_item_type =
						leader.get_spec::<EquippableItemType>(SpecIdx::Byte3);

					let rarity_value = {
						let rarity_val = hash_provider.hash[i] % 8 + 1;
						if rarity_val < level {
							if rarity_val > RarityTier::Rare.as_byte() {
								RarityTier::Rare
							} else {
								RarityTier::from_byte(rarity_val)
							}
						} else {
							RarityTier::from_byte(1)
						}
					};

					let item_sp = {
						let item_sp = level + rarity_value.as_byte();
						if item_sp as u32 > soul_points {
							soul_points
						} else {
							item_sp as SoulCount
						}
					};

					let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
					let generated_equippable = AvatarBuilder::with_dna(season_id, dna)
						.try_into_armor_and_component(
							&pet_type,
							&slot_type,
							&[equippable_item_type],
							&rarity_value,
							&(ColorType::Null, ColorType::Null),
							&Force::Null,
							item_sp,
							hash_provider,
						)
						.map_err(|_| Error::<T>::IncompatibleForgeComponents)?
						.build();

					generated_equippables.push(generated_equippable);

					soul_points -= 1;
				}

				build_prop = build_prop.saturating_sub(15 - (level as u32 * 3));
			}

			for _ in 0..soul_points as usize {
				let sacrifice_index = hash_provider.next() as usize % generated_equippables.len();
				generated_equippables[sacrifice_index % 32].souls.saturating_inc();
			}

			output_sacrifices
				.extend(generated_equippables.into_iter().map(|gen| ForgeOutput::Minted(gen)));
		} else {
			// TODO: Incomplete
			output_sacrifices.extend(input_sacrifices.into_iter().map(
				|(sacrifice_id, sacrifice)| {
					ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0)
				},
			));
		}

		let leader_output = if leader_consumed {
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
	fn test_build_successfully_no_materials_left() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let forge_hash = [
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;
			let equip_type = EquippableItemType::ArmorBase;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
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

			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let blueprint_input_1 =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 5);

			let total_soul_points = blueprint_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 15);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 8);
			// All materials have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 4);

			// All minted items are ArmorBase equippables
			assert_eq!(
				sacrifice_output
					.iter()
					.filter(|output| if let ForgeOutput::Minted(avatar) = output {
						WrappedAvatar::new(avatar.clone())
							.has_full_type(ItemType::Equippable, EquippableItemType::ArmorBase)
					} else {
						false
					})
					.count(),
				4
			);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert!(WrappedAvatar::new(avatar.clone()).has_type(ItemType::Blueprint));
			} else {
				panic!("LeaderForgeOutput should be Forged!");
			}
		});
	}

	#[test]
	fn test_build_successfully_some_materials_left() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let forge_hash = [
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;
			let equip_type = EquippableItemType::ArmorBase;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
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

			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 2);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 2);

			let blueprint_input_1 =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 5);

			let total_soul_points = blueprint_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 20);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 8);
			// All materials that had only 1 item have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);
			// All materials that had more than 1 item have been forged
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 2);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 4);
			// All minted items are ArmorBase equippables
			assert_eq!(
				sacrifice_output
					.iter()
					.filter(|output| if let ForgeOutput::Minted(avatar) = output {
						WrappedAvatar::new(avatar.clone())
							.has_full_type(ItemType::Equippable, EquippableItemType::ArmorBase)
					} else {
						false
					})
					.count(),
				4
			);

			for material in [MaterialItemType::PowerCells, MaterialItemType::Superconductors] {
				assert_eq!(
					sacrifice_output
						.iter()
						.filter(|output| if let ForgeOutput::Forged((_, avatar), _) = output {
							WrappedAvatar::new(avatar.clone())
								.has_full_type(ItemType::Material, material)
						} else {
							false
						})
						.count(),
					1
				);
			}

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert!(WrappedAvatar::new(avatar.clone()).has_type(ItemType::Blueprint));
			} else {
				panic!("LeaderForgeOutput should be Forged!");
			}
		});
	}

	#[test]
	fn test_build_successfully_all_materials_left() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let forge_hash = [
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;
			let equip_type = EquippableItemType::ArmorBase;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
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

			let material_input_1 = create_random_material(&ALICE, &pattern[0], 10);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 10);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 10);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 10);

			let blueprint_input_1 =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 5);

			let total_soul_points = blueprint_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 105);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 10);
			// All materials that had only 1 item have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			// All materials that had more than 1 item have been forged
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 6);
			// All minted items are ArmorBase equippables
			assert_eq!(
				sacrifice_output
					.iter()
					.filter(|output| if let ForgeOutput::Minted(avatar) = output {
						WrappedAvatar::new(avatar.clone())
							.has_full_type(ItemType::Equippable, EquippableItemType::ArmorBase)
					} else {
						false
					})
					.count(),
				6
			);

			for material in pattern {
				assert_eq!(
					sacrifice_output
						.iter()
						.filter(|output| if let ForgeOutput::Forged((_, avatar), _) = output {
							WrappedAvatar::new(avatar.clone())
								.has_full_type(ItemType::Material, material)
						} else {
							false
						})
						.count(),
					1
				);
			}

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert!(WrappedAvatar::new(avatar.clone()).has_type(ItemType::Blueprint));
			} else {
				panic!("LeaderForgeOutput should be Forged!");
			}
		});
	}

	#[test]
	fn test_build_failure_wrong_material_order() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let forge_hash = [
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;
			let equip_type = EquippableItemType::ArmorBase;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
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

			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let blueprint_input_1 =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 5);

			let total_soul_points = blueprint_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 15);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			// All materials that had only 1 item have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			// All materials that had more than 1 item have been forged
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			for material in pattern {
				assert_eq!(
					sacrifice_output
						.iter()
						.filter(|output| if let ForgeOutput::Forged((_, avatar), _) = output {
							WrappedAvatar::new(avatar.clone())
								.has_full_type(ItemType::Material, material)
						} else {
							false
						})
						.count(),
					1
				);
			}

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert!(WrappedAvatar::new(avatar.clone()).has_type(ItemType::Blueprint));
			} else {
				panic!("LeaderForgeOutput should be Forged!");
			}
		});
	}

	#[test]
	fn test_build_failure_wrong_material() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let forge_hash = [
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let pet_type = PetType::FoxishDude;
			let slot_type = SlotType::Head;
			let equip_type = EquippableItemType::ArmorBase;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
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

			// We change the materials from the original pattern
			// in order to cause a mismatch with the Blueprint specs
			let mutated_pattern = {
				let mut mutated = pattern.clone();
				mutated[2] = MaterialItemType::Metals;
				mutated
			};

			let material_input_1 = create_random_material(&ALICE, &mutated_pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &mutated_pattern[1], 1);
			let material_input_3 = create_random_material(&ALICE, &mutated_pattern[2], 1);
			let material_input_4 = create_random_material(&ALICE, &mutated_pattern[3], 1);

			// Here we use the original pattern
			let blueprint_input_1 =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 5);

			let total_soul_points = blueprint_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 17);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			// All materials that had only 1 item have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			// All materials that had more than 1 item have been forged
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			for material in mutated_pattern {
				assert_eq!(
					sacrifice_output
						.iter()
						.filter(|output| if let ForgeOutput::Forged((_, avatar), _) = output {
							WrappedAvatar::new(avatar.clone())
								.has_full_type(ItemType::Material, material)
						} else {
							false
						})
						.count(),
					1
				);
			}

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert!(WrappedAvatar::new(avatar.clone()).has_type(ItemType::Blueprint));
			} else {
				panic!("LeaderForgeOutput should be Forged!");
			}
		});
	}

	#[test]
	fn test_build_success_on_other_pattern() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let forge_hash = [
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let pet_type = PetType::TankyBullwog;
			let slot_type = SlotType::ArmBack;
			let equip_type = EquippableItemType::ArmorComponent2;

			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
			);

			assert_eq!(
				pattern,
				vec![
					MaterialItemType::Nanomaterials,
					MaterialItemType::Metals,
					MaterialItemType::PowerCells,
					MaterialItemType::Electronics
				]
			);

			let material_input_1 = create_random_material(&ALICE, &pattern[0], 1);
			let material_input_2 = create_random_material(&ALICE, &pattern[1], 1);
			let material_input_3 = create_random_material(&ALICE, &pattern[2], 1);
			let material_input_4 = create_random_material(&ALICE, &pattern[3], 1);

			let blueprint_input_1 =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 5);

			let total_soul_points = blueprint_input_1.1.get_souls() +
				material_input_1.1.get_souls() +
				material_input_2.1.get_souls() +
				material_input_3.1.get_souls() +
				material_input_4.1.get_souls();
			assert_eq!(total_soul_points, 15);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 8);
			// All materials have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 4);

			// All minted items are ArmorBase equippables
			assert_eq!(
				sacrifice_output
					.iter()
					.filter(|output| if let ForgeOutput::Minted(avatar) = output {
						WrappedAvatar::new(avatar.clone()).has_full_type(
							ItemType::Equippable,
							EquippableItemType::ArmorComponent2,
						)
					} else {
						false
					})
					.count(),
				4
			);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert!(WrappedAvatar::new(avatar.clone()).has_type(ItemType::Blueprint));
			} else {
				panic!("LeaderForgeOutput should be Forged!");
			}
		});
	}

	#[test]
	fn test_build_prep_1() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes([
				0x4C, 0x0B, 0xF6, 0x8A, 0xFF, 0x3D, 0xAD, 0xB0, 0x01, 0x15, 0xE1, 0x7B, 0x90, 0x36,
				0x38, 0x60, 0x55, 0x91, 0x25, 0x4D, 0x57, 0xFA, 0x57, 0x1D, 0x38, 0xB9, 0xC9, 0x99,
				0x42, 0xEA, 0x20, 0x37,
			]);

			let hash_base = [
				[
					0x51, 0x24, 0x13, 0x05, 0x00, 0x59, 0x4e, 0x03, 0x01, 0x01, 0x01, 0x01, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x24, 0x00, 0x11, 0xe5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x21, 0x00, 0x11, 0xe6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x27, 0x00, 0x11, 0xfa, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x25, 0x00, 0x11, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
			];

			let avatar_fn = |souls: SoulCount| {
				let mutate_fn = move |avatar: Avatar| {
					let mut avatar = avatar;
					avatar.souls = souls;
					WrappedAvatar::new(avatar)
				};

				Some(mutate_fn)
			};

			let leader = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[0]), avatar_fn(5));
			let sac_1 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[1]), avatar_fn(229));
			let sac_2 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[2]), avatar_fn(230));
			let sac_3 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[3]), avatar_fn(250));
			let sac_4 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[4]), avatar_fn(239));

			let total_souls =
				leader.1.get_souls() +
					sac_1.1.get_souls() + sac_2.1.get_souls() +
					sac_3.1.get_souls() + sac_4.1.get_souls();
			assert_eq!(total_souls, 953);

			assert_eq!(leader.1.get_quantity(), 5);
			assert_eq!(leader.1.get_souls(), 5);

			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(
					&leader.1,
					&[&sac_1.1, &sac_2.1, &sac_3.1, &sac_4.1]
				),
				ForgeType::Build
			);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				leader,
				vec![sac_1, sac_2, sac_3, sac_4],
				1,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 10);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 6);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				let item_type = DnaUtils::read_attribute::<ItemType>(&avatar, AvatarAttr::ItemType);
				assert_eq!(item_type, ItemType::Blueprint);

				assert_eq!(avatar.souls, 4);

				let leader_quantity = DnaUtils::read_attribute_raw(&avatar, AvatarAttr::Quantity);
				assert_eq!(leader_quantity, 4);

				let soul_points = sacrifice_output
					.into_iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Minted(avatar) => avatar.souls,
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>();

				assert_eq!(avatar.souls + soul_points, 987);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
