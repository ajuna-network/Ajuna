use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn mate_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		if input_sacrifices.len() != 1 {
			return Ok((
				LeaderForgeOutput::Forged(input_leader, 0),
				input_sacrifices
					.into_iter()
					.map(|input| ForgeOutput::Forged(input, 0))
					.collect(),
			))
		}

		let (leader_id, mut leader) = input_leader;
		let (partner_id, mut partner) = {
			let mut input_sacrifices = input_sacrifices;
			input_sacrifices.pop().unwrap()
		};

		if AvatarUtils::spec_byte_split_ten_count(&leader) < MAX_EQUIPPED_SLOTS {
			return Ok((
				LeaderForgeOutput::Forged((leader_id, leader), 0),
				vec![ForgeOutput::Forged((partner_id, partner), 0)],
			))
		}
		if AvatarUtils::spec_byte_split_ten_count(&partner) < MAX_EQUIPPED_SLOTS {
			leader.souls += partner.souls;

			return Ok((
				LeaderForgeOutput::Forged((leader_id, leader), 0),
				vec![ForgeOutput::Consumed(partner_id)],
			))
		}

		let (mirrors, _) = AvatarUtils::match_progress_arrays(
			AvatarUtils::read_progress_array(&leader),
			AvatarUtils::read_progress_array(&partner),
		);

		if mirrors < 4 {
			leader.souls += partner.souls;

			return Ok((
				LeaderForgeOutput::Forged((leader_id, leader), 0),
				vec![ForgeOutput::Consumed(partner_id)],
			))
		}

		let leader_pet_type =
			AvatarUtils::enums_to_bits(&[AvatarUtils::read_attribute_as::<PetType>(
				&leader,
				&AvatarAttributes::ClassType2,
			)]) as u8;

		let leader_pet_variation =
			AvatarUtils::read_attribute(&leader, &AvatarAttributes::CustomType2);
		let partner_pet_variation =
			AvatarUtils::read_attribute(&partner, &AvatarAttributes::CustomType2);

		let legendary_egg_flag = ((hash_provider.hash[0] | hash_provider.hash[1]) == 0x7F) &&
			((leader_pet_variation | partner_pet_variation) == 0x7F);

		let random_pet_variation = hash_provider.hash[0] & hash_provider.hash[1] & 0b0111_1111;

		let pet_variation = if !legendary_egg_flag {
			(leader_pet_variation ^ partner_pet_variation) | leader_pet_type | random_pet_variation
		} else {
			0x00
		};

		let soul_points = (hash_provider.hash[0] | hash_provider.hash[1] & 0x7F) as SoulCount;

		let (leader_output, other_output, any_survived) = if partner.souls > soul_points {
			partner.souls -= soul_points;
			(
				LeaderForgeOutput::Forged((leader_id, leader), 0),
				vec![ForgeOutput::Forged((partner_id, partner), 0)],
				true,
			)
		} else if leader.souls + partner.souls > soul_points {
			leader.souls -= soul_points - partner.souls;
			(
				LeaderForgeOutput::Forged((leader_id, leader), 0),
				vec![ForgeOutput::Consumed(partner_id)],
				true,
			)
		} else {
			let generated_dust = {
				let dna = MinterV2::<T>::generate_base_avatar_dna(hash_provider, 0)?;

				AvatarBuilder::with_dna(season_id, dna)
					.into_dust(leader.souls + partner.souls)
					.build()
			};
			(
				LeaderForgeOutput::Consumed(leader_id),
				vec![ForgeOutput::Consumed(partner_id), ForgeOutput::Minted(generated_dust)],
				false,
			)
		};

		let additional_output = any_survived
			.then(|| {
				let dna = MinterV2::<T>::generate_base_avatar_dna(hash_provider, 10)?;
				let generated_egg = AvatarBuilder::with_dna(season_id, dna)
					.into_egg(&RarityTier::Rare, pet_variation, soul_points, None)
					.build();
				Ok::<_, DispatchError>(ForgeOutput::Minted(generated_egg))
			})
			.transpose()?;

		Ok((leader_output, other_output.into_iter().chain(additional_output.into_iter()).collect()))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_mate() {
		ExtBuilder::default().build().execute_with(|| {
			let forge_hash = [
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			];
			let mut hash_provider = HashProvider::new_with_bytes(forge_hash);

			let leader_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54, 0x51, 0x53, 0x55, 0x53];
			let leader_spec_bytes = [
				0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x75, 0x97, 0x50,
				0x00, 0x00,
			];

			let partner_progress_array =
				[0x53, 0x54, 0x51, 0x52, 0x54, 0x50, 0x50, 0x53, 0x55, 0x53, 0x51];
			let partner_spec_bytes = [
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
			let partner = create_random_pet(
				&ALICE,
				&PetType::CrazyDude,
				0b0101_0011,
				partner_spec_bytes,
				partner_progress_array,
				1000,
			);

			let expected_dna = [
				0x11, 0x05, 0x05, 0x01, 0x19, 0x97, 0x59, 0x75, 0x97, 0x50, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x09, 0x75, 0x97, 0x50, 0x00, 0x00, 0x53, 0x54, 0x51, 0x52, 0x55, 0x55, 0x54,
				0x51, 0x53, 0x55, 0x53,
			];
			assert_eq!(leader.1.dna.as_slice(), &expected_dna);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::mate_avatars(
				leader,
				vec![partner],
				0,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 1000);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}

			if let ForgeOutput::Forged((_, avatar), _) = &sacrifice_output[0] {
				assert_eq!(avatar.souls, 878);
			} else {
				panic!("ForgeOutput for first output should have been Forged!")
			}

			if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
				assert_eq!(avatar.souls, 122);
				assert_eq!(
					AvatarUtils::read_attribute_as::<PetItemType>(
						avatar,
						&AvatarAttributes::ItemSubType
					),
					PetItemType::Egg
				);
				assert_eq!(
					AvatarUtils::read_attribute(avatar, &AvatarAttributes::CustomType2),
					0b0101_1010
				);
			} else {
				panic!("ForgeOutput for second output should have been Minted!")
			}
		});
	}
}
