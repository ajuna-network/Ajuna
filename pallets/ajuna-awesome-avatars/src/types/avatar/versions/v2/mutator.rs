use super::*;
use crate::types::Avatar;

pub(crate) trait AvatarMutator<T: Config> {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()>;
}

impl<T: Config> AvatarMutator<T> for PetItemType {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()> {
		let avatar = match self {
			PetItemType::Pet => {
				let pet_type = PetType::from_byte((hash_provider.get_hash_byte() % 4) + 1);
				let pet_variation = 2_u8.pow(pet_type.as_byte() as u32);

				let spec_bytes = [0; 16];

				let progress_array = AvatarUtils::generate_progress_bytes(
					&RarityTier::Legendary,
					SCALING_FACTOR_PERC,
					SPARK_PROGRESS_PROB_PERC,
					hash_provider,
				);

				let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;

				AvatarBuilder::with_base_avatar(base_avatar).into_pet(
					&pet_type,
					pet_variation,
					spec_bytes,
					progress_array,
					soul_count,
				)
			},
			PetItemType::PetPart => {
				let quantity = (hash_provider.get_hash_byte() % MAX_QUANTITY) + 1;
				let slot_type = SlotRoller::<T>::roll_on(&ARMOR_SLOT_PROBABILITIES, hash_provider);
				let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);

				AvatarBuilder::with_base_avatar(base_avatar)
					.into_pet_part(&pet_type, &slot_type, quantity)
			},
			PetItemType::Egg => {
				let pet_variation = (hash_provider.get_hash_byte() % 15) + 1;
				let soul_points = (hash_provider.get_hash_byte() % 99) + 1;

				let egg_rarity = RarityTier::Rare;

				let progress_array = AvatarUtils::generate_progress_bytes(
					&egg_rarity,
					SCALING_FACTOR_PERC,
					SPARK_PROGRESS_PROB_PERC,
					hash_provider,
				);

				AvatarBuilder::with_base_avatar(base_avatar).into_egg(
					&egg_rarity,
					pet_variation,
					soul_points as SoulCount,
					progress_array,
				)
			},
		}
		.build();

		Ok(avatar)
	}
}

impl<T: Config> AvatarMutator<T> for MaterialItemType {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()> {
		let avatar = AvatarBuilder::with_base_avatar(base_avatar)
			.into_material(self, (hash_provider.get_hash_byte() % MAX_QUANTITY) + 1)
			.build();

		Ok(avatar)
	}
}

impl<T: Config> AvatarMutator<T> for EssenceItemType {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()> {
		let souls = (hash_provider.get_hash_byte() % 99) + 1;

		let avatar = match *self {
			EssenceItemType::Glimmer =>
				AvatarBuilder::with_base_avatar(base_avatar).into_glimmer(1),
			EssenceItemType::ColorSpark | EssenceItemType::PaintFlask => {
				let hash_byte = hash_provider.get_hash_byte();
				let color_pair = (
					ColorType::from_byte(AvatarUtils::high_nibble_of(hash_byte)),
					ColorType::from_byte(AvatarUtils::low_nibble_of(hash_byte)),
				);

				if *self == EssenceItemType::ColorSpark {
					let progress_array = AvatarUtils::generate_progress_bytes(
						&RarityTier::Rare,
						SCALING_FACTOR_PERC,
						SPARK_PROGRESS_PROB_PERC,
						hash_provider,
					);

					AvatarBuilder::with_base_avatar(base_avatar).into_color_spark(
						&color_pair,
						souls as SoulCount,
						progress_array,
					)
				} else {
					let progress_array = AvatarUtils::generate_progress_bytes(
						&RarityTier::Epic,
						SCALING_FACTOR_PERC,
						SPARK_PROGRESS_PROB_PERC,
						hash_provider,
					);

					AvatarBuilder::with_base_avatar(base_avatar).into_paint_flask(
						&color_pair,
						souls as SoulCount,
						progress_array,
					)
				}
			},
			EssenceItemType::GlowSpark | EssenceItemType::GlowFlask => {
				let force =
					Force::from_byte(hash_provider.get_hash_byte() % Force::range().end as u8);

				if *self == EssenceItemType::GlowSpark {
					let progress_array = AvatarUtils::generate_progress_bytes(
						&RarityTier::Rare,
						SCALING_FACTOR_PERC,
						SPARK_PROGRESS_PROB_PERC,
						hash_provider,
					);

					AvatarBuilder::with_base_avatar(base_avatar).into_glow_spark(
						&force,
						souls as SoulCount,
						progress_array,
					)
				} else {
					let progress_array = AvatarUtils::generate_progress_bytes(
						&RarityTier::Epic,
						SCALING_FACTOR_PERC,
						SPARK_PROGRESS_PROB_PERC,
						hash_provider,
					);

					AvatarBuilder::with_base_avatar(base_avatar).into_glow_flask(
						&force,
						souls as SoulCount,
						progress_array,
					)
				}
			},
		}
		.build();

