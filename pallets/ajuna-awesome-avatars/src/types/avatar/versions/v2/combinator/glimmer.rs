use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn glimmer_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let color_types = 5;
		let forces = 7;

		let (leader_id, mut leader) = input_leader;
		let mut leader_consumed = false;

		let mut other_output = Vec::new();

		for (i, (sacrifice_id, mut sacrifice)) in input_sacrifices.into_iter().enumerate() {
			if leader_consumed {
				// If we consumed the leader in a previous step, we collect all
				// sacrifices and skip all future loops
				other_output.push(ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0));
				continue
			}

			let leader_quantity = leader.get_quantity();
			let sacrifice_quantity = sacrifice.get_quantity();

			if leader_quantity < GLIMMER_FORGE_GLIMMER_USE ||
				sacrifice_quantity < GLIMMER_FORGE_MATERIAL_USE
			{
				// If we skip the loop then the sacrifice remains unused
				other_output.push(ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0));
				continue
			}

			let (_, consumed, out_leader_souls) = leader.use_avatar(GLIMMER_FORGE_GLIMMER_USE);
			leader_consumed = consumed;
			let (_, consumed_sacrifice, out_sacrifice_souls) =
				sacrifice.use_avatar(GLIMMER_FORGE_MATERIAL_USE);

			other_output.push(if consumed_sacrifice {
				ForgeOutput::Consumed(sacrifice_id)
			} else {
				ForgeOutput::Forged((sacrifice_id, sacrifice.unwrap()), 0)
			});

			let soul_points = out_leader_souls + out_sacrifice_souls;

			let index = i * 4;
			let rand_0 = hash_provider.hash[index];
			let rand_1 = hash_provider.hash[index + 1];
			let rand_2 = hash_provider.hash[index + 2];
			let rand_3 = hash_provider.hash[index + 3];

			let dna = MinterV2::<T>::generate_empty_dna::<32>()?;

			let mut gen_avatar = AvatarBuilder::with_dna(season_id, dna);

			if rand_0 as u32 * SCALING_FACTOR_PERC < GLIMMER_PROB_PERC * MAX_BYTE {
				if rand_1 == rand_2 {
					let progress_array = DnaUtils::generate_progress(
						&RarityTier::Rare,
						SCALING_FACTOR_PERC,
						Some(SPARK_PROGRESS_PROB_PERC),
						hash_provider,
					);
					gen_avatar =
						gen_avatar.into_egg(&RarityTier::Rare, 0x00, soul_points, progress_array);
				} else if rand_1 ==
					(DnaUtils::high_nibble_of(rand_1) + DnaUtils::low_nibble_of(rand_2))
				{
					let color_pair = (
						ColorType::from_byte(rand_1 % (color_types + 1)),
						ColorType::from_byte(rand_2 % (color_types + 1)),
					);
					let force = Force::from_byte((rand_3 % forces) + 1);

					gen_avatar = gen_avatar.into_unidentified(color_pair, force, soul_points);
				} else if rand_1 as u32 * SCALING_FACTOR_PERC < COLOR_GLOW_SPARK * MAX_BYTE {
					let color_pair = (
						ColorType::from_byte(rand_2 % (color_types + 1)),
						ColorType::from_byte(rand_3 % (color_types + 1)),
					);
					let progress_array = DnaUtils::generate_progress(
						&RarityTier::Rare,
						SCALING_FACTOR_PERC,
						Some(SPARK_PROGRESS_PROB_PERC),
						hash_provider,
					);
					gen_avatar =
						gen_avatar.into_color_spark(&color_pair, soul_points, progress_array);
				} else {
					let force = Force::from_byte((rand_2 % forces) + 1);
					let progress_array = DnaUtils::generate_progress(
						&RarityTier::Rare,
						SCALING_FACTOR_PERC,
						Some(SPARK_PROGRESS_PROB_PERC),
						hash_provider,
					);
					gen_avatar = gen_avatar.into_glow_spark(&force, soul_points, progress_array);
				}
			} else if (rand_0 as u32 * SCALING_FACTOR_PERC <
				(GLIMMER_PROB_PERC + TOOLBOX_PERC) * MAX_BYTE) &&
				leader.can_use(GLIMMER_FORGE_TOOLBOX_USE)
			{
				let (_, consumed, out_leader_souls) = leader.use_avatar(GLIMMER_FORGE_TOOLBOX_USE);
				leader_consumed = consumed;
				gen_avatar = gen_avatar.into_toolbox(soul_points + out_leader_souls);
			} else {
				// the glimmer used to gamble is burnt
				gen_avatar = gen_avatar.into_dust(soul_points - out_leader_souls);
			}

			other_output.push(ForgeOutput::Minted(gen_avatar.build()));
		}

		let leader_output = if leader_consumed {
			LeaderForgeOutput::Consumed(leader_id)
		} else {
			LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0)
		};

		Ok((leader_output, other_output))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_glimmer_simple() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x43, 0x5E, 0x2E, 0xD2, 0xE0, 0x39, 0x1A, 0x1C, 0x4F, 0x54, 0x24, 0xE4, 0x19, 0x51,
				0x4A, 0xC5, 0xA8, 0x29, 0x59, 0x5B, 0x54, 0x21, 0x72, 0x94, 0xFD, 0xCF, 0x78, 0xC9,
				0xDE, 0x0A, 0xAF, 0x2D,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 10);
			let sacrifice = create_random_material(&ALICE, &MaterialItemType::Polymers, 8);

			let expected_dna = [
				0x31, 0x00, 0x22, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00,
			];
			assert_eq!(leader.1.get_dna().as_slice(), &expected_dna);

			let total_soul_points = leader.1.get_souls() + sacrifice.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let wrapped = WrappedAvatar::new(leader_avatar);
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + wrapped.get_souls();
				assert_eq!(output_souls + GLIMMER_SP as u32, total_soul_points);

				assert_eq!(wrapped.get_quantity(), 9);
				assert_eq!(wrapped.get_item_type(), ItemType::Essence);
				assert_eq!(
					wrapped.get_item_sub_type::<EssenceItemType>(),
					EssenceItemType::Glimmer
				);

				if let ForgeOutput::Forged((_, avatar), _) = &sacrifice_output[0] {
					assert_eq!(DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity), 4);
					assert_eq!(
						DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType,),
						ItemType::Material
					);
					assert_eq!(
						DnaUtils::read_attribute::<MaterialItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						),
						MaterialItemType::Polymers
					);
				} else {
					panic!("ForgeOutput for the first output should have been Forged!")
				}

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					assert_eq!(DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity), 8);
					assert_eq!(
						DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType,),
						ItemType::Special
					);
					assert_eq!(
						DnaUtils::read_attribute::<SpecialItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						),
						SpecialItemType::Dust
					);

					let expected_dna_head = [0x61, 0x00, 0x11, 0x08, 0x00];
					let avatar_dna_slice = &avatar.dna[0..5];

					// We only need to check the 5 first bytes since the rest are not relevant for
					// Dust avatars
					assert_eq!(avatar_dna_slice, &expected_dna_head);
				} else {
					panic!("ForgeOutput for the second output should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_glimmer_multiple() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x48, 0xD2, 0x2C, 0xCA, 0xEE, 0x3F, 0x81, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 100);
			let sacrifice_1 = create_random_material(&ALICE, &MaterialItemType::Polymers, 20);
			let sacrifice_2 =
				create_random_material(&ALICE, &MaterialItemType::Superconductors, 20);
			let sacrifice_3 = create_random_material(&ALICE, &MaterialItemType::Ceramics, 20);
			let sacrifice_4 = create_random_material(&ALICE, &MaterialItemType::Metals, 20);

			let total_soul_points = leader.1.get_souls() +
				sacrifice_1.1.get_souls() +
				sacrifice_2.1.get_souls() +
				sacrifice_3.1.get_souls() +
				sacrifice_4.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice_1, sacrifice_2, sacrifice_3, sacrifice_4],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 8);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let wrapped = WrappedAvatar::new(leader_avatar);
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + wrapped.get_souls();
				assert_eq!(output_souls + 4 * GLIMMER_SP as u32, total_soul_points);

				assert_eq!(wrapped.get_quantity(), 96);
				assert_eq!(wrapped.get_item_type(), ItemType::Essence);
				assert_eq!(
					wrapped.get_item_sub_type::<EssenceItemType>(),
					EssenceItemType::Glimmer
				);

				let material_set = [
					MaterialItemType::Polymers,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::Metals,
				];

				for (i, material) in material_set.into_iter().enumerate() {
					if let ForgeOutput::Forged((_, avatar), _) = &sacrifice_output[i * 2] {
						assert_eq!(DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity), 16);
						assert_eq!(
							DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType,),
							ItemType::Material
						);
						assert_eq!(
							DnaUtils::read_attribute::<MaterialItemType>(
								avatar,
								AvatarAttr::ItemSubType,
							),
							material
						);
					} else {
						panic!("ForgeOutput should have been Forged!")
					}
				}

				for i in (1..8).step_by(2) {
					if let ForgeOutput::Minted(avatar) = &sacrifice_output[i] {
						assert_eq!(
							DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType,),
							ItemType::Special
						);

						let item_sub_type = DnaUtils::read_attribute::<SpecialItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						);

						let qty = DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity);

						match item_sub_type {
							SpecialItemType::Dust => assert!(qty == 4 || qty == 8 || qty == 12),
							SpecialItemType::ToolBox => assert_eq!(qty, 1),
							_ => panic!("Item sub type should be Dust or Toolbox"),
						}
					} else {
						panic!("ForgeOutput should have been Minted!")
					}
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_glimmer_toolbox() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x55, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 100);
			let sacrifice_1 = create_random_material(&ALICE, &MaterialItemType::Polymers, 20);
			let sacrifice_2 =
				create_random_material(&ALICE, &MaterialItemType::Superconductors, 20);
			let sacrifice_3 = create_random_material(&ALICE, &MaterialItemType::Ceramics, 20);
			let sacrifice_4 = create_random_material(&ALICE, &MaterialItemType::Metals, 20);

			let total_soul_points = leader.1.get_souls() +
				sacrifice_1.1.get_souls() +
				sacrifice_2.1.get_souls() +
				sacrifice_3.1.get_souls() +
				sacrifice_4.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice_1, sacrifice_2, sacrifice_3, sacrifice_4],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 8);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let wrapped = WrappedAvatar::new(leader_avatar);
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + wrapped.get_souls();
				assert_eq!(output_souls + 4 * GLIMMER_SP as u32, total_soul_points);

				assert_eq!(wrapped.get_quantity(), 96);
				assert_eq!(wrapped.get_item_type(), ItemType::Essence);
				assert_eq!(
					wrapped.get_item_sub_type::<EssenceItemType>(),
					EssenceItemType::Glimmer
				);

				let material_set = [
					MaterialItemType::Polymers,
					MaterialItemType::Superconductors,
					MaterialItemType::Ceramics,
					MaterialItemType::Metals,
				];

				for (i, material) in material_set.into_iter().enumerate() {
					if let ForgeOutput::Forged((_, avatar), _) = &sacrifice_output[i * 2] {
						assert_eq!(DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity), 16);
						assert_eq!(
							DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType,),
							ItemType::Material
						);
						assert_eq!(
							DnaUtils::read_attribute::<MaterialItemType>(
								avatar,
								AvatarAttr::ItemSubType,
							),
							material
						);
					} else {
						panic!("ForgeOutput should have been Forged!")
					}
				}

				for i in (1..8).step_by(2) {
					if let ForgeOutput::Minted(avatar) = &sacrifice_output[i] {
						assert_eq!(
							DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType,),
							ItemType::Special
						);

						let item_sub_type = DnaUtils::read_attribute::<SpecialItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						);

						let qty = DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity);

						match item_sub_type {
							SpecialItemType::Dust => assert!(qty == 4 || qty == 8 || qty == 12),
							SpecialItemType::ToolBox => assert_eq!(qty, 1),
							_ => panic!("Item sub type should be Dust or Toolbox"),
						}
					} else {
						panic!("ForgeOutput should have been Minted!")
					}
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_empty_spark() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x58, 0xD2, 0x1C, 0xCA, 0xEE, 0x4F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 1);
			let sacrifice = create_random_material(&ALICE, &MaterialItemType::Polymers, 4);

			let total_soul_points = leader.1.get_souls() + sacrifice.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			assert!(is_leader_consumed(&leader_output));

			if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
				assert_eq!(avatar.souls + GLIMMER_SP as u32, total_soul_points);
				assert_eq!(DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity), 8);
				assert_eq!(
					DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType),
					ItemType::Special
				);
				assert_eq!(
					DnaUtils::read_attribute::<SpecialItemType>(avatar, AvatarAttr::ItemSubType,),
					SpecialItemType::Dust
				);
			} else {
				panic!("ForgeOutput should have been Minted!")
			}
		});
	}

	#[test]
	fn test_color_spark() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x0A, 0xA9, 0x45, 0x37, 0x78, 0x1C, 0x04, 0x39, 0x8E, 0x1C, 0xDC, 0x95, 0xE2, 0x75,
				0xD5, 0xE7, 0x69, 0xB1, 0x27, 0xDC, 0xA4, 0x9B, 0x6E, 0xF0, 0x95, 0x6B, 0x89, 0xC5,
				0xA5, 0x2E, 0xDF, 0x03,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 10);
			let sacrifice = create_random_material(&ALICE, &MaterialItemType::Polymers, 8);

			let total_soul_points = leader.1.get_souls() + sacrifice.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + leader_avatar.souls;
				assert_eq!(output_souls, total_soul_points);

				if let ForgeOutput::Minted(avatar) = sacrifice_output.into_iter().nth(1).unwrap() {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_quantity(), 1);
					assert_eq!(wrapped.get_item_type(), ItemType::Essence);
					assert_eq!(
						wrapped.get_item_sub_type::<EssenceItemType>(),
						EssenceItemType::GlowSpark
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_glow_spark() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x04, 0xE5, 0x5C, 0xED, 0x33, 0x80, 0x1E, 0x2B, 0x10, 0xEB, 0xD1, 0xB0, 0xC4, 0x09,
				0x78, 0x0D, 0xF4, 0x33, 0x92, 0x6D, 0x1F, 0x8B, 0x53, 0xE0, 0x1B, 0x23, 0x84, 0x7B,
				0x4A, 0xF0, 0xEA, 0x94,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 10);
			let sacrifice = create_random_material(&ALICE, &MaterialItemType::Polymers, 8);

			let total_soul_points = leader.1.get_souls() + sacrifice.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + leader_avatar.souls;
				assert_eq!(output_souls, total_soul_points);

				if let ForgeOutput::Minted(avatar) = sacrifice_output.into_iter().nth(1).unwrap() {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_quantity(), 1);
					assert_eq!(wrapped.get_item_type(), ItemType::Essence);
					assert_eq!(
						wrapped.get_item_sub_type::<EssenceItemType>(),
						EssenceItemType::GlowSpark
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_egg() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x04, 0xBB, 0xBB, 0x1E, 0x64, 0x4E, 0xEA, 0x73, 0x97, 0x0D, 0x1D, 0x68, 0xEB, 0xB3,
				0x15, 0x85, 0xE4, 0xA0, 0x33, 0x6A, 0x0D, 0xDC, 0x0D, 0x88, 0xFD, 0x97, 0x87, 0x0B,
				0x23, 0x3B, 0x46, 0x1F,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 10);
			let sacrifice = create_random_material(&ALICE, &MaterialItemType::Polymers, 8);

			let total_soul_points = leader.1.get_souls() + sacrifice.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + leader_avatar.souls;
				assert_eq!(output_souls, total_soul_points);

				if let ForgeOutput::Minted(avatar) = sacrifice_output.into_iter().nth(1).unwrap() {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_quantity(), 1);
					assert_eq!(wrapped.get_item_type(), ItemType::Pet);
					assert_eq!(wrapped.get_item_sub_type::<PetItemType>(), PetItemType::Egg);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_unidentified() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x0C, 0x0B, 0x1B, 0x95, 0x61, 0x50, 0xAF, 0xFB, 0xCD, 0x39, 0x9F, 0x55, 0x88, 0x2D,
				0xAB, 0x46, 0xDF, 0x40, 0x9A, 0x32, 0x27, 0x33, 0xBB, 0x80, 0x5F, 0xD6, 0x45, 0xA0,
				0xFB, 0xE4, 0xE0, 0x79,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader = create_random_glimmer(&ALICE, 10);
			let sacrifice = create_random_material(&ALICE, &MaterialItemType::Polymers, 8);

			let total_soul_points = leader.1.get_souls() + sacrifice.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
				leader,
				vec![sacrifice],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let output_souls = sacrifice_output
					.iter()
					.map(|sacrifice| match sacrifice {
						ForgeOutput::Forged((_, avatar), _) => avatar.souls,
						ForgeOutput::Minted(avatar) => avatar.souls,
						_ => 0,
					})
					.sum::<SoulCount>() + leader_avatar.souls;
				assert_eq!(output_souls, total_soul_points);

				if let ForgeOutput::Minted(avatar) = sacrifice_output.into_iter().nth(1).unwrap() {
					let wrapped = WrappedAvatar::new(avatar);
					assert_eq!(wrapped.get_quantity(), 1);
					assert_eq!(wrapped.get_item_type(), ItemType::Special);
					assert_eq!(
						wrapped.get_item_sub_type::<SpecialItemType>(),
						SpecialItemType::Unidentified
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_glimmer_probability_test() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let mut probability_array = [0_u32; 8];

			for i in 0..10_000 {
				let leader = create_random_glimmer(&ALICE, 150);
				let sacrifice_1 = create_random_material(&ALICE, &MaterialItemType::Polymers, 20);
				let sacrifice_2 =
					create_random_material(&ALICE, &MaterialItemType::Superconductors, 20);
				let sacrifice_3 =
					create_random_material(&ALICE, &MaterialItemType::Superconductors, 20);
				let sacrifice_4 = create_random_material(&ALICE, &MaterialItemType::Ceramics, 20);

				let (_leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
					leader,
					vec![sacrifice_1, sacrifice_2, sacrifice_3, sacrifice_4],
					0,
					&mut hash_provider,
				)
				.expect("Should succeed in forging");

				probability_array[0] += 1;
				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					match DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType) {
						ItemType::Pet => probability_array[1] += 1,
						ItemType::Essence => match DnaUtils::read_attribute::<EssenceItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						) {
							EssenceItemType::ColorSpark => probability_array[2] += 1,
							EssenceItemType::GlowSpark => probability_array[3] += 1,
							_ => panic!("Generated avatar EssenceItemType not valid!"),
						},
						ItemType::Special => match DnaUtils::read_attribute::<SpecialItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						) {
							SpecialItemType::Dust => probability_array[4] += 1,
							SpecialItemType::Unidentified => probability_array[5] += 1,
							SpecialItemType::Fragment => probability_array[6] += 1,
							SpecialItemType::ToolBox => probability_array[7] += 1,
						},
						_ => panic!("Generated avatar ItemType not valid!"),
					}
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
				hash_provider = HashProvider::new(&hash_provider.full_hash((i * 13) % 31));
			}

			assert_eq!(probability_array[0], 10_000);
			assert_eq!(probability_array[1], 9);
			assert_eq!(probability_array[2], 797);
			assert_eq!(probability_array[3], 723);
			assert_eq!(probability_array[4], 6965);
			assert_eq!(probability_array[5], 6);
			assert_eq!(probability_array[6], 0);
			assert_eq!(probability_array[7], 1500);
		});
	}

	#[test]
	fn test_glimmer_gamble_prep_1() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let hash_base = [
				[
					0x31, 0x00, 0x12, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x24, 0x00, 0x11, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x27, 0x00, 0x11, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x21, 0x00, 0x11, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
				[
					0x25, 0x00, 0x11, 0xf3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				],
			];

			let unit_fn = |avatar: Avatar| {
				let mut avatar = avatar;
				avatar.souls = 100;
				WrappedAvatar::new(avatar)
			};

			let mut probability_array = [0_u32; 8];

			for i in 0..10_000 {
				let leader =
					create_random_avatar::<Test, _>(&ALICE, Some(hash_base[0]), Some(unit_fn));
				let sac_1 =
					create_random_avatar::<Test, _>(&ALICE, Some(hash_base[1]), Some(unit_fn));
				let sac_2 =
					create_random_avatar::<Test, _>(&ALICE, Some(hash_base[2]), Some(unit_fn));
				let sac_3 =
					create_random_avatar::<Test, _>(&ALICE, Some(hash_base[3]), Some(unit_fn));
				let sac_4 =
					create_random_avatar::<Test, _>(&ALICE, Some(hash_base[4]), Some(unit_fn));

				let (_leader_output, sacrifice_output) = AvatarCombinator::<Test>::glimmer_avatars(
					leader,
					vec![sac_1, sac_2, sac_3, sac_4],
					0,
					&mut hash_provider,
				)
				.expect("Should succeed in forging");

				probability_array[0] += 1;
				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					match DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType) {
						ItemType::Pet => probability_array[1] += 1,
						ItemType::Essence => match DnaUtils::read_attribute::<EssenceItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						) {
							EssenceItemType::ColorSpark => probability_array[2] += 1,
							EssenceItemType::GlowSpark => probability_array[3] += 1,
							_ => panic!("Generated avatar EssenceItemType not valid!"),
						},
						ItemType::Special => match DnaUtils::read_attribute::<SpecialItemType>(
							avatar,
							AvatarAttr::ItemSubType,
						) {
							SpecialItemType::Dust => probability_array[4] += 1,
							SpecialItemType::Unidentified => probability_array[5] += 1,
							SpecialItemType::Fragment => probability_array[6] += 1,
							SpecialItemType::ToolBox => probability_array[7] += 1,
						},
						_ => panic!("Generated avatar ItemType not valid!"),
					}
				} else {
					panic!("ForgeOutput should have been Minted!")
				}

				let hash_text = format!("hash_loop_{:#07X}", i);
				let hash = Pallet::<Test>::random_hash(hash_text.as_bytes(), &ALICE);
				hash_provider = HashProvider::new(&hash);
			}

			assert_eq!(probability_array[0], 10_000);
			// Total = 0.09 + 7.86 + 7.17 + 84.82 + 0.06 --> ~100%
			assert_eq!(probability_array[1], 9); // Egg ~0.09%
			assert_eq!(probability_array[2], 786); // ColorSpark ~7.86%
			assert_eq!(probability_array[3], 717); // GlowSpark ~7.17%
			assert_eq!(probability_array[4], 8482); // Dust ~84.82%
			assert_eq!(probability_array[5], 6); // Unidentified ~0.06%
			assert_eq!(probability_array[6], 0); // Fragment ~0%
			assert_eq!(probability_array[7], 0); // Toolbox ~0%
		});
	}
}
