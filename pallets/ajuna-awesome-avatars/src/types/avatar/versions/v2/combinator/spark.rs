use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn spark_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let all_count = input_sacrifices.len();

		let ((leader_id, mut leader), sacrifices) = Self::match_avatars(
			input_leader,
			input_sacrifices,
			MATCH_ALGO_START_RARITY.as_byte(),
			hash_provider,
		);

		let progress_rarity = RarityTier::from_byte(DnaUtils::lowest_progress_byte(
			&leader.get_progress(),
			ByteType::High,
		));
		let essence_type = leader.get_item_sub_type::<EssenceItemType>();

		if essence_type == EssenceItemType::ColorSpark && progress_rarity == RarityTier::Epic {
			let rolls = MAX_SACRIFICE - all_count + 1;

			for _ in 0..rolls {
				let rand = hash_provider.next() as usize;
				if rand > 150 {
					let (_, sacrifice) = &sacrifices[rand % all_count];
					let spec_byte_index = if rand > 200 { SpecIdx::Byte1 } else { SpecIdx::Byte2 };
					leader.set_spec(spec_byte_index, sacrifice.get_spec(spec_byte_index));
				}

				let leader_spec_byte_1 = leader.get_spec::<u8>(SpecIdx::Byte1);
				let leader_spec_byte_2 = leader.get_spec::<u8>(SpecIdx::Byte2);

				let color_bits = ((leader_spec_byte_1 - 1) << 6) | (leader_spec_byte_2 - 1) << 4;

				leader.set_spec(SpecIdx::Byte1, color_bits);
				leader.set_spec(SpecIdx::Byte2, 0b0000_1000);

				leader.set_item_sub_type(EssenceItemType::PaintFlask);
			}
		} else if essence_type == EssenceItemType::GlowSpark && progress_rarity == RarityTier::Epic
		{
			leader.set_item_sub_type(EssenceItemType::GlowFlask);
		}

		leader.set_rarity(progress_rarity);

		let output_vec: Vec<ForgeOutput<T>> = sacrifices
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
			assert_eq!(leader.1.get_dna().as_slice(), &expected_dna);

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::spark_avatars(leader, sacrifices, &mut hash_provider)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);

				let expected_progress_array =
					[0x44, 0x45, 0x43, 0x40, 0x40, 0x41, 0x41, 0x44, 0x44, 0x43, 0x42];
				let leader_progress_array = DnaUtils::read_progress(&leader_avatar);
				assert_eq!(leader_progress_array, expected_progress_array);
				assert_eq!(
					DnaUtils::read_attribute::<RarityTier>(&leader_avatar, AvatarAttr::RarityTier),
					RarityTier::Epic
				);
				assert_eq!(
					DnaUtils::read_attribute::<EssenceItemType>(
						&leader_avatar,
						AvatarAttr::ItemSubType
					),
					EssenceItemType::PaintFlask
				);
				assert_eq!(DnaUtils::read_spec_raw(&leader_avatar, SpecIdx::Byte1), 0b0010_0000);
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
				let leader_progress_array = DnaUtils::read_progress(&leader_avatar);
				assert_eq!(leader_progress_array, expected_progress_array);
				assert_eq!(
					DnaUtils::read_attribute::<RarityTier>(&leader_avatar, AvatarAttr::RarityTier),
					RarityTier::Epic
				);
				assert_eq!(
					DnaUtils::read_attribute::<EssenceItemType>(
						&leader_avatar,
						AvatarAttr::ItemSubType
					),
					EssenceItemType::GlowFlask
				);
				assert_eq!(
					Force::from_byte(DnaUtils::read_spec_raw(&leader_avatar, SpecIdx::Byte1)),
					Force::Kinetic
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
