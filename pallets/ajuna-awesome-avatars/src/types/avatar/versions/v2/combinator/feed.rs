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
				let (sacrifice_souls, other_output) = input_sacrifices.into_iter().fold(
					(0, Vec::with_capacity(sacrifice_count)),
					|(souls, mut items), (sacrifice_id, sacrifice)| {
						items.push(ForgeOutput::Consumed(sacrifice_id));
						(souls + sacrifice.get_souls(), items)
					},
				);

				let (leader_id, mut leader) = input_leader;
				leader.add_souls(sacrifice_souls);

				Ok((LeaderForgeOutput::Forged((leader_id, leader.unwrap()), 0), other_output))
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
}
