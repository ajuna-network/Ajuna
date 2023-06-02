use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn equip_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let mut new_souls = SoulCount::MIN;

		let mut leader_spec_bytes = AvatarUtils::read_full_spec_bytes(&leader);

		let equipment_slots_state = AvatarUtils::spec_byte_split_ten(&leader)
			.into_iter()
			.map(|slot| slot.iter().sum::<u8>() == 0)
			.collect::<Vec<_>>();

		let are_slots_maxed =
			equipment_slots_state.iter().filter(|slot| !**slot).count() >= MAX_EQUIPPED_SLOTS;

		for (_, sacrifice) in input_sacrifices.iter() {
			new_souls += sacrifice.souls;

			let slot_type =
				AvatarUtils::read_attribute(sacrifice, &AvatarAttributes::ClassType1) as usize - 1;

			if are_slots_maxed && equipment_slots_state[slot_type] {
				continue
			}

			let sacrifice_spec_byte_1 =
				AvatarUtils::read_spec_byte(sacrifice, &AvatarSpecBytes::SpecByte1);
			let sacrifice_spec_byte_2 =
				AvatarUtils::read_spec_byte(sacrifice, &AvatarSpecBytes::SpecByte2);
			let slot_type_mod = slot_type % 2;
			let slot_index = ((slot_type - slot_type_mod) * 3) / 2;

			if slot_type_mod == 0 {
				leader_spec_bytes[slot_index] = sacrifice_spec_byte_1;
				leader_spec_bytes[slot_index + 1] &= 0x0F;
				leader_spec_bytes[slot_index + 1] |= sacrifice_spec_byte_2 << 4;
			} else {
				leader_spec_bytes[slot_index + 1] &= 0xF0;
				leader_spec_bytes[slot_index + 1] |= sacrifice_spec_byte_1 >> 4;
				leader_spec_bytes[slot_index + 2] &= 0x00;
				leader_spec_bytes[slot_index + 2] |= sacrifice_spec_byte_1 << 4;
				leader_spec_bytes[slot_index + 2] |= sacrifice_spec_byte_2;
			}
		}

		AvatarUtils::write_full_spec_bytes(&mut leader, leader_spec_bytes);

		leader.souls += new_souls;

		let output_vec: Vec<ForgeOutput<T>> = input_sacrifices
			.into_iter()
			.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id))
			.collect();

		Ok((LeaderForgeOutput::Forged((leader_id, leader), 0), output_vec))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_equip_success() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let leader =
				create_random_pet(&ALICE, &PetType::FoxishDude, 0x0F, [0; 16], [0; 11], 100);

			let armor_progress = vec![
				EquippableItemType::ArmorBase,
				EquippableItemType::ArmorComponent1,
				EquippableItemType::ArmorComponent2,
			];

			let sacrifice_hash_base = [
				[
					0x80, 0x31, 0x6D, 0x62, 0xA2, 0x5B, 0xB9, 0x7F, 0x15, 0xEA, 0xAF, 0xE2, 0xB1,
					0xAC, 0x32, 0x61, 0x39, 0x92, 0x97, 0xE3, 0x30, 0x9C, 0xB3, 0x42, 0xB4, 0xC6,
					0xAB, 0xE7, 0x37, 0x71, 0xB8, 0x92,
				],
				[
					0x30, 0x19, 0x6D, 0xFF, 0x7A, 0x27, 0x97, 0x0C, 0xF2, 0x5E, 0xFB, 0xD8, 0x44,
					0x7C, 0xCF, 0x60, 0x68, 0x58, 0x57, 0x02, 0xB5, 0x56, 0x5E, 0x8A, 0xBD, 0x7C,
					0x07, 0xEE, 0x12, 0xB0, 0xAF, 0x7F,
				],
				[
					0xF4, 0xAA, 0x26, 0xC4, 0xE4, 0xC2, 0x12, 0xED, 0x41, 0x55, 0xC0, 0x58, 0xE6,
					0xB2, 0x1A, 0xB4, 0x74, 0x40, 0x41, 0xA6, 0x4E, 0xA9, 0x65, 0x22, 0x83, 0x57,
					0x3E, 0xCE, 0x56, 0x6E, 0xF0, 0xF2,
				],
				[
					0xD6, 0x0D, 0xB8, 0x30, 0x11, 0x55, 0x07, 0x7D, 0x5E, 0x54, 0xFC, 0x6D, 0x26,
					0x56, 0x4E, 0x2F, 0x99, 0xD2, 0x9E, 0x72, 0xDE, 0x28, 0xA2, 0xCB, 0x8B, 0xE8,
					0xDC, 0x2E, 0xEB, 0x3F, 0x04, 0x19,
				],
			];

			let armor_slots = [SlotType::Head, SlotType::Breast, SlotType::ArmFront];
			let pet_type = PetType::FoxishDude;
			let force = Force::Astral;
			let color_pair = (ColorType::ColorC, ColorType::ColorB);

			let mut armor_sacrifices = sacrifice_hash_base
				.into_iter()
				.zip(armor_slots)
				.map(|(hash, armor_slot)| {
					create_random_armor_component(
						hash,
						&ALICE,
						&pet_type,
						&armor_slot,
						&RarityTier::Legendary,
						&armor_progress,
						&color_pair,
						&force,
						100,
						&mut hash_provider,
					)
				})
				.collect::<Vec<_>>();

			let weapon_sacrifice = create_random_weapon(
				sacrifice_hash_base[3],
				&ALICE,
				&pet_type,
				&SlotType::WeaponBack,
				&EquippableItemType::WeaponVersion2,
				&color_pair,
				&force,
				100,
				&mut hash_provider,
			);

			let total_soul_points = leader.1.souls +
				armor_sacrifices.iter().map(|(_, avatar)| avatar.souls).sum::<SoulCount>() +
				weapon_sacrifice.1.souls;
			assert_eq!(total_soul_points, 500);

			let expected_dna = [
				0x11, 0x02, 0x05, 0x01, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00,
			];
			assert_eq!(leader.1.dna.as_slice(), &expected_dna);

			armor_sacrifices.push(weapon_sacrifice);

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::equip_avatars(leader, armor_sacrifices)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((avatar_id, avatar), _) = leader_output {
				let expected_dna = [
					0x11, 0x02, 0x05, 0x01, 0x0F, 0x97, 0xD9, 0x7D, 0x97, 0xD0, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x92, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);

				let weapon_2_dna = [
					0xC3, 0x06, 0x8D, 0x1D, 0x8D, 0xFB, 0x1C, 0xE6, 0x38, 0x23, 0x8F, 0x98, 0x9B,
					0xC2, 0x26, 0x35, 0x6B, 0x49, 0xF2, 0x86, 0x4F, 0x31, 0x4F, 0xF8, 0xBE, 0xD4,
					0xF4, 0x42, 0x11, 0xFD, 0x3F, 0x26,
				];

				let weapon_sacrifice_2 = create_random_weapon(
					weapon_2_dna,
					&ALICE,
					&pet_type,
					&SlotType::WeaponFront,
					&EquippableItemType::WeaponVersion3,
					&color_pair,
					&force,
					100,
					&mut hash_provider,
				);

				let (leader_output_2, sacrifice_output_2) =
					AvatarCombinator::<Test>::equip_avatars(
						(avatar_id, avatar),
						vec![weapon_sacrifice_2],
					)
					.expect("Should succeed in forging");

				assert_eq!(sacrifice_output_2.len(), 1);
				assert_eq!(
					sacrifice_output_2.iter().filter(|output| is_consumed(output)).count(),
					1
				);
				assert_eq!(sacrifice_output_2.iter().filter(|output| is_forged(output)).count(), 0);

				if let LeaderForgeOutput::Forged((_avatar_id, avatar), _) = leader_output_2 {
					let expected_dna_2 = [
						0x11, 0x02, 0x05, 0x01, 0x0F, 0x97, 0xD9, 0x7D, 0x97, 0xD0, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x09, 0x4D, 0x92, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					];
					assert_eq!(avatar.dna.as_slice(), &expected_dna_2);
				} else {
					panic!("LeaderForgeOutput for second forge should have been Forged!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_equip_leg_back() {
		ExtBuilder::default().build().execute_with(|| {
			let leader =
				create_random_pet(&ALICE, &PetType::FoxishDude, 0x0F, [0; 16], [0; 11], 100);

			let armor_progress = vec![
				EquippableItemType::ArmorBase,
				EquippableItemType::ArmorComponent1,
				EquippableItemType::ArmorComponent2,
				EquippableItemType::ArmorComponent3,
			];

			let mut armor_hash = HashProvider::new_with_bytes([
				0x9C, 0xB3, 0x42, 0xB4, 0xC6, 0xAB, 0xE7, 0x37, 0x71, 0xB8, 0x92, 0x9C, 0xB3, 0x42,
				0xB4, 0xC6, 0xAB, 0xE7, 0x37, 0x71, 0xB8, 0x92, 0x9C, 0xB3, 0x42, 0xB4, 0xC6, 0xAB,
				0xE7, 0x37, 0x71, 0xB8,
			]);

			let sac_1 = create_random_armor_component(
				[0; 32],
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::LegBack,
				&RarityTier::Legendary,
				&armor_progress,
				&(ColorType::ColorD, ColorType::ColorD),
				&Force::Astral,
				100,
				&mut armor_hash,
			);

			let expected_leader_dna = [
				0x11, 0x02, 0x05, 0x01, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00,
			];
			assert_eq!(leader.1.dna.as_slice(), &expected_leader_dna);

			let expected_sacrifice_dna = [
				0x41, 0x62, 0x05, 0x01, 0x00, 0xFF, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x55, 0x50, 0x50, 0x50, 0x53, 0x53,
				0x51, 0x55, 0x54, 0x52,
			];
			assert_eq!(sac_1.1.dna.as_slice(), &expected_sacrifice_dna);

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::equip_avatars(leader, vec![sac_1])
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				let expected_dna = [
					0x11, 0x02, 0x05, 0x01, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0F,
					0xFD, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_equip_failure() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];
			let leader = create_random_pet(
				&ALICE,
				&PetType::FoxishDude,
				0x0F,
				leader_spec_bytes,
				[0; 11],
				100,
			);

			let sacrifice_base_hash = [
				0xFB, 0x0E, 0x54, 0x01, 0x1C, 0x36, 0xA8, 0xBA, 0xED, 0x77, 0x45, 0x2A, 0x45, 0xD7,
				0xC8, 0xA1, 0x08, 0xE4, 0x97, 0x29, 0x44, 0x1F, 0xBA, 0xE7, 0x22, 0x2E, 0x90, 0x20,
				0x71, 0xFD, 0xA3, 0x68,
			];
			let sacrifice = create_random_armor_component(
				sacrifice_base_hash,
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::ArmFront,
				&RarityTier::Legendary,
				&[
					EquippableItemType::ArmorBase,
					EquippableItemType::ArmorComponent1,
					EquippableItemType::ArmorComponent2,
				],
				&(ColorType::ColorC, ColorType::ColorB),
				&Force::Astral,
				100,
				&mut hash_provider,
			);

			let total_soul_points = leader.1.souls + sacrifice.1.souls;
			assert_eq!(total_soul_points, 200);

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::equip_avatars(leader, vec![sacrifice])
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 200);

				let expected_dna = [
					0x11, 0x02, 0x05, 0x01, 0x0F, 0x97, 0x59, 0x75, 0x97, 0xD0, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x09, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				];
				assert_eq!(avatar.dna.as_slice(), expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
