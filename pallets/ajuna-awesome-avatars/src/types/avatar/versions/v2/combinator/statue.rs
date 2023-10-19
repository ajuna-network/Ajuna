use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn statue_avatars(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
		block_number: BlockNumberFor<T>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let (mut new_quantity, mut new_souls) = input_sacrifices
			.iter()
			.map(|(_, sacrifice)| (sacrifice.get_quantity() as u32, sacrifice.get_souls()))
			.reduce(|(acc_qty, acc_souls), (qty, souls)| {
				(acc_qty.saturating_add(qty), acc_souls.saturating_add(souls))
			})
			.unwrap_or_default();

		let leader_quantity = leader.get_quantity();
		new_quantity = new_quantity.saturating_add(leader_quantity as u32);
		new_souls = new_souls.saturating_add(leader.get_souls());

		let max_quantity = 16;
		if new_quantity > max_quantity {
			let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
			let dust_avatar = vec![ForgeOutput::Minted(
				AvatarBuilder::with_dna(season_id, dna).into_dust(new_souls).build(),
			)];

			return Ok((
				LeaderForgeOutput::Consumed(leader_id),
				input_sacrifices
					.into_iter()
					.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id))
					.chain(dust_avatar)
					.collect::<Vec<_>>(),
			))
		}

		leader.set_quantity(new_quantity as u8);
		leader.set_souls(new_souls);

		let updated_spec_bytes = {
			let mut leader_spec_bytes = leader.get_specs();

			for spec_bytes in input_sacrifices.iter().map(|(_, sacrifice)| sacrifice.get_specs()) {
				for (i, spec_byte) in spec_bytes.into_iter().enumerate() {
					leader_spec_bytes[i] = leader_spec_bytes[i].saturating_add(spec_byte);
				}
			}

			let pet_type_index = (DnaUtils::current_period::<T>(
				PET_MOON_PHASE_SIZE,
				PET_MOON_PHASE_AMOUNT,
				block_number,
			) % 7) as usize + 9;
			leader_spec_bytes[pet_type_index] = leader_spec_bytes[pet_type_index].saturating_add(1);

			leader.set_specs(leader_spec_bytes);

			leader_spec_bytes
		};

		if updated_spec_bytes.iter().take(8).skip(1).all(|spec_byte| *spec_byte > 0) {
			let slot_types = DnaUtils::indexes_of_max(&updated_spec_bytes[0..8]);
			let final_slot_type = slot_types[hash_provider.hash[0] as usize % slot_types.len()];

			let pet_types = DnaUtils::indexes_of_max(&updated_spec_bytes[8..16]);
			let final_pet_type = pet_types[hash_provider.hash[1] as usize % pet_types.len()];

			leader.set_rarity(RarityTier::Rare);
			leader.set_class_type_1(final_slot_type as u8);
			leader.set_class_type_2(final_pet_type as u8);

			let base_seed = final_slot_type + final_pet_type;

			let base_0 = DnaUtils::create_pattern::<NibbleType>(
				base_seed,
				EquippableItemType::ArmorBase.as_byte() as usize,
			);
			let comp_1 = DnaUtils::create_pattern::<NibbleType>(
				base_seed,
				EquippableItemType::ArmorComponent1.as_byte() as usize,
			);
			let comp_2 = DnaUtils::create_pattern::<NibbleType>(
				base_seed,
				EquippableItemType::ArmorComponent2.as_byte() as usize,
			);
			let comp_3 = DnaUtils::create_pattern::<NibbleType>(
				base_seed,
				EquippableItemType::ArmorComponent3.as_byte() as usize,
			);

			leader.set_spec(SpecIdx::Byte1, DnaUtils::enums_to_bits(&base_0) as u8);
			leader.set_spec(SpecIdx::Byte2, DnaUtils::enums_order_to_bits(&base_0) as u8);
			leader.set_spec(SpecIdx::Byte3, DnaUtils::enums_to_bits(&comp_1) as u8);
			leader.set_spec(SpecIdx::Byte4, DnaUtils::enums_order_to_bits(&comp_1) as u8);
			leader.set_spec(SpecIdx::Byte5, DnaUtils::enums_to_bits(&comp_2) as u8);
			leader.set_spec(SpecIdx::Byte6, DnaUtils::enums_order_to_bits(&comp_2) as u8);
			leader.set_spec(SpecIdx::Byte7, DnaUtils::enums_to_bits(&comp_3) as u8);
			leader.set_spec(SpecIdx::Byte8, DnaUtils::enums_order_to_bits(&comp_3) as u8);
		}

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
	fn test_forge_statue() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let lead_statue = create_random_generic_part(
				&ALICE,
				&[SlotType::Breast, SlotType::ArmFront, SlotType::LegFront],
				3,
			);
			let statue_2 = create_random_generic_part(
				&ALICE,
				&[SlotType::Breast, SlotType::Head, SlotType::LegBack],
				3,
			);

			let expected_spec_bytes = [
				0x00, 0x00, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00,
			];
			assert_eq!(lead_statue.1.get_specs(), expected_spec_bytes);

			let prev_souls = lead_statue.1.get_souls() + statue_2.1.get_souls();

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::statue_avatars(
				lead_statue,
				vec![statue_2],
				season_id,
				&mut hash_provider,
				1,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);

			if let LeaderForgeOutput::Forged((leader_id, leader_avatar), _) = leader_output {
				let wrapped = WrappedAvatar::new(leader_avatar);
				assert_eq!(wrapped.get_souls(), prev_souls);
				assert_eq!(wrapped.get_quantity(), 6);

				assert_eq!(wrapped.get_class_type_1::<HexType>(), HexType::X0);
				assert_eq!(wrapped.get_class_type_2::<HexType>(), HexType::X0);

				let expected_dna = [
					0x12, 0x00, 0x12, 0x06, 0x00, 0x00, 0x01, 0x02, 0x01, 0x00, 0x01, 0x01, 0x00,
					0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				];
				assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);

				let (statue_3_id, statue_3) = create_random_generic_part(
					&ALICE,
					&[SlotType::ArmBack, SlotType::Tail, SlotType::Breast],
					4,
				);

				let prev_souls = wrapped.get_souls() + statue_3.get_souls();

				assert_eq!(
					ForgerV2::<Test>::determine_forge_type(&wrapped, &[&statue_3]),
					ForgeType::Statue
				);

				let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::statue_avatars(
					(leader_id, wrapped),
					vec![(statue_3_id, statue_3)],
					season_id,
					&mut hash_provider,
					1,
				)
				.expect("Should succeed in forging");

				assert_eq!(sacrifice_output.len(), 1);
				assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);

				if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
					let wrapped = WrappedAvatar::new(leader_avatar);
					assert_eq!(wrapped.get_souls(), prev_souls);
					assert_eq!(wrapped.get_quantity(), 10);

					assert_eq!(wrapped.get_class_type_1::<SlotType>(), SlotType::Breast);
					assert_eq!(wrapped.get_class_type_2::<PetType>(), PetType::TankyBullwog);

					let expected_dna = [
						0x12, 0x21, 0x13, 0x0A, 0x00, 0x6C, 0x78, 0xD8, 0x72, 0x55, 0x78, 0x66,
						0x6C, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
						0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					];
					assert_eq!(wrapped.get_dna().as_slice(), &expected_dna);
				} else {
					panic!("LeaderForgeOutput should have been Forged!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_forge_statue_overflow() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let (lead_id, lead_statue) = create_random_generic_part(
				&ALICE,
				&[SlotType::Breast, SlotType::ArmFront, SlotType::LegFront],
				14,
			);
			let (statue_2_id, statue_2) = create_random_generic_part(
				&ALICE,
				&[SlotType::Breast, SlotType::Head, SlotType::LegBack],
				3,
			);

			let expected_spec_bytes = [
				0x00, 0x00, 0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00,
			];
			assert_eq!(lead_statue.get_specs(), expected_spec_bytes);

			let prev_souls = lead_statue.get_souls() + statue_2.get_souls();

			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&lead_statue, &[&statue_2]),
				ForgeType::Statue
			);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::statue_avatars(
				(lead_id, lead_statue),
				vec![(statue_2_id, statue_2)],
				season_id,
				&mut hash_provider,
				1,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			assert!(is_leader_consumed(&leader_output));

			if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
				assert_eq!(avatar.souls, prev_souls);
				assert_eq!(DnaUtils::read_attribute_raw(avatar, AvatarAttr::Quantity), 17);

				assert_eq!(
					DnaUtils::read_attribute::<ItemType>(avatar, AvatarAttr::ItemType),
					ItemType::Special
				);
				assert_eq!(
					DnaUtils::read_attribute::<SpecialItemType>(avatar, AvatarAttr::ItemSubType),
					SpecialItemType::Dust
				);
			}
		});
	}
}
