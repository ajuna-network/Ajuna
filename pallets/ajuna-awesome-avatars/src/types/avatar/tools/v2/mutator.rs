use super::*;
use crate::types::Avatar;

pub(crate) trait AvatarMutator<T: Config> {
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar;
}

impl<T> AvatarMutator<T> for PetItemType
where
	T: Config,
{
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar {
		match self {
			PetItemType::Pet => {
				let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);
				let pet_variation = 2_u8.pow(pet_type.as_byte() as u32);

				let spec_bytes = [0; 16];

				let progress_array = AvatarUtils::generate_progress_bytes(
					RarityType::Legendary,
					SCALING_FACTOR_PERC,
					SPARK_PROGRESS_PROB_PERC,
					[0; 11],
				);

				let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;

				AvatarBuilder::with_base_avatar(base_avatar).into_pet(
					pet_type,
					pet_variation,
					spec_bytes,
					Some(progress_array),
					soul_count,
				)
			},
			PetItemType::PetPart => {
				let quantity = (hash_provider.get_hash_byte() % MAX_QUANTITY) + 1;
				let slot_type = SlotRoller::<T>::roll_on(&ARMOR_SLOT_PROBABILITIES, hash_provider);
				let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);

				AvatarBuilder::with_base_avatar(base_avatar)
					.into_pet_part(pet_type, slot_type, quantity)
			},
			PetItemType::Egg => {
				let pet_variation = (hash_provider.get_hash_byte() % 15) + 1;
				let soul_points = (hash_provider.get_hash_byte() % 99) + 1;

				AvatarBuilder::with_base_avatar(base_avatar).into_egg(
					RarityType::Epic,
					pet_variation,
					soul_points as SoulCount,
					None,
				)
			},
		}
		.build()
	}
}

impl<T> AvatarMutator<T> for MaterialItemType
where
	T: Config,
{
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar {
		AvatarBuilder::with_base_avatar(base_avatar)
			.into_material(*self, (hash_provider.get_hash_byte() % MAX_QUANTITY) + 1)
			.build()
	}
}

impl<T> AvatarMutator<T> for EssenceItemType
where
	T: Config,
{
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar {
		let souls = (hash_provider.get_hash_byte() % 99) + 1;

		match *self {
			EssenceItemType::Glimmer =>
				AvatarBuilder::with_base_avatar(base_avatar).into_glimmer(souls),
			EssenceItemType::ColorSpark | EssenceItemType::PaintFlask => {
				let hash_byte = hash_provider.get_hash_byte();
				let color_pair = (
					ColorType::from_byte(AvatarUtils::high_nibble_of(hash_byte)),
					ColorType::from_byte(AvatarUtils::low_nibble_of(hash_byte)),
				);

				if *self == EssenceItemType::ColorSpark {
					AvatarBuilder::with_base_avatar(base_avatar).into_color_spark(
						color_pair,
						souls as SoulCount,
						None,
					)
				} else {
					AvatarBuilder::with_base_avatar(base_avatar).into_paint_flask(
						color_pair,
						souls as SoulCount,
						None,
					)
				}
			},
			EssenceItemType::GlowSpark | EssenceItemType::ForceGlow => {
				let force_type = ForceType::from_byte(
					hash_provider.get_hash_byte() % ForceType::range().end as u8,
				);

				if *self == EssenceItemType::GlowSpark {
					AvatarBuilder::with_base_avatar(base_avatar).into_glow_spark(
						force_type,
						souls as SoulCount,
						None,
					)
				} else {
					AvatarBuilder::with_base_avatar(base_avatar).into_force_glow(
						force_type,
						souls as SoulCount,
						None,
					)
				}
			},
		}
		.build()
	}
}

impl<T> AvatarMutator<T> for EquipableItemType
where
	T: Config,
{
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar {
		let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;
		let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);

		match *self {
			EquipableItemType::ArmorBase |
			EquipableItemType::ArmorComponent1 |
			EquipableItemType::ArmorComponent2 |
			EquipableItemType::ArmorComponent3 => {
				let slot_type = SlotRoller::<T>::roll_on(&ARMOR_SLOT_PROBABILITIES, hash_provider);

				let rarity_type = {
					if (hash_provider.get_hash_byte() % 3) > 1 {
						RarityType::Rare
					} else {
						RarityType::Epic
					}
				};

				AvatarBuilder::with_base_avatar(base_avatar)
					.try_into_armor_and_component(
						pet_type,
						slot_type,
						vec![*self],
						rarity_type,
						(ColorType::None, ColorType::None),
						ForceType::None,
						soul_count,
					)
					.unwrap()
			},
			EquipableItemType::WeaponVersion1 |
			EquipableItemType::WeaponVersion2 |
			EquipableItemType::WeaponVersion3 => {
				let slot_type = SlotRoller::<T>::roll_on(&WEAPON_SLOT_PROBABILITIES, hash_provider);

				let hash_byte = hash_provider.get_hash_byte();
				let color_pair = (
					ColorType::from_byte(AvatarUtils::high_nibble_of(hash_byte)),
					ColorType::from_byte(AvatarUtils::low_nibble_of(hash_byte)),
				);
				let force_type = ForceType::from_byte(
					hash_provider.get_hash_byte() % ForceType::range().end as u8,
				);

				AvatarBuilder::with_base_avatar(base_avatar)
					.try_into_weapon(pet_type, slot_type, *self, color_pair, force_type, soul_count)
					.unwrap()
			},
		}
		.build()
	}
}

impl<T> AvatarMutator<T> for BlueprintItemType
where
	T: Config,
{
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar {
		let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;

		let pet_type = SlotRoller::<T>::roll_on(&PET_TYPE_PROBABILITIES, hash_provider);
		let slot_type = SlotRoller::<T>::roll_on(&ARMOR_SLOT_PROBABILITIES, hash_provider);
		let equipable_item_type =
			SlotRoller::<T>::roll_on(&EQUIPMENT_TYPE_PROBABILITIES, hash_provider);

		let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
		let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
			base_seed,
			equipable_item_type.as_byte() as usize,
		);

		AvatarBuilder::with_base_avatar(base_avatar)
			.into_blueprint(*self, pet_type, slot_type, equipable_item_type, pattern, soul_count)
			.build()
	}
}

impl<T> AvatarMutator<T> for SpecialItemType
where
	T: Config,
{
	fn mutate_from_base(
		&self,
		base_avatar: Avatar,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Avatar {
		match self {
			SpecialItemType::Dust => AvatarBuilder::with_base_avatar(base_avatar).into_dust(1),
			SpecialItemType::Unidentified => {
				let soul_count = (hash_provider.get_hash_byte() as SoulCount % 25) + 1;
				let hash_byte = hash_provider.get_hash_byte();
				let color_pair = (
					ColorType::from_byte(AvatarUtils::high_nibble_of(hash_byte)),
					ColorType::from_byte(AvatarUtils::low_nibble_of(hash_byte)),
				);
				let force_type = ForceType::from_byte(
					hash_provider.get_hash_byte() % ForceType::range().end as u8,
				);

				AvatarBuilder::with_base_avatar(base_avatar)
					.into_unidentified(color_pair, force_type, soul_count)
			},
		}
		.build()
	}
}