		Ok(avatar)
	}
}

impl<T: Config> AvatarMutator<T> for EquippableItemType {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()> {
		let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;
		let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);

		let avatar = match *self {
			EquippableItemType::ArmorBase |
			EquippableItemType::ArmorComponent1 |
			EquippableItemType::ArmorComponent2 |
			EquippableItemType::ArmorComponent3 => {
				let slot_type = SlotRoller::<T>::roll_on(&ARMOR_SLOT_PROBABILITIES, hash_provider);

				let rarity = {
					if (hash_provider.get_hash_byte() % 3) > 1 {
						RarityTier::Rare
					} else {
						RarityTier::Epic
					}
				};

				AvatarBuilder::with_base_avatar(base_avatar).try_into_armor_and_component(
					&pet_type,
					&slot_type,
					&[*self],
					&rarity,
					&(ColorType::None, ColorType::None),
					&Force::None,
					soul_count,
					hash_provider,
				)
			},
			EquippableItemType::WeaponVersion1 |
			EquippableItemType::WeaponVersion2 |
			EquippableItemType::WeaponVersion3 => {
				let slot_type = SlotRoller::<T>::roll_on(&WEAPON_SLOT_PROBABILITIES, hash_provider);

				let hash_byte = hash_provider.get_hash_byte();
				let color_pair = (
					ColorType::from_byte(AvatarUtils::high_nibble_of(hash_byte)),
					ColorType::from_byte(AvatarUtils::low_nibble_of(hash_byte)),
				);
				let force = Force::from_byte(
					hash_provider.get_hash_byte() % variant_count::<Force>() as u8,
				);

				AvatarBuilder::with_base_avatar(base_avatar).try_into_weapon(
					&pet_type,
					&slot_type,
					self,
					&color_pair,
					&force,
					soul_count,
					hash_provider,
				)
			},
		}?
		.build();

		Ok(avatar)
	}
}

impl<T: Config> AvatarMutator<T> for BlueprintItemType {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()> {
		let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;

		let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);
		let slot_type = SlotRoller::<T>::roll_on(&ARMOR_SLOT_PROBABILITIES, hash_provider);
		let equippable_item_type =
			SlotRoller::<T>::roll_on(&EQUIPMENT_TYPE_PROBABILITIES, hash_provider);

		let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
		let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
			base_seed,
			equippable_item_type.as_byte() as usize,
		);

		let avatar = AvatarBuilder::with_base_avatar(base_avatar)
			.into_blueprint(
				self,
				&pet_type,
				&slot_type,
				&equippable_item_type,
				&pattern,
				soul_count,
			)
			.build();

		Ok(avatar)
	}
}

