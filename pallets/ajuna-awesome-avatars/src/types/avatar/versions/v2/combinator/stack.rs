use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn stack_avatars(
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let (mut new_quantity, new_souls) = input_sacrifices
			.iter()
			.map(|sacrifice| {
				(
					AvatarUtils::read_attribute(&sacrifice.1, &AvatarAttributes::Quantity) as u32,
					sacrifice.1.souls,
				)
			})
			.reduce(|(acc_qty, acc_souls), (qty, souls)| {
				(acc_qty.saturating_add(qty), acc_souls.saturating_add(souls))
			})
			.unwrap_or_default();

		let leader_quantity = AvatarUtils::read_attribute(&leader, &AvatarAttributes::Quantity);

		new_quantity = new_quantity.saturating_add(leader_quantity as u32);

		let mut dust_quantity = if new_quantity > MAX_BYTE {
			let leader_custom_type_1 =
				AvatarUtils::read_attribute(&leader, &AvatarAttributes::CustomType1);
			let dust_qty = (new_quantity - MAX_BYTE) * leader_custom_type_1 as u32;
			new_quantity = MAX_BYTE;
			dust_qty
		} else {
			0
		};

		let exploit_level = (dust_quantity / MAX_BYTE) % 5;
		let transform_per_cycle = ((exploit_level * exploit_level) + 1) as u8;
		let add_prob_perc = 3 * (transform_per_cycle - 1) as u32;

		AvatarUtils::write_attribute(&mut leader, &AvatarAttributes::Quantity, new_quantity as u8);
		leader.souls = (leader.souls + new_souls).saturating_sub(dust_quantity);

		let mut total_soul_points = 0;

		for i in 0..input_sacrifices.len() {
			if hash_provider.hash[i] as u32 * SCALING_FACTOR_PERC <
				(STACK_PROB_PERC + add_prob_perc) * MAX_BYTE &&
				AvatarUtils::can_use_avatar(&leader, transform_per_cycle)
			{
				let (_, _, out_soul_points) =
					AvatarUtils::use_avatar(&mut leader, transform_per_cycle);
				total_soul_points += out_soul_points;
			}
		}

		let leader_forge_output = if leader.souls > 0 {
			LeaderForgeOutput::Forged((leader_id, leader), 0)
		} else {
			LeaderForgeOutput::Consumed(leader_id)
		};

		let glimmer_avatar = if total_soul_points > 0 {
			let glimmer_quantity = total_soul_points / GLIMMER_SP as SoulCount;
			dust_quantity += total_soul_points % GLIMMER_SP as SoulCount;

			if glimmer_quantity > 0 {
				let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
				Some(
					AvatarBuilder::with_dna(season_id, dna)
						.into_glimmer(glimmer_quantity as u8)
						.build(),
				)
			} else {
				None
			}
		} else {
			None
		};

		let dust_avatars = if dust_quantity > 0 {
			let mut dust_qty = dust_quantity;
			let times = dust_quantity / MAX_BYTE;

			let mut dust_avatars = Vec::with_capacity(times as usize);

			for _ in 0..(times + 1) {
				let soul_points = if dust_qty > MAX_BYTE { MAX_BYTE } else { dust_qty };

				let dna = MinterV2::<T>::generate_empty_dna::<32>()?;
				dust_avatars
					.push(AvatarBuilder::with_dna(season_id, dna).into_dust(soul_points).build());
				dust_qty = dust_qty.saturating_sub(MAX_BYTE);

				if dust_qty == 0 {
					break
				}
			}

			dust_avatars
		} else {
			Vec::with_capacity(0)
		};

		let output_vec: Vec<ForgeOutput<T>> = input_sacrifices
			.into_iter()
			.map(|(sacrifice_id, _)| ForgeOutput::Consumed(sacrifice_id))
			.chain(glimmer_avatar.map(|minted_avatar| ForgeOutput::Minted(minted_avatar)))
			.chain(dust_avatars.into_iter().map(|minted_avatar| ForgeOutput::Minted(minted_avatar)))
			.collect();

		Ok((leader_forge_output, output_vec))
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

			let material_input_1 = create_random_material(&ALICE, &MaterialItemType::Polymers, 1);
			let material_input_2 = create_random_material(&ALICE, &MaterialItemType::Polymers, 2);
			let material_input_3 = create_random_material(&ALICE, &MaterialItemType::Polymers, 5);
			let material_input_4 = create_random_material(&ALICE, &MaterialItemType::Polymers, 3);

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
				assert_eq!(leader_avatar.souls, 16);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					8
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
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::Head, 3);
			let pet_part_input_2 =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::ArmBack, 4);
			let pet_part_input_3 =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::LegBack, 5);
			let pet_part_input_4 =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::LegFront, 5);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				pet_part_input_1,
				vec![pet_part_input_2, pet_part_input_3, pet_part_input_4],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert!(sacrifice_output.iter().all(|output| !is_forged(output)));
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 2);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 14);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					14
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_dust() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let dust_input_1 = create_random_dust(&ALICE, 45);
			let dust_input_2 = create_random_dust(&ALICE, 55);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				dust_input_1,
				vec![dust_input_2],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert!(sacrifice_output.iter().all(|output| !is_forged(output)));
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 100);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					100
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}

			let dust_input_1 = create_random_dust(&ALICE, 25);
			let dust_input_2 = create_random_dust(&ALICE, 30);
			let dust_input_3 = create_random_dust(&ALICE, 45);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				dust_input_1,
				vec![dust_input_2, dust_input_3],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert!(sacrifice_output.iter().all(|output| !is_forged(output)));
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 100);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					100
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_glimmer() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let glimmer_input_1 = create_random_glimmer(&ALICE, 45);
			let glimmer_input_2 = create_random_glimmer(&ALICE, 55);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				glimmer_input_1,
				vec![glimmer_input_2],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 200);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					100
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}

			let glimmer_input_1 = create_random_glimmer(&ALICE, 25);
			let glimmer_input_2 = create_random_glimmer(&ALICE, 30);
			let glimmer_input_3 = create_random_glimmer(&ALICE, 45);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				glimmer_input_1,
				vec![glimmer_input_2, glimmer_input_3],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 0);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 2);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 200);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					100
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_blueprint() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let pattern = [
				MaterialItemType::Polymers,
				MaterialItemType::Electronics,
				MaterialItemType::PowerCells,
				MaterialItemType::Optics,
			];

			let blueprint_input_1 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);
			let blueprint_input_2 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);

			let total_souls = blueprint_input_1.1.souls + blueprint_input_2.1.souls;

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				blueprint_input_1,
				vec![blueprint_input_2],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, total_souls);
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					8
				);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}

			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);

			let blueprint_input_1 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);
			let blueprint_input_2 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);
			let blueprint_input_3 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);
			let blueprint_input_4 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);
			let blueprint_input_5 = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&pattern,
				4,
			);

			let total_souls = blueprint_input_1.1.souls +
				blueprint_input_2.1.souls +
				blueprint_input_3.1.souls +
				blueprint_input_4.1.souls +
				blueprint_input_5.1.souls;

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				blueprint_input_1,
				vec![blueprint_input_2, blueprint_input_3, blueprint_input_4, blueprint_input_5],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 5);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					16
				);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[4] {
					let item_type = AvatarUtils::read_attribute_as::<ItemType>(
						avatar,
						&AvatarAttributes::ItemType,
					);
					assert_eq!(item_type, ItemType::Essence);

					assert_eq!(AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity), 2);

					assert_eq!(avatar.souls + leader_avatar.souls, total_souls);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_overflow_dust() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let dust_input_1 = create_random_material(&ALICE, &MaterialItemType::Ceramics, u8::MAX);
			let dust_input_2 = create_random_material(&ALICE, &MaterialItemType::Ceramics, u8::MAX);

			let total_souls = dust_input_1.1.souls + dust_input_2.1.souls;

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				dust_input_1,
				vec![dust_input_2],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					u8::MAX
				);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					let item_type = AvatarUtils::read_attribute_as::<ItemType>(
						avatar,
						&AvatarAttributes::ItemType,
					);
					assert_eq!(item_type, ItemType::Special);

					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);

					assert_eq!(avatar.souls + leader_avatar.souls, total_souls);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_overflow_pet_part() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let pet_part_input_1 =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::Head, u8::MAX);
			let pet_part_input_2 =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::ArmBack, u8::MAX);

			let total_souls = pet_part_input_1.1.souls + pet_part_input_2.1.souls;

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				pet_part_input_1,
				vec![pet_part_input_2],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					u8::MAX
				);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[1] {
					let item_type = AvatarUtils::read_attribute_as::<ItemType>(
						avatar,
						&AvatarAttributes::ItemType,
					);
					assert_eq!(item_type, ItemType::Special);

					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);

					assert_eq!(avatar.souls + leader_avatar.souls, total_souls);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_overflow_glimmer() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let glimmer_input_1 = create_random_glimmer(&ALICE, u8::MAX);
			let glimmer_input_2 = create_random_glimmer(&ALICE, u8::MAX);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				glimmer_input_1,
				vec![glimmer_input_2],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 3);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					250
				);
				assert_eq!(leader_avatar.souls, 500);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[2] {
					let item_type = AvatarUtils::read_attribute_as::<ItemType>(
						avatar,
						&AvatarAttributes::ItemType,
					);
					assert_eq!(item_type, ItemType::Special);

					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);

					assert_eq!(avatar.souls, MAX_BYTE);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}

	#[test]
	fn test_stack_exploit_glimmer() {
		ExtBuilder::default().build().execute_with(|| {
			let season_id = 0 as SeasonId;
			let mut hash_provider = HashProvider::new_with_bytes([
				0x28, 0xD2, 0x1C, 0xCA, 0xEE, 0x3F, 0x80, 0xD9, 0x83, 0x21, 0x5D, 0xF9, 0xAC, 0x5E,
				0x29, 0x74, 0x6A, 0xD9, 0x6C, 0xB0, 0x20, 0x16, 0xB5, 0xAD, 0xEA, 0x86, 0xFD, 0xE0,
				0xCC, 0xFD, 0x01, 0xB4,
			]);

			let dust_input_1 = create_random_dust(&ALICE, u8::MAX as SoulCount);
			let dust_input_2 = create_random_dust(&ALICE, u8::MAX as SoulCount);
			let dust_input_3 = create_random_dust(&ALICE, u8::MAX as SoulCount);
			let dust_input_4 = create_random_dust(&ALICE, u8::MAX as SoulCount);
			let dust_input_5 = create_random_dust(&ALICE, u8::MAX as SoulCount);

			let (leader_output, sacrifice_output) = AvatarCombinator::<Test>::stack_avatars(
				dust_input_1,
				vec![dust_input_2, dust_input_3, dust_input_4, dust_input_5],
				season_id,
				&mut hash_provider,
			)
			.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 9);
			assert_eq!(sacrifice_output.iter().filter(|output| is_consumed(output)).count(), 4);
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 5);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(
					AvatarUtils::read_attribute(&leader_avatar, &AvatarAttributes::Quantity),
					221
				);

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[4] {
					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						17
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[5] {
					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[6] {
					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[7] {
					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}

				if let ForgeOutput::Minted(avatar) = &sacrifice_output[8] {
					assert_eq!(
						AvatarUtils::read_attribute(avatar, &AvatarAttributes::Quantity),
						u8::MAX
					);
				} else {
					panic!("ForgeOutput should have been Minted!")
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		});
	}
}
