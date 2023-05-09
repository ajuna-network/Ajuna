use super::*;

impl<'a, T> AvatarCombinator<'a, T>
where
	T: Config,
{
	pub(super) fn spark_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let all_count = input_sacrifices.len();

		let (
			(leader_id, mut leader),
			matching_sacrifices,
			consumed_sacrifices,
			non_matching_sacrifices,
		) = Self::match_avatars(input_leader, input_sacrifices, hash_provider);

		let rarity_type = RarityType::from_byte(AvatarUtils::read_lowest_progress_byte(
			&AvatarUtils::read_progress_array(&leader),
			ByteType::High,
		));
		let essence_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
			&leader,
			AvatarAttributes::ItemSubType,
		);

		if essence_type == EssenceItemType::ColorSpark && rarity_type == RarityType::Rare {
			let rolls = MAX_SACRIFICE - all_count + 1;

			for _ in 0..rolls {
				let rand = hash_provider.get_hash_byte() as usize;
				if rand > 150 {
					let (_, sacrifice) = &matching_sacrifices[rand % all_count];
					let spec_byte_index = if rand > 200 {
						AvatarSpecBytes::SpecByte1
					} else {
						AvatarSpecBytes::SpecByte2
					};
					AvatarUtils::write_spec_byte(
						&mut leader,
						spec_byte_index.clone(),
						AvatarUtils::read_spec_byte(sacrifice, spec_byte_index),
					);
				}

				let leader_spec_byte_1 =
					AvatarUtils::read_spec_byte(&leader, AvatarSpecBytes::SpecByte1);
				let leader_spec_byte_2 =
					AvatarUtils::read_spec_byte(&leader, AvatarSpecBytes::SpecByte2);
				let color_bits = ((leader_spec_byte_1 - 1) << 6) | (leader_spec_byte_2 - 1) << 4;

				AvatarUtils::write_spec_byte(&mut leader, AvatarSpecBytes::SpecByte1, color_bits);
				AvatarUtils::write_spec_byte(&mut leader, AvatarSpecBytes::SpecByte2, 0x00);

				AvatarUtils::write_typed_attribute(
					&mut leader,
					AvatarAttributes::ItemSubType,
					EssenceItemType::PaintFlask,
				);
			}
		} else if essence_type == EssenceItemType::GlowSpark && rarity_type == RarityType::Epic {
			AvatarUtils::write_typed_attribute(
				&mut leader,
				AvatarAttributes::ItemSubType,
				EssenceItemType::ForceGlow,
			);
		}

		AvatarUtils::write_typed_attribute(&mut leader, AvatarAttributes::RarityType, rarity_type);

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

			let progress_arrays: [[u8; 11]; 5] = [
				[0x34, 0x35, 0x23, 0x30, 0x30, 0x21, 0x21, 0x34, 0x34, 0x23, 0x32],
				[0x22, 0x35, 0x22, 0x30, 0x23, 0x23, 0x25, 0x25, 0x24, 0x30, 0x22],
				[0x24, 0x20, 0x24, 0x24, 0x33, 0x21, 0x20, 0x20, 0x34, 0x23, 0x25],
				[0x22, 0x21, 0x24, 0x20, 0x33, 0x21, 0x23, 0x24, 0x22, 0x22, 0x22],
				[0x34, 0x23, 0x24, 0x22, 0x25, 0x22, 0x21, 0x24, 0x20, 0x31, 0x21],
			];

			let mut avatars = Vec::with_capacity(5);

			for progress_array in progress_arrays {
				avatars.push(create_random_color_spark(
					None,
					&ALICE,
					(ColorType::ColorA, ColorType::ColorC),
					5,
					Some(progress_array),
				));
			}

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
					[0x34, 0x35, 0x33, 0x30, 0x30, 0x31, 0x31, 0x34, 0x34, 0x33, 0x32];
				let _leader_progress_array = AvatarUtils::read_progress_array(&leader_avatar);
				assert_eq!(
					AvatarUtils::read_progress_array(&leader_avatar),
					expected_progress_array
				);
				assert_eq!(
					AvatarUtils::read_attribute_as::<RarityType>(
						&leader_avatar,
						AvatarAttributes::RarityType
					),
					RarityType::Rare
				);
				assert_eq!(
					AvatarUtils::read_attribute_as::<EssenceItemType>(
						&leader_avatar,
						AvatarAttributes::ItemSubType
					),
					EssenceItemType::PaintFlask
				);
				assert_eq!(
					AvatarUtils::read_spec_byte(&leader_avatar, AvatarSpecBytes::SpecByte1),
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

			let force_types = [
				ForceType::Kinetic,
				ForceType::Thermal,
				ForceType::Thermal,
				ForceType::Thermal,
				ForceType::Kinetic,
			];

			let mut avatars = force_types
				.into_iter()
				.zip(progress_arrays)
				.map(|(force_type, progress_array)| {
					create_random_glow_spark(None, &ALICE, force_type, 5, Some(progress_array))
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
				let _leader_progress_array = AvatarUtils::read_progress_array(&leader_avatar);
				assert_eq!(
					AvatarUtils::read_progress_array(&leader_avatar),
					expected_progress_array
				);
				assert_eq!(
					AvatarUtils::read_attribute_as::<RarityType>(
						&leader_avatar,
						AvatarAttributes::RarityType
					),
					RarityType::Epic
				);
				assert_eq!(
					AvatarUtils::read_attribute_as::<EssenceItemType>(
						&leader_avatar,
						AvatarAttributes::ItemSubType
					),
					EssenceItemType::ForceGlow
				);
				assert_eq!(
					ForceType::from_byte(AvatarUtils::read_spec_byte(
						&leader_avatar,
						AvatarSpecBytes::SpecByte1
					)),
					ForceType::Kinetic
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
