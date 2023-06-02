use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn spark_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let all_count = input_sacrifices.len();

		let ((leader_id, mut leader), sacrifices) = Self::match_avatars(
			input_leader,
			input_sacrifices,
			MATCH_ALGO_START_RARITY.as_byte(),
			hash_provider,
		);

		let rarity = RarityTier::from_byte(AvatarUtils::read_lowest_progress_byte(
			&AvatarUtils::read_progress_array(&leader),
			&ByteType::High,
		));
		let essence_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
			&leader,
			&AvatarAttributes::ItemSubType,
		);

		if essence_type == EssenceItemType::ColorSpark && rarity == RarityTier::Epic {
			let rolls = MAX_SACRIFICE - all_count + 1;

			for _ in 0..rolls {
				let rand = hash_provider.get_hash_byte() as usize;
				if rand > 150 {
					let (_, sacrifice) = &sacrifices[rand % all_count];
					let spec_byte_index = if rand > 200 {
						AvatarSpecBytes::SpecByte1
					} else {
						AvatarSpecBytes::SpecByte2
					};
					AvatarUtils::write_spec_byte(
						&mut leader,
						&spec_byte_index,
						AvatarUtils::read_spec_byte(sacrifice, &spec_byte_index),
					);
				}

				let leader_spec_byte_1 =
					AvatarUtils::read_spec_byte(&leader, &AvatarSpecBytes::SpecByte1);
				let leader_spec_byte_2 =
					AvatarUtils::read_spec_byte(&leader, &AvatarSpecBytes::SpecByte2);
				let color_bits = ((leader_spec_byte_1 - 1) << 6) | (leader_spec_byte_2 - 1) << 4;

				AvatarUtils::write_spec_byte(&mut leader, &AvatarSpecBytes::SpecByte1, color_bits);
				AvatarUtils::write_spec_byte(&mut leader, &AvatarSpecBytes::SpecByte2, 0b0000_1000);

				AvatarUtils::write_typed_attribute(
					&mut leader,
					&AvatarAttributes::ItemSubType,
					&EssenceItemType::PaintFlask,
				);
			}
		} else if essence_type == EssenceItemType::GlowSpark && rarity == RarityTier::Epic {
			AvatarUtils::write_typed_attribute(
				&mut leader,
				&AvatarAttributes::ItemSubType,
				&EssenceItemType::GlowFlask,
			);
		}

		AvatarUtils::write_typed_attribute(&mut leader, &AvatarAttributes::RarityTier, &rarity);

		let output_vec: Vec<ForgeOutput<T>> = sacrifices
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
	fn test_spark_to_paint() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let progress_arrays = [
				[0x44, 0x45, 0x33, 0x40, 0x40, 0x31, 0x31, 0x44, 0x44, 0x33, 0x42],
				[0x32, 0x45, 0x32, 0x40, 0x33, 0x33, 0x35, 0x35, 0x34, 0x40, 0x32],
				[0x34, 0x30, 0x34, 0x34, 0x43, 0x31, 0x30, 0x30, 0x44, 0x33, 0x35],
				[0x32, 0x31, 0x34, 0x30, 0x43, 0x31, 0x33, 0x34, 0x32, 0x32, 0x32],
				[0x44, 0x33, 0x34, 0x32, 0x35, 0x32, 0x31, 0x34, 0x30, 0x41, 0x31],
			];

			let mut avatars = progress_arrays
				.into_iter()
				.map(|progress_array| {
					create_random_color_spark(
						None,
						&ALICE,
						&(ColorType::ColorA, ColorType::ColorC),
						5,
						progress_array,
					)
				})
				.collect::<Vec<_>>();

			let total_soul_points = 25;

			let sacrifices = avatars.split_off(1);
			let leader = avatars.pop().unwrap();

			let expected_dna = [
				0x32, 0x00, 0x03, 0x01, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x44, 0x45, 0x33, 0x40, 0x40, 0x31, 0x31,
				0x44, 0x44, 0x33, 0x42,
			];
			assert_eq!(leader.1.dna.as_slice(), &expected_dna);

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::spark_avatars(leader, sacrifices, &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);

				let expected_progress_array =
					[0x44, 0x45, 0x43, 0x40, 0x40, 0x41, 0x41, 0x44, 0x44, 0x43, 0x42];
				let leader_progress_array = AvatarUtils::read_progress_array(&leader_avatar);
				assert_eq!(leader_progress_array, expected_progress_array);
				assert_eq!(
					AvatarUtils::read_attribute_as::<RarityTier>(
						&leader_avatar,
						&AvatarAttributes::RarityTier
					),
					RarityTier::Epic
				);
				assert_eq!(
					AvatarUtils::read_attribute_as::<EssenceItemType>(
						&leader_avatar,
						&AvatarAttributes::ItemSubType
					),
					EssenceItemType::PaintFlask
				);
				assert_eq!(
					AvatarUtils::read_spec_byte(&leader_avatar, &AvatarSpecBytes::SpecByte1),
					0b0010_0000
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_spark_to_force() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let progress_arrays = [
				[0x44, 0x45, 0x33, 0x40, 0x40, 0x31, 0x31, 0x44, 0x44, 0x33, 0x42],
				[0x32, 0x45, 0x32, 0x40, 0x33, 0x33, 0x35, 0x35, 0x34, 0x40, 0x32],
				[0x34, 0x30, 0x34, 0x34, 0x43, 0x31, 0x30, 0x30, 0x44, 0x33, 0x35],
				[0x32, 0x31, 0x34, 0x30, 0x43, 0x31, 0x33, 0x34, 0x32, 0x32, 0x32],
				[0x44, 0x33, 0x34, 0x32, 0x35, 0x32, 0x31, 0x34, 0x30, 0x41, 0x31],
			];

			let forces =
				[Force::Kinetic, Force::Thermal, Force::Thermal, Force::Thermal, Force::Kinetic];

			let mut avatars = forces
				.into_iter()
				.zip(progress_arrays)
				.map(|(force, progress_array)| {
					create_random_glow_spark(None, &ALICE, &force, 5, progress_array)
				})
				.collect::<Vec<_>>();

			let total_soul_points = 25;

			let sacrifices = avatars.split_off(1);
			let leader = avatars.pop().unwrap();

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::spark_avatars(leader, sacrifices, &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);

				let expected_progress_array =
					[0x44, 0x45, 0x43, 0x40, 0x40, 0x41, 0x41, 0x44, 0x44, 0x43, 0x42];
				let leader_progress_array = AvatarUtils::read_progress_array(&leader_avatar);
				assert_eq!(leader_progress_array, expected_progress_array);
				assert_eq!(
					AvatarUtils::read_attribute_as::<RarityTier>(
						&leader_avatar,
						&AvatarAttributes::RarityTier
					),
					RarityTier::Epic
				);
				assert_eq!(
					AvatarUtils::read_attribute_as::<EssenceItemType>(
						&leader_avatar,
						&AvatarAttributes::ItemSubType
					),
					EssenceItemType::GlowFlask
				);
				assert_eq!(
					Force::from_byte(AvatarUtils::read_spec_byte(
						&leader_avatar,
						&AvatarSpecBytes::SpecByte1
					)),
					Force::Kinetic
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
