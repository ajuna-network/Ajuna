use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn build_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let mut output_sacrifices = Vec::with_capacity(input_sacrifices.len());

		let leader_spec_bytes = AvatarUtils::read_full_spec_bytes(&input_leader.1);

		let unord_1 = AvatarUtils::bits_to_enums::<MaterialItemType>(leader_spec_bytes[0] as u32);
		let pat_1 = AvatarUtils::bits_order_to_enum(leader_spec_bytes[1] as u32, 4, unord_1);

		let quantities = [
			leader_spec_bytes[3],
			leader_spec_bytes[4],
			leader_spec_bytes[5],
			leader_spec_bytes[6],
		];

		let sacrifice_pattern = input_sacrifices
			.iter()
			.map(|(_, sacrifice)| {
				AvatarUtils::read_attribute_as::<MaterialItemType>(
					sacrifice,
					&AvatarAttributes::ItemSubType,
				)
			})
			.collect::<Vec<MaterialItemType>>();

		let level = {
			let mut lvl = 0;
			for i in 1..6 {
				if input_sacrifices.iter().enumerate().all(|(index, (_, sacrifice))| {
					AvatarUtils::can_use_avatar(sacrifice, quantities[index] * i)
				}) {
					lvl = i;
				}
			}
			lvl
		};

		let mut soul_points = 0 as SoulCount;

		let can_use_leader = AvatarUtils::can_use_avatar(&input_leader.1, 1);

		if sacrifice_pattern == pat_1 && can_use_leader && level > 0 {
			for (i, (sacrifice_id, mut sacrifice)) in input_sacrifices.into_iter().enumerate() {
				let (_, avatar_consumed, out_soul_points) =
					AvatarUtils::use_avatar(&mut sacrifice, quantities[i] * level);
				soul_points += out_soul_points;

				let sacrifice_output = if avatar_consumed {
					ForgeOutput::Consumed(sacrifice_id)
				} else {
					ForgeOutput::Forged((sacrifice_id, sacrifice), 0)
				};

				output_sacrifices.push(sacrifice_output);
			}

			let max_build = 6_usize;
			let mut build_prop = SCALING_FACTOR_PERC;

			let mut generated_equippables = Vec::with_capacity(3);

			for i in 0..=max_build {
				if soul_points > 0 &&
					(build_prop * MAX_BYTE >= hash_provider.hash[i] as u32 * SCALING_FACTOR_PERC)
				{
					let pet_type = AvatarUtils::read_attribute_as::<PetType>(
						&input_leader.1,
						&AvatarAttributes::ClassType2,
					);

					let slot_type = AvatarUtils::read_attribute_as::<SlotType>(
						&input_leader.1,
						&AvatarAttributes::ClassType1,
					);

					let equippable_item_type = AvatarUtils::read_spec_byte_as::<EquippableItemType>(
						&input_leader.1,
						&AvatarSpecBytes::SpecByte3,
					);

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

					let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
					let generated_equippable = AvatarBuilder::with_dna(season_id, dna)
						.try_into_armor_and_component(
							&pet_type,
							&slot_type,
							&[equippable_item_type],
							&rarity_value,
							&(ColorType::None, ColorType::None),
							&Force::None,
							1,
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
				let sacrifice_index =
					hash_provider.get_hash_byte() as usize % generated_equippables.len();
				generated_equippables[sacrifice_index].souls.saturating_inc();
			}

			output_sacrifices
				.extend(generated_equippables.into_iter().map(|gen| ForgeOutput::Minted(gen)));
		} else {
			// TODO: Incomplete
			output_sacrifices.extend(
				input_sacrifices.into_iter().map(|sacrifice| ForgeOutput::Forged(sacrifice, 0)),
			);
		}

		Ok((LeaderForgeOutput::Forged(input_leader, 0), output_sacrifices))
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
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
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

			let total_soul_points = blueprint_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 9);

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
					.filter(|output| {
						is_minted_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Equippable.as_byte()),
								(
									AvatarAttributes::ItemSubType,
									EquippableItemType::ArmorBase.as_byte(),
								),
							],
						)
					})
					.count(),
				4
			);

			assert!(is_leader_forged_with_attributes(
				&leader_output,
				&[(AvatarAttributes::ItemType, ItemType::Blueprint.as_byte())]
			));
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
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
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

			let total_soul_points = blueprint_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 11);

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
					.filter(|output| {
						is_minted_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Equippable.as_byte()),
								(
									AvatarAttributes::ItemSubType,
									EquippableItemType::ArmorBase.as_byte(),
								),
							],
						)
					})
					.count(),
				4
			);

			for material in [MaterialItemType::PowerCells, MaterialItemType::Superconductors] {
				assert_eq!(
					sacrifice_output
						.iter()
						.filter(|output| is_forged_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Material.as_byte()),
								(AvatarAttributes::ItemSubType, material.as_byte()),
							]
						))
						.count(),
					1
				);
			}

			assert!(is_leader_forged_with_attributes(
				&leader_output,
				&[(AvatarAttributes::ItemType, ItemType::Blueprint.as_byte())]
			));
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
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
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

			let total_soul_points = blueprint_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 45);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::build_avatars(
				blueprint_input_1,
				vec![material_input_1, material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 11);
			// All materials that had only 1 item have been consumed
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 0);
			// All materials that had more than 1 item have been forged
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			// We minted equipment pieces for each material
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 7);
			// All minted items are ArmorBase equippables
			assert_eq!(
				sacrifice_output
					.iter()
					.filter(|output| {
						is_minted_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Equippable.as_byte()),
								(
									AvatarAttributes::ItemSubType,
									EquippableItemType::ArmorBase.as_byte(),
								),
							],
						)
					})
					.count(),
				7
			);

			for material in pattern {
				assert_eq!(
					sacrifice_output
						.iter()
						.filter(|output| is_forged_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Material.as_byte()),
								(AvatarAttributes::ItemSubType, material.as_byte()),
							]
						))
						.count(),
					1
				);
			}

			assert!(is_leader_forged_with_attributes(
				&leader_output,
				&[(AvatarAttributes::ItemType, ItemType::Blueprint.as_byte())]
			));
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
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
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

			let total_soul_points = blueprint_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 9);

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
						.filter(|output| is_forged_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Material.as_byte()),
								(AvatarAttributes::ItemSubType, material.as_byte()),
							]
						))
						.count(),
					1
				);
			}

			assert!(is_leader_forged_with_attributes(
				&leader_output,
				&[(AvatarAttributes::ItemType, ItemType::Blueprint.as_byte())]
			));
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
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
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

			let total_soul_points = blueprint_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 9);

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
						.filter(|output| is_forged_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Material.as_byte()),
								(AvatarAttributes::ItemSubType, material.as_byte()),
							]
						))
						.count(),
					1
				);
			}

			assert!(is_leader_forged_with_attributes(
				&leader_output,
				&[(AvatarAttributes::ItemType, ItemType::Blueprint.as_byte())]
			));
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
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
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

			let total_soul_points = blueprint_input_1.1.souls +
				material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;
			assert_eq!(total_soul_points, 9);

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
					.filter(|output| {
						is_minted_with_attributes(
							output,
							&[
								(AvatarAttributes::ItemType, ItemType::Equippable.as_byte()),
								(
									AvatarAttributes::ItemSubType,
									EquippableItemType::ArmorComponent2.as_byte(),
								),
							],
						)
					})
					.count(),
				4
			);

			assert!(is_leader_forged_with_attributes(
				&leader_output,
				&[(AvatarAttributes::ItemType, ItemType::Blueprint.as_byte())]
			));
		});
	}
}
