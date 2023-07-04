use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn feed_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		match input_sacrifices.len() {
			0 => Ok((
				LeaderForgeOutput::Forged((input_leader.0, input_leader.1.unwrap()), 0),
				Vec::with_capacity(0),
			)),
			sacrifice_count => {
				let (leader_id, mut leader) = input_leader;
				let mut sacrifice_output = Vec::with_capacity(sacrifice_count);

				for (sacrifice_id, sacrifice) in input_sacrifices.into_iter() {
					leader.add_souls(sacrifice.get_souls());

					let spec_16 = leader.get_spec::<u8>(SpecIdx::Byte16);

					if sacrifice.get_rarity() == RarityTier::Legendary && spec_16 < 11 {
						let mut progress = leader.get_progress();
						progress[spec_16 as usize] &= 0x0F;
						progress[spec_16 as usize] |= RarityTier::Mythical.as_byte() << 4;
						leader.set_progress(progress);
						leader.set_spec(SpecIdx::Byte16, spec_16 + 1);
						if spec_16 == 10 {
							leader.set_rarity(RarityTier::Mythical);
						}
					}

					sacrifice_output.push(ForgeOutput::Consumed(sacrifice_id));
				}

				Ok((LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0), sacrifice_output))
			},
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_feed() {
		ExtBuilder::default().build().execute_with(|| {
			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let leader = create_random_pet(
				&ALICE,
				&PetType::BigHybrid,
				0b0001_1001,
				leader_spec_bytes,
				leader_progress_array,
				1000,
			);
			let sacrifice_1 =
				create_random_egg(None, &ALICE, &RarityTier::Epic, 0b0000_1111, 100, [0x00; 11]);
			let sacrifice_2 =
				create_random_egg(None, &ALICE, &RarityTier::Epic, 0b0000_1111, 100, [0x00; 11]);

			let total_soul_points =
				leader.1.get_souls() + sacrifice_1.1.get_souls() + sacrifice_2.1.get_souls();

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::feed_avatars(leader, vec![sacrifice_1, sacrifice_2])
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_feed_prep_1() {
		ExtBuilder::default().build().execute_with(|| {
			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let leader = create_random_pet(
				&ALICE,
				&PetType::BigHybrid,
				0b0001_1001,
				leader_spec_bytes,
				leader_progress_array,
				1000,
			);

			let sacrifice_1 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let total_soul_points = leader.1.get_souls() + sacrifice_1.1.get_souls();

			assert_eq!(leader.1.get_spec::<u8>(SpecIdx::Byte16), 0);

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::feed_avatars(leader, vec![sacrifice_1])
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);

			let leader = if let LeaderForgeOutput::Forged((leader_id, leader_avatar), _) =
				leader_output
			{
				assert_eq!(leader_avatar.souls, total_soul_points);
				assert_eq!(DnaUtils::read_spec::<u8>(&leader_avatar, SpecIdx::Byte16), 1);
				assert_eq!(
					DnaUtils::read_attribute::<RarityTier>(&leader_avatar, AvatarAttr::RarityTier),
					RarityTier::Legendary
				);
				let expected_progress_array =
					[0x63, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
				assert_eq!(DnaUtils::read_progress(&leader_avatar), expected_progress_array);

				(leader_id, WrappedAvatar::new(leader_avatar))
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			};

			let sacrifice_2 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_3 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_4 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_5 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let total_soul_points = leader.1.get_souls() +
				sacrifice_2.1.get_souls() +
				sacrifice_3.1.get_souls() +
				sacrifice_4.1.get_souls() +
				sacrifice_5.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::feed_avatars(
				leader,
				vec![sacrifice_2, sacrifice_3, sacrifice_4, sacrifice_5],
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);

			let leader = if let LeaderForgeOutput::Forged((leader_id, leader_avatar), _) =
				leader_output
			{
				assert_eq!(leader_avatar.souls, total_soul_points);
				assert_eq!(DnaUtils::read_spec::<u8>(&leader_avatar, SpecIdx::Byte16), 5);
				assert_eq!(
					DnaUtils::read_attribute::<RarityTier>(&leader_avatar, AvatarAttr::RarityTier),
					RarityTier::Legendary
				);
				let expected_progress_array =
					[0x63, 0x64, 0x61, 0x62, 0x65, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
				assert_eq!(DnaUtils::read_progress(&leader_avatar), expected_progress_array);

				(leader_id, WrappedAvatar::new(leader_avatar))
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			};

			let sacrifice_6 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_7 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_8 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_9 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let total_soul_points = leader.1.get_souls() +
				sacrifice_6.1.get_souls() +
				sacrifice_7.1.get_souls() +
				sacrifice_8.1.get_souls() +
				sacrifice_9.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::feed_avatars(
				leader,
				vec![sacrifice_6, sacrifice_7, sacrifice_8, sacrifice_9],
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);

			let leader = if let LeaderForgeOutput::Forged((leader_id, leader_avatar), _) =
				leader_output
			{
				assert_eq!(leader_avatar.souls, total_soul_points);
				assert_eq!(DnaUtils::read_spec::<u8>(&leader_avatar, SpecIdx::Byte16), 9);
				assert_eq!(
					DnaUtils::read_attribute::<RarityTier>(&leader_avatar, AvatarAttr::RarityTier),
					RarityTier::Legendary
				);
				let expected_progress_array =
					[0x63, 0x64, 0x61, 0x62, 0x65, 0x65, 0x64, 0x61, 0x63, 0x55, 0x53];
				assert_eq!(DnaUtils::read_progress(&leader_avatar), expected_progress_array);

				(leader_id, WrappedAvatar::new(leader_avatar))
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			};

			let sacrifice_10 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let sacrifice_11 = create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				100,
				[0x00; 11],
			);

			let total_soul_points =
				leader.1.get_souls() + sacrifice_10.1.get_souls() + sacrifice_11.1.get_souls();

			let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::feed_avatars(leader, vec![sacrifice_10, sacrifice_11])
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);
				assert_eq!(DnaUtils::read_spec::<u8>(&leader_avatar, SpecIdx::Byte16), 11);
				assert_eq!(
					DnaUtils::read_attribute::<RarityTier>(&leader_avatar, AvatarAttr::RarityTier),
					RarityTier::Mythical
				);
				let expected_progress_array =
					[0x63, 0x64, 0x61, 0x62, 0x65, 0x65, 0x64, 0x61, 0x63, 0x65, 0x63];
				assert_eq!(DnaUtils::read_progress(&leader_avatar), expected_progress_array);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			};
		});
	}
}
