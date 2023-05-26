use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn assemble_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (
			(leader_id, mut input_leader),
			matching_sacrifices,
			consumed_sacrifices,
			non_matching_sacrifices,
		) = Self::match_avatars(
			input_leader,
			input_sacrifices,
			MATCH_ALGO_START_RARITY.as_byte(),
			hash_provider,
		);

		let rarity = RarityTier::from_byte(AvatarUtils::read_lowest_progress_byte(
			&AvatarUtils::read_progress_array(&input_leader),
			&ByteType::High,
		));

		let leader_rarity = AvatarUtils::read_attribute_as::<RarityTier>(
			&input_leader,
			&AvatarAttributes::RarityTier,
		);

		if AvatarUtils::has_attribute_set_with_values(
			&input_leader,
			&[
				(AvatarAttributes::ItemType, ItemType::Equippable.as_byte()),
				(AvatarAttributes::ItemSubType, EquippableItemType::ArmorBase.as_byte()),
			],
		) && leader_rarity < rarity
		{
			// Add a component to the base armor, only first component will be added
			if let Some((_, armor_component)) = matching_sacrifices.iter().find(|(_, sacrifice)| {
				AvatarUtils::has_attribute_with_value(
					sacrifice,
					&AvatarAttributes::ItemType,
					ItemType::Equippable,
				) && AvatarUtils::has_attribute_with_value_different_than(
					sacrifice,
					&AvatarAttributes::ItemSubType,
					EquippableItemType::ArmorBase,
				)
			}) {
				let spec_byte = AvatarSpecBytes::SpecByte1;
				let current_spec = AvatarUtils::read_spec_byte(&input_leader, &spec_byte);
				let armor_spec = AvatarUtils::read_spec_byte(armor_component, &spec_byte);

				AvatarUtils::write_spec_byte(
					&mut input_leader,
					&AvatarSpecBytes::SpecByte1,
					current_spec | armor_spec,
				);
			}
		}

		AvatarUtils::write_typed_attribute(
			&mut input_leader,
			&AvatarAttributes::RarityTier,
			&rarity,
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

		Ok((LeaderForgeOutput::Forged((leader_id, input_leader), 0), output_vec))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_assemble_single_base() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
				0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
				0xB0, 0x54, 0xFB, 0x62,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let hash_base = [
				[
					0xE7, 0x46, 0x00, 0xE4, 0xE8, 0x78, 0x12, 0xC4, 0xCA, 0x86, 0x53, 0x7F, 0x36,
					0x1B, 0x64, 0xA0, 0xC3, 0x6B, 0x5C, 0x5F, 0x13, 0x40, 0xBC, 0xC6, 0x97, 0x12,
					0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3,
				],
				[
					0x3B, 0x06, 0x56, 0x4C, 0x0C, 0x96, 0x6F, 0x41, 0x28, 0x85, 0x40, 0xEC, 0x53,
					0xAB, 0xF4, 0xCE, 0xCE, 0x6C, 0x60, 0x81, 0xBE, 0xBC, 0xCF, 0x82, 0xBD, 0x70,
					0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13,
				],
				[
					0x81, 0xA7, 0xCD, 0x5A, 0x36, 0x51, 0xB8, 0xB6, 0xE8, 0x9F, 0x6C, 0xE4, 0xE3,
					0x52, 0x15, 0xD0, 0xEB, 0xF5, 0x25, 0x97, 0xA7, 0xD2, 0xE4, 0xC0, 0xDC, 0x7C,
					0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76,
				],
				[
					0x1A, 0x1C, 0x41, 0x48, 0x0B, 0x96, 0xF9, 0xDC, 0xDA, 0x7A, 0x40, 0x28, 0x99,
					0x86, 0x58, 0xC3, 0x6A, 0xD4, 0x7C, 0x66, 0x58, 0xD1, 0x9C, 0x8E, 0x81, 0xCF,
					0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D,
				],
				[
					0x84, 0x78, 0x9A, 0x96, 0x77, 0xA2, 0xCE, 0xC3, 0x0E, 0x3C, 0x29, 0x20, 0x33,
					0x01, 0x67, 0x9C, 0xF8, 0x4D, 0x03, 0x36, 0x80, 0xB5, 0x37, 0x43, 0x6C, 0x71,
					0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8,
				],
			];

			let mut armor_component_set = hash_base
				.into_iter()
				.enumerate()
				.map(|(i, hash)| {
					create_random_armor_component(
						hash,
						&ALICE,
						&PetType::FoxishDude,
						&SlotType::Head,
						&RarityTier::Common,
						&[EquippableItemType::ArmorBase],
						&(ColorType::None, ColorType::None),
						&Force::None,
						i as SoulCount,
					)
				})
				.collect::<Vec<_>>();

			let total_soul_points =
				armor_component_set.iter().map(|(_, avatar)| avatar.souls).sum::<SoulCount>();
			assert_eq!(total_soul_points, 10);

			let armor_component_sacrifices = armor_component_set.split_off(1);
			let leader_armor_component = armor_component_set.pop().unwrap();

			let expected_progress_array =
				[0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];

			assert_eq!(
				AvatarUtils::read_progress_array(&leader_armor_component.1),
				expected_progress_array
			);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
				leader_armor_component,
				armor_component_sacrifices,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 10);

				let leader_progress_array = AvatarUtils::read_progress_array(&avatar);
				let expected_leader_progress_array =
					[0x14, 0x22, 0x20, 0x11, 0x20, 0x21, 0x20, 0x15, 0x11, 0x25, 0x23];
				assert_eq!(leader_progress_array, expected_leader_progress_array);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_assemble_single_base_with_component() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let hash_base = [
				[
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA1,
				],
				[
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA2,
				],
				[
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA3,
				],
				[
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA4,
				],
				[
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
					0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xA5,
				],
			];

			let progress_arrays = [
				[0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24],
				[0x21, 0x15, 0x22, 0x15, 0x13, 0x12, 0x12, 0x10, 0x13, 0x10, 0x15],
				[0x12, 0x13, 0x11, 0x12, 0x12, 0x20, 0x12, 0x13, 0x13, 0x12, 0x15],
				[0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x12, 0x12, 0x12],
				[0x11, 0x11, 0x25, 0x24, 0x14, 0x23, 0x13, 0x12, 0x52, 0x12, 0x12],
			];

			let mut armor_component_set = [
				EquippableItemType::ArmorBase,
				EquippableItemType::ArmorBase,
				EquippableItemType::ArmorBase,
				EquippableItemType::ArmorBase,
				EquippableItemType::ArmorComponent1,
			]
			.into_iter()
			.zip(hash_base)
			.zip(progress_arrays)
			.enumerate()
			.map(|(i, ((equip_type, hash), progress_array))| {
				let (id, mut avatar) = create_random_armor_component(
					hash,
					&ALICE,
					&PetType::FoxishDude,
					&SlotType::Head,
					&RarityTier::Common,
					&[equip_type],
					&(ColorType::None, ColorType::None),
					&Force::None,
					i as SoulCount,
				);
				AvatarUtils::write_progress_array(&mut avatar, progress_array);
				(id, avatar)
			})
			.collect::<Vec<_>>();

			let total_soul_points =
				armor_component_set.iter().map(|(_, avatar)| avatar.souls).sum::<SoulCount>();
			assert_eq!(total_soul_points, 10);

			let armor_component_sacrifices = armor_component_set.split_off(1);
			let leader_armor_component = armor_component_set.pop().unwrap();

			let expected_progress_array =
				[0x21, 0x10, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24];
			assert_eq!(
				AvatarUtils::read_progress_array(&leader_armor_component.1),
				expected_progress_array
			);

			let pre_assemble = AvatarUtils::bits_to_enums::<EquippableItemType>(
				AvatarUtils::read_spec_byte(&leader_armor_component.1, &AvatarSpecBytes::SpecByte1)
					as u32,
			);
			assert_eq!(pre_assemble.len(), 1);
			assert_eq!(pre_assemble[0], EquippableItemType::ArmorBase);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
				leader_armor_component,
				armor_component_sacrifices,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 2);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 7);

				let post_assemble = AvatarUtils::bits_to_enums::<EquippableItemType>(
					AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1) as u32,
				);
				assert_eq!(post_assemble.len(), 2);
				assert_eq!(post_assemble[0], EquippableItemType::ArmorBase);
				assert_eq!(post_assemble[1], EquippableItemType::ArmorComponent1);

				let leader_progress_array = AvatarUtils::read_progress_array(&avatar);
				let expected_leader_progress_array =
					[0x21, 0x20, 0x25, 0x23, 0x20, 0x23, 0x21, 0x22, 0x22, 0x22, 0x24];
				assert_eq!(leader_progress_array, expected_leader_progress_array);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_assemble_failure() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let hash_base = [
				[
					0xE7, 0x46, 0x00, 0xE4, 0xE8, 0x78, 0x12, 0xC4, 0xCA, 0x86, 0x53, 0x7F, 0x36,
					0x1B, 0x64, 0xA0, 0xC3, 0x6B, 0x5C, 0x5F, 0x13, 0x40, 0xBC, 0xC6, 0x97, 0x12,
					0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3,
				],
				[
					0x3B, 0x06, 0x56, 0x4C, 0x0C, 0x96, 0x6F, 0x41, 0x28, 0x85, 0x40, 0xEC, 0x53,
					0xAB, 0xF4, 0xCE, 0xCE, 0x6C, 0x60, 0x81, 0xBE, 0xBC, 0xCF, 0x82, 0xBD, 0x70,
					0x61, 0x14, 0xA2, 0x5E, 0x1A, 0x13,
				],
				[
					0x81, 0xA7, 0xCD, 0x5A, 0x36, 0x51, 0xB8, 0xB6, 0xE8, 0x9F, 0x6C, 0xE4, 0xE3,
					0x52, 0x15, 0xD0, 0xEB, 0xF5, 0x25, 0x97, 0xA7, 0xD2, 0xE4, 0xC0, 0xDC, 0x7C,
					0xF3, 0x6F, 0xE0, 0xB3, 0x88, 0x76,
				],
				[
					0x1A, 0x1C, 0x41, 0x48, 0x0B, 0x96, 0xF9, 0xDC, 0xDA, 0x7A, 0x40, 0x28, 0x99,
					0x86, 0x58, 0xC3, 0x6A, 0xD4, 0x7C, 0x66, 0x58, 0xD1, 0x9C, 0x8E, 0x81, 0xCF,
					0xE5, 0x78, 0x70, 0x68, 0x12, 0x0D,
				],
				[
					0x84, 0x78, 0x9A, 0x96, 0x77, 0xA2, 0xCE, 0xC3, 0x0E, 0x3C, 0x29, 0x20, 0x33,
					0x01, 0x67, 0x9C, 0xF8, 0x4D, 0x03, 0x36, 0x80, 0xB5, 0x37, 0x43, 0x6C, 0x71,
					0xAA, 0xA9, 0x3D, 0x9F, 0x8C, 0xB8,
				],
			];

			let slot_types =
				[SlotType::Head, SlotType::Head, SlotType::LegBack, SlotType::Head, SlotType::Head];

			let mut armor_component_set = hash_base
				.into_iter()
				.zip(slot_types)
				.enumerate()
				.map(|(i, (hash, slot_type))| {
					create_random_armor_component(
						hash,
						&ALICE,
						&PetType::FoxishDude,
						&slot_type,
						&RarityTier::Common,
						&[EquippableItemType::ArmorBase],
						&(ColorType::None, ColorType::None),
						&Force::None,
						i as SoulCount,
					)
				})
				.collect::<Vec<_>>();

			let total_soul_points =
				armor_component_set.iter().map(|(_, avatar)| avatar.souls).sum::<SoulCount>();
			assert_eq!(total_soul_points, 10);

			let armor_component_sacrifices = armor_component_set.split_off(1);
			let leader_armor_component = armor_component_set.pop().unwrap();

			let expected_progress_array =
				[0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
			assert_eq!(
				AvatarUtils::read_progress_array(&leader_armor_component.1),
				expected_progress_array
			);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::assemble_avatars(
				leader_armor_component,
				armor_component_sacrifices,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			assert!(is_leader_forged(&leader_output));
		});
	}
}
