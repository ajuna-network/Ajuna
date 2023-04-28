use super::*;

impl<'a, T> AvatarCombinator<'a, T>
where
	T: Config,
{
	pub(super) fn stack_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let (new_quantity, new_souls) = input_sacrifices
			.iter()
			.map(|sacrifice| {
				(
					AvatarUtils::read_attribute(&sacrifice.1, AvatarAttributes::Quantity),
					sacrifice.1.souls,
				)
			})
			.reduce(|(acc_qty, acc_souls), (qty, souls)| {
				(acc_qty.saturating_add(qty), acc_souls.saturating_add(souls))
			})
			.unwrap_or_default();
		let leader_quantity = AvatarUtils::read_attribute(&leader, AvatarAttributes::Quantity)
			.saturating_add(new_quantity);
		AvatarUtils::write_attribute(&mut leader, AvatarAttributes::Quantity, leader_quantity);

		let mut glimmer_avatar: Option<Avatar> = None;
		let mut glimmer_additional_qty = 0;

		for i in 0..input_sacrifices.len() {
			if hash_provider.hash[i] as u32 * SCALING_FACTOR_PERC < STACK_PROB_PERC * MAX_BYTE {
				match glimmer_avatar {
					None => {
						let dna = AvatarMinterV2::<T>(PhantomData)
							.generate_base_avatar_dna(hash_provider, i)?;
						glimmer_avatar =
							Some(AvatarBuilder::with_dna(season_id, dna).into_glimmer(1).build());
					},
					Some(_) => {
						glimmer_additional_qty += 1;
					},
				}
			}
		}

		if let Some(ref mut avatar) = glimmer_avatar {
			AvatarUtils::write_attribute(
				avatar,
				AvatarAttributes::Quantity,
				glimmer_additional_qty,
			);
		}

		leader.souls += new_souls;

		let output_vec: Vec<ForgeOutput<T>> = input_sacrifices
			.into_iter()
			.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id))
			.chain(glimmer_avatar.map(|minted_avatar| ForgeOutput::Minted(minted_avatar)))
			.collect();

		Ok((LeaderForgeOutput::Forged((leader_id, leader), 0), output_vec))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_stack_material() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let material_input_1 = create_random_material(&ALICE, MaterialItemType::Polymers, 1);
			let material_input_2 = create_random_material(&ALICE, MaterialItemType::Polymers, 2);
			let material_input_3 = create_random_material(&ALICE, MaterialItemType::Polymers, 5);
			let material_input_4 = create_random_material(&ALICE, MaterialItemType::Polymers, 3);

			let total_soul_points = material_input_1.1.souls +
				material_input_2.1.souls +
				material_input_3.1.souls +
				material_input_4.1.souls;

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				material_input_1,
				vec![material_input_2, material_input_3, material_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert!(sacrifice_output.iter().all(|output| !is_forged(output)));
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, AvatarAttributes::Quantity),
					11
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_pet_parts() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let pet_part_input_1 =
				create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::Head, 3);
			let pet_part_input_2 =
				create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::ArmBack, 4);
			let pet_part_input_3 =
				create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::LegBack, 5);
			let pet_part_input_4 =
				create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::LegFront, 5);

			let total_soul_points = pet_part_input_1.1.souls +
				pet_part_input_2.1.souls +
				pet_part_input_3.1.souls +
				pet_part_input_4.1.souls;

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				pet_part_input_1,
				vec![pet_part_input_2, pet_part_input_3, pet_part_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert!(sacrifice_output.iter().all(|output| !is_forged(output)));
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_soul_points);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, AvatarAttributes::Quantity),
					17
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