impl<T: Config> AvatarMutator<T> for SpecialItemType {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Avatar, ()> {
		let avatar = match self {
			SpecialItemType::Dust => AvatarBuilder::with_base_avatar(base_avatar).into_dust(1),
			SpecialItemType::Unidentified => {
				let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;
				let hash_byte = hash_provider.get_hash_byte();
				let color_pair = (
					ColorType::from_byte(AvatarUtils::high_nibble_of(hash_byte)),
					ColorType::from_byte(AvatarUtils::low_nibble_of(hash_byte)),
				);
				let force = Force::from_byte(
					hash_provider.get_hash_byte() % variant_count::<Force>() as u8,
				);

				AvatarBuilder::with_base_avatar(base_avatar)
					.into_unidentified(color_pair, force, soul_count)
			},
			SpecialItemType::Fragment => AvatarBuilder::with_base_avatar(base_avatar).into_dust(1),
			SpecialItemType::ToolBox => {
				let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;

				AvatarBuilder::with_base_avatar(base_avatar).into_toolbox(soul_count)
			},
		}
		.build();

		Ok(avatar)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_mutate_pet() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = PetItemType::Pet
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Pet);

			let item_sub_type = AvatarUtils::read_attribute_as::<PetItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, PetItemType::Pet);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType2);
			assert!({ class_type_2 > 0 && class_type_2 < 5 });

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, 2_u8.pow(class_type_2 as u32));

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Legendary);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);
		});
	}

	#[test]
	fn test_mutate_pet_part() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = PetItemType::PetPart
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Pet);

			let item_sub_type = AvatarUtils::read_attribute_as::<PetItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, PetItemType::PetPart);

			let class_type_1 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType1);
			assert!(class_type_1 > 0 && class_type_1 < 10);
			let class_type_2 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType2);
			assert!(class_type_2 > 0 && class_type_2 < 8);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X1);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Uncommon);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert!(quantity > 0 && quantity < 9);
		});
	}

	#[test]
	fn test_mutate_egg() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = PetItemType::Egg
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Pet);

			let item_sub_type = AvatarUtils::read_attribute_as::<PetItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, PetItemType::Egg);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute(&avatar, &AvatarAttributes::CustomType2);
			assert!(custom_type_2 > 0 && custom_type_2 < 16);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Rare);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);
		});
	}

	#[test]
	fn test_mutate_material() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = MaterialItemType::Polymers
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Material);

			let item_sub_type = AvatarUtils::read_attribute_as::<MaterialItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, MaterialItemType::Polymers);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X1);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Common);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert!(quantity > 0 && quantity < 9);
		});
	}

	#[test]
	fn test_mutate_color_spark() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = EssenceItemType::ColorSpark
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Essence);

			let item_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, EssenceItemType::ColorSpark);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Rare);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 < 5);
			let spec_byte_2 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte2);
			assert!(spec_byte_2 < 5);
		});
	}

	#[test]
	fn test_mutate_paint_flask() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = EssenceItemType::PaintFlask
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Essence);

			let item_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, EssenceItemType::PaintFlask);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Epic);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 < 0b1111_1000);
			let spec_byte_2 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte2);
			assert_eq!(spec_byte_2, 0b0000_1000);
		});
	}

	#[test]
	fn test_mutate_glow_spark() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = EssenceItemType::GlowSpark
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Essence);

			let item_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, EssenceItemType::GlowSpark);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Rare);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 < 7);
			let spec_byte_2 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte2);
			assert_eq!(spec_byte_2, 0);
		});
	}

	#[test]
	fn test_mutate_glow_flask() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = EssenceItemType::GlowFlask
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Essence);

			let item_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, EssenceItemType::GlowFlask);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Epic);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 < 7);
			let spec_byte_2 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte2);
			assert_eq!(spec_byte_2, 0);
		});
	}

	#[test]
	fn test_mutate_armor() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = EquippableItemType::ArmorBase
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Equippable);

			let item_sub_type = AvatarUtils::read_attribute_as::<EquippableItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, EquippableItemType::ArmorBase);

			let class_type_1 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType1);
			assert!(class_type_1 > 0 && class_type_1 < 10);
			let class_type_2 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType2);
			assert!(class_type_2 > 0 && class_type_2 < 8);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert!(rarity_tier == RarityTier::Rare || rarity_tier == RarityTier::Epic);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 > 0);
		});
	}

	#[test]
	fn test_mutate_weapon() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = EquippableItemType::WeaponVersion3
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Equippable);

			let item_sub_type = AvatarUtils::read_attribute_as::<EquippableItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, EquippableItemType::WeaponVersion3);

			let class_type_1 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType1);
			assert!(class_type_1 > 0 && class_type_1 < 10);
			let class_type_2 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType2);
			assert!(class_type_2 > 0 && class_type_2 < 8);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Legendary);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 > 0);
		});
	}

	#[test]
	fn test_mutate_blueprint() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = BlueprintItemType::Blueprint
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Blueprint);

			let item_sub_type = AvatarUtils::read_attribute_as::<BlueprintItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, BlueprintItemType::Blueprint);

			let class_type_1 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType1);
			assert!(class_type_1 > 0 && class_type_1 < 10);
			let class_type_2 = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::ClassType2);
			assert!(class_type_2 > 0 && class_type_2 < 8);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X1);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Rare);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert!(quantity > 0 && quantity < 26);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 > 0);
		});
	}

	#[test]
	fn test_mutate_dust() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = SpecialItemType::Dust
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Special);

			let item_sub_type = AvatarUtils::read_attribute_as::<SpecialItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, SpecialItemType::Dust);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X1);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Common);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);
		});
	}

	#[test]
	fn test_mutate_unidentified() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = SpecialItemType::Unidentified
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Special);

			let item_sub_type = AvatarUtils::read_attribute_as::<SpecialItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, SpecialItemType::Unidentified);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Legendary);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);

			let spec_byte_1 = AvatarUtils::read_spec_byte(&avatar, &AvatarSpecBytes::SpecByte1);
			assert!(spec_byte_1 > 0);
		});
	}

	#[test]
	fn test_mutate_toolbox() {
		ExtBuilder::default().build().execute_with(|| {
			let (_, avatar) = create_random_avatar::<Test, _>(&ALICE, None, Some(|avatar| avatar));
			let mut hash_provider =
				HashProvider::<Test, 32>::new(&Pallet::<Test>::random_hash(b"test_mutate", &ALICE));

			let avatar = SpecialItemType::ToolBox
				.mutate_from_base(avatar, &mut hash_provider)
				.expect("Should mutate avatar");

			let item_type =
				AvatarUtils::read_attribute_as::<ItemType>(&avatar, &AvatarAttributes::ItemType);
			assert_eq!(item_type, ItemType::Special);

			let item_sub_type = AvatarUtils::read_attribute_as::<SpecialItemType>(
				&avatar,
				&AvatarAttributes::ItemSubType,
			);
			assert_eq!(item_sub_type, SpecialItemType::ToolBox);

			let class_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType1);
			assert_eq!(class_type_1, HexType::X0);
			let class_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::ClassType2);
			assert_eq!(class_type_2, HexType::X0);

			let custom_type_1 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType1);
			assert_eq!(custom_type_1, HexType::X0);
			let custom_type_2 =
				AvatarUtils::read_attribute_as::<HexType>(&avatar, &AvatarAttributes::CustomType2);
			assert_eq!(custom_type_2, HexType::X0);

			let rarity_tier = AvatarUtils::read_attribute_as::<RarityTier>(
				&avatar,
				&AvatarAttributes::RarityTier,
			);
			assert_eq!(rarity_tier, RarityTier::Epic);

			let quantity = AvatarUtils::read_attribute(&avatar, &AvatarAttributes::Quantity);
			assert_eq!(quantity, 1);
		});
	}
}
