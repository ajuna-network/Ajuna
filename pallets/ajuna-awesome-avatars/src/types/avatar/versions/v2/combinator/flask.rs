use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn flask_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let glimmer_count = input_sacrifices
			.iter()
			.filter(|(_, sacrifice)| {
				sacrifice.has_full_type(ItemType::Essence, EssenceItemType::Glimmer)
			})
			.count();

		let flask = input_sacrifices.iter().find(|(_, sacrifice)| {
			let item_sub_type = sacrifice.get_item_sub_type::<EssenceItemType>();

			sacrifice.has_type(ItemType::Essence) &&
				(item_sub_type == EssenceItemType::PaintFlask ||
					item_sub_type == EssenceItemType::GlowFlask)
		});

		if let Some((_, flask_avatar)) = flask {
			let mut leader_progress_array = leader.get_progress();
			let flask_progress_array = flask_avatar.get_progress();

			if let Some(mut matches) = DnaUtils::is_progress_match(
				leader_progress_array,
				flask_progress_array,
				MATCH_ALGO_START_RARITY.as_byte(),
			) {
				let flask_essence_type = flask_avatar.get_item_sub_type::<EssenceItemType>();

				let flask_spec_byte_1 = flask_avatar.get_spec::<u8>(SpecIdx::Byte1);
				let leader_spec_byte_2 = leader.get_spec::<u8>(SpecIdx::Byte2);

				if flask_essence_type == EssenceItemType::PaintFlask {
					let leader_spec_byte_1 = leader.get_spec::<u8>(SpecIdx::Byte1);
					let flask_spec_byte_2 = flask_avatar.get_spec::<u8>(SpecIdx::Byte2);

					leader
						.set_spec(SpecIdx::Byte1, (leader_spec_byte_1 & 0x0F) | flask_spec_byte_1);
					leader.set_spec(SpecIdx::Byte2, leader_spec_byte_2 | flask_spec_byte_2);
				} else if flask_essence_type == EssenceItemType::GlowFlask {
					leader
						.set_spec(SpecIdx::Byte2, (leader_spec_byte_2 & 0x08) | flask_spec_byte_1);
				}

				let mut index = matches.remove(0) as usize;
				leader_progress_array[index] += 0x10;

				let glimmer_chance = {
					let eff_glimmer_count = if glimmer_count > 8 { 8 } else { glimmer_count };
					80 + eff_glimmer_count * 15
				};
				let matches_count = if matches.len() > MAX_FLASK_PROGRESS {
					MAX_FLASK_PROGRESS
				} else {
					matches.len()
				};

				for i in 0..matches_count {
					if hash_provider.hash[i + 1] < glimmer_chance as u8 {
						index = matches.remove(0) as usize;
						leader_progress_array[index] += 0x10;
					}
				}

				let rarity_type =
					DnaUtils::lowest_progress_byte(&leader_progress_array, ByteType::High);
				leader.set_rarity(RarityTier::from_byte(rarity_type));
				leader.set_progress(leader_progress_array);
			}
		}

		let new_souls = {
			let mut new_souls = 0;

			for (_, sacrifice) in &input_sacrifices {
				new_souls += sacrifice.get_souls();
			}

			new_souls
		};

		leader.add_souls(new_souls);

		let output_vec: Vec<ForgeOutput<T>> = input_sacrifices
			.into_iter()
			.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id))
			.collect();

		Ok((LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0), output_vec))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_flask_paint() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
				0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
				0xB0, 0x54, 0xFB, 0x62,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let armor_hash_base = [
				0xE7, 0x46, 0x00, 0xE4, 0xE8, 0x78, 0x12, 0xC4, 0xCA, 0x86, 0x53, 0x7F, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25, 0x48,
				0xC5, 0xD9, 0x05, 0xC3,
			];

			let mut progress_array_generator = HashProvider::new_with_bytes([
				0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC, 0xC6,
				0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25,
				0x48, 0xC5, 0xD9, 0x05,
			]);

			let armor_component = create_random_armor_component(
				armor_hash_base,
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&RarityTier::Epic,
				&[EquippableItemType::ArmorBase, EquippableItemType::ArmorComponent3],
				&(ColorType::Null, ColorType::Null),
				&Force::Null,
				80,
				&mut progress_array_generator,
			);

			let paint_flask = create_random_paint_flask(
				&ALICE,
				&(ColorType::ColorA, ColorType::ColorA),
				64,
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x41],
			);

			let expected_dna = [
				0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40,
				0x45, 0x41, 0x55, 0x43,
			];
			assert_eq!(armor_component.1.get_dna().as_slice(), &expected_dna);

			let expected_leader_progress_array =
				[0x44, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40, 0x45, 0x41, 0x55, 0x43];
			let leader_progress_array = armor_component.1.get_progress();
			assert_eq!(leader_progress_array, expected_leader_progress_array);

			let expected_sacrifice_progress_array =
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x41];
			let sacrifice_progress_array = paint_flask.1.get_progress();
			assert_eq!(sacrifice_progress_array, expected_sacrifice_progress_array);

			assert!(DnaUtils::is_progress_match(
				leader_progress_array,
				sacrifice_progress_array,
				MATCH_ALGO_START_RARITY.as_byte()
			)
			.is_some());

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::flask_avatars(
				armor_component,
				vec![paint_flask],
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 144);

				let expected_dna = [
					0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x42, 0x40, 0x41, 0x50,
					0x51, 0x40, 0x45, 0x41, 0x55, 0x43,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_flask_glow_1() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
				0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
				0xB0, 0x54, 0xFB, 0x62,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let hash_base = [
				0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40,
				0x45, 0x41, 0x55, 0x43,
			];

			let unit_closure = |avatar: Avatar| {
				let mut avatar = avatar;
				avatar.souls = 623;
				WrappedAvatar::new(avatar)
			};

			let avatar =
				create_random_avatar::<Test, _>(&ALICE, Some(hash_base), Some(unit_closure));

			let glow_flask = create_random_glow_flask(
				&ALICE,
				&Force::Empathy,
				64,
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x42],
			);

			let glimmer = create_random_glimmer(&ALICE, 14);

			let expected_dna = [
				0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40,
				0x45, 0x41, 0x55, 0x43,
			];
			assert_eq!(avatar.1.get_dna().as_slice(), &expected_dna);

			let leader_progress_array = avatar.1.get_progress();
			let sacrifice_progress_array = glow_flask.1.get_progress();

			let expected_progress_array_leader =
				[0x54, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40, 0x45, 0x41, 0x55, 0x43];
			assert_eq!(leader_progress_array, expected_progress_array_leader);

			let expected_progress_array_other =
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x42];
			assert_eq!(sacrifice_progress_array, expected_progress_array_other);

			assert!(DnaUtils::is_progress_match(
				leader_progress_array,
				sacrifice_progress_array,
				MATCH_ALGO_START_RARITY.as_byte()
			)
			.is_some());

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::flask_avatars(
				avatar,
				vec![glow_flask, glimmer],
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 715);

				let expected_dna = [
					0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x52, 0x40, 0x41, 0x50,
					0x51, 0x40, 0x45, 0x41, 0x55, 0x43,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_flask_glow_2() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
				0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
				0xB0, 0x54, 0xFB, 0x62,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let mut progress_array_generator = HashProvider::new_with_bytes([
				0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC, 0xC6,
				0x97, 0x12, 0x25, 0x48, 0xC5, 0xD9, 0x05, 0xC3, 0x40, 0xBC, 0xC6, 0x97, 0x12, 0x25,
				0x48, 0xC5, 0xD9, 0x05,
			]);

			let armor_component = create_random_armor_component(
				[0; 32],
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::LegBack,
				&RarityTier::Epic,
				&[
					EquippableItemType::ArmorBase,
					EquippableItemType::ArmorComponent1,
					EquippableItemType::ArmorComponent2,
					EquippableItemType::ArmorComponent3,
				],
				&(ColorType::ColorD, ColorType::ColorD),
				&Force::Null,
				100,
				&mut progress_array_generator,
			);

			let glow_flask = create_random_glow_flask(
				&ALICE,
				&Force::Empathy,
				64,
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x42],
			);

			let expected_dna = [
				0x41, 0x62, 0x04, 0x01, 0x00, 0xFF, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40,
				0x45, 0x41, 0x55, 0x43,
			];
			assert_eq!(armor_component.1.get_dna().as_slice(), &expected_dna);

			let leader_progress_array = armor_component.1.get_progress();
			let sacrifice_progress_array = glow_flask.1.get_progress();

			let expected_progress_array_leader =
				[0x44, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40, 0x45, 0x41, 0x55, 0x43];
			assert_eq!(leader_progress_array, expected_progress_array_leader);

			let expected_progress_array_other =
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x42];
			assert_eq!(sacrifice_progress_array, expected_progress_array_other);

			assert!(DnaUtils::is_progress_match(
				leader_progress_array,
				sacrifice_progress_array,
				MATCH_ALGO_START_RARITY.as_byte()
			)
			.is_some());

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::flask_avatars(
				armor_component,
				vec![glow_flask],
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 164);

				let expected_dna = [
					0x41, 0x62, 0x04, 0x01, 0x00, 0xFF, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x52, 0x40, 0x41, 0x50,
					0x51, 0x40, 0x45, 0x41, 0x55, 0x43,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);

				let spec_byte_2 = DnaUtils::read_spec_raw(&avatar, SpecIdx::Byte2);
				assert_eq!(spec_byte_2, 0b0000_1110_u8);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_flask_glow_3() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
				0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
				0xB0, 0x54, 0xFB, 0x62,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let hash_base = [
				[
					0x41, 0x11, 0x04, 0x01, 0x00, 0x0f, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x40, 0x52, 0x40, 0x55,
					0x43, 0x44, 0x44, 0x44, 0x45, 0x40,
				],
				[
					0x35, 0x00, 0x04, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x43, 0x43, 0x41, 0x41,
					0x43, 0x43, 0x43, 0x43, 0x43, 0x42,
				],
			];

			let unit_fn = |avatar: Avatar| {
				let mut avatar = avatar;
				avatar.souls = 623;
				WrappedAvatar::new(avatar)
			};

			let leader = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[0]), Some(unit_fn));
			let sac_1 = create_random_avatar::<Test, _>(&ALICE, Some(hash_base[1]), Some(unit_fn));

			let leader_progress_array = leader.1.get_progress();
			let sacrifice_progress_array = sac_1.1.get_progress();

			let total_soul_points = leader.1.get_souls() + sac_1.1.get_souls();

			assert!(DnaUtils::is_progress_match(
				leader_progress_array,
				sacrifice_progress_array,
				MATCH_ALGO_START_RARITY.as_byte()
			)
			.is_some());

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::flask_avatars(leader, vec![sac_1], &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, total_soul_points);

				let expected_dna = [
					0x41, 0x11, 0x04, 0x01, 0x00, 0x0F, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0x40, 0x52, 0x50, 0x55,
					0x43, 0x54, 0x54, 0x44, 0x45, 0x40,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_flask_fail() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x3F, 0x83, 0x25, 0x3B, 0xA9, 0x24, 0xF2, 0xF6, 0xB5, 0xA9, 0x37, 0x15, 0x25, 0x2C,
				0x04, 0xFC, 0xEC, 0x45, 0xC1, 0x4D, 0x86, 0xE7, 0x96, 0xE5, 0x20, 0x5D, 0x8B, 0x39,
				0xB0, 0x54, 0xFB, 0x62,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let hash_base = [
				0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40,
				0x45, 0x41, 0x55, 0x43,
			];

			let unit_closure = |avatar: Avatar| {
				let mut avatar = avatar;
				avatar.souls = 623;
				WrappedAvatar::new(avatar)
			};

			let avatar =
				create_random_avatar::<Test, _>(&ALICE, Some(hash_base), Some(unit_closure));

			let glow_flask = create_random_glow_flask(
				&ALICE,
				&Force::Empathy,
				64,
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x41],
			);

			let glimmer = create_random_glimmer(&ALICE, 14);

			let expected_dna = [
				0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40,
				0x45, 0x41, 0x55, 0x43,
			];
			assert_eq!(avatar.1.get_dna().as_slice(), &expected_dna);

			let leader_progress_array = avatar.1.get_progress();
			let sacrifice_progress_array = glow_flask.1.get_progress();

			let expected_progress_array_leader =
				[0x54, 0x42, 0x40, 0x41, 0x50, 0x51, 0x40, 0x45, 0x41, 0x55, 0x43];
			assert_eq!(leader_progress_array, expected_progress_array_leader);

			let expected_progress_array_other =
				[0x45, 0x43, 0x54, 0x53, 0x54, 0x51, 0x52, 0x50, 0x54, 0x55, 0x41];
			assert_eq!(sacrifice_progress_array, expected_progress_array_other);

			assert!(DnaUtils::is_progress_match(
				leader_progress_array,
				sacrifice_progress_array,
				MATCH_ALGO_START_RARITY.as_byte()
			)
			.is_none());

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::flask_avatars(
				avatar,
				vec![glow_flask, glimmer],
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, avatar), _) = leader_output {
				assert_eq!(avatar.souls, 715);

				let expected_dna = [
					0x41, 0x12, 0x04, 0x01, 0x00, 0x09, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x42, 0x40, 0x41, 0x50,
					0x51, 0x40, 0x45, 0x41, 0x55, 0x43,
				];
				assert_eq!(avatar.dna.as_slice(), &expected_dna);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
