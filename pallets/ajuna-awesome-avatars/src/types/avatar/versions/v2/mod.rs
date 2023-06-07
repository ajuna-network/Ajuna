mod avatar_utils;
mod combinator;
mod constants;
mod mutator;
mod slot_roller;
#[cfg(test)]
mod test_utils;
mod types;

pub(self) use avatar_utils::*;
pub(self) use combinator::*;
pub(self) use constants::*;
pub(self) use mutator::*;
pub(self) use slot_roller::*;
#[cfg(test)]
pub(self) use test_utils::*;
pub(self) use types::*;

use super::*;
use crate::{
	pallet::SeasonOf,
	types::{MintOption, SeasonId},
	Config,
};
use sp_runtime::DispatchError;
use sp_std::{mem::variant_count, prelude::*};

pub(crate) struct AttributeMapperV2;

impl AttributeMapper for AttributeMapperV2 {
	fn rarity(target: &Avatar) -> u8 {
		AvatarUtils::read_attribute(target, &AvatarAttributes::RarityTier).saturating_sub(1)
	}

	fn force(target: &Avatar) -> u8 {
		// TODO: Determine proper mapping
		AvatarUtils::read_spec_byte(target, &AvatarSpecBytes::SpecByte1).saturating_sub(1)
	}
}

pub(crate) struct MinterV2<T: Config>(pub PhantomData<T>);

impl<T: Config> Minter<T> for MinterV2<T> {
	fn mint(
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError> {
		let mut hash_provider =
			HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(b"avatar_minter_v2", player));

		let roll_amount = mint_option.pack_size.as_mint_count() as usize;
		(0..roll_amount)
			.map(|_| {
				let rolled_item_type = SlotRoller::<T>::roll_on_pack_type(
					mint_option.pack_type.clone(),
					&PACK_TYPE_MATERIAL_ITEM_PROBABILITIES,
					&PACK_TYPE_EQUIPMENT_ITEM_PROBABILITIES,
					&PACK_TYPE_SPECIAL_ITEM_PROBABILITIES,
					&mut hash_provider,
				);

				let avatar_id = Pallet::<T>::random_hash(b"avatar_minter_v2", player);

				let base_dna = Self::generate_empty_dna::<32>()?;
				let base_avatar = Avatar {
					season_id: *season_id,
					version: AvatarVersion::V2,
					dna: base_dna,
					souls: SoulCount::zero(),
				};

				let avatar = Self::mutate_from_item_type(
					mint_option.pack_type.clone(),
					rolled_item_type,
					&mut hash_provider,
					base_avatar,
				)?;

				Avatars::<T>::insert(avatar_id, (player, avatar));
				Owners::<T>::try_append(&player, &season_id, avatar_id)
					.map_err(|_| Error::<T>::MaxOwnershipReached)?;

				Ok(avatar_id)
			})
			.collect()
	}
}

impl<T: Config> MinterV2<T> {
	pub(super) fn generate_empty_dna<const N: usize>() -> Result<Dna, DispatchError> {
		Dna::try_from([0_u8; N].to_vec()).map_err(|_| Error::<T>::IncorrectDna.into())
	}

	fn mutate_from_item_type(
		pack_type: PackType,
		item_type: ItemType,
		hash_provider: &mut HashProvider<T, 32>,
		avatar: Avatar,
	) -> Result<Avatar, DispatchError> {
		match item_type {
			ItemType::Pet => SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_PET_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_PET_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_PET_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)
			.mutate_from_base(avatar, hash_provider),
			ItemType::Material => SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_MATERIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_MATERIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_MATERIAL_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)
			.mutate_from_base(avatar, hash_provider),
			ItemType::Essence => SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_ESSENCE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_ESSENCE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_ESSENCE_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)
			.mutate_from_base(avatar, hash_provider),
			ItemType::Equippable => SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_EQUIPABLE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_EQUIPABLE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_EQUIPABLE_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)
			.mutate_from_base(avatar, hash_provider),
			ItemType::Blueprint => SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_BLUEPRINT_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_BLUEPRINT_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_BLUEPRINT_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)
			.mutate_from_base(avatar, hash_provider),
			ItemType::Special => SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_SPECIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_SPECIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_SPECIAL_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)
			.mutate_from_base(avatar, hash_provider),
		}
		.map_err(|_| Error::<T>::IncompatibleMintComponents.into())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ForgeType {
	None,
	Stack,
	Tinker,
	Build,
	Assemble,
	Breed,
	Equip,
	Mate,
	Feed,
	Glimmer,
	Spark,
	#[allow(dead_code)]
	Special,
	Flask,
}

pub(crate) struct ForgerV2<T: Config>(pub PhantomData<T>);

impl<T: Config> Forger<T> for ForgerV2<T> {
	fn forge(
		player: &T::AccountId,
		season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let mut hash_provider =
			HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(b"avatar_forger_v2", player));

		ensure!(
			input_sacrifices.len() >= MIN_SACRIFICE && input_sacrifices.len() <= MAX_SACRIFICE,
			Error::<T>::IncompatibleAvatarVersions
		);

		let sacrifices =
			input_sacrifices.iter().map(|(_, sacrifice)| sacrifice).collect::<Vec<_>>();

		let forge_type = Self::determine_forge_type(&input_leader.1, sacrifices.as_slice());

		AvatarCombinator::<T>::combine_avatars_in(
			forge_type,
			season_id,
			season,
			input_leader,
			input_sacrifices,
			&mut hash_provider,
		)
	}
}

impl<T: Config> ForgerV2<T> {
	fn determine_forge_type(input_leader: &Avatar, input_sacrifices: &[&Avatar]) -> ForgeType {
		let input_leader_item_type =
			AvatarUtils::read_attribute_as::<ItemType>(input_leader, &AvatarAttributes::ItemType);

		match input_leader_item_type {

			/// ItemType Pet, including the sub types Pet, PetPart & Egg
			ItemType::Pet => {

				let leader_sub_type = AvatarUtils::read_attribute_as::<PetItemType>(
					input_leader,
					&AvatarAttributes::ItemSubType,
				);

				match leader_sub_type {
					PetItemType::Pet => {
						if input_sacrifices.iter().all(|sacrifice| {
							let equippable_item = AvatarUtils::read_attribute_as(
								sacrifice,
								&AvatarAttributes::ItemSubType,
							);

								AvatarUtils::has_attribute_set_with_same_values_as(
								input_leader,
								sacrifice,
								&[AvatarAttributes::RarityTier, AvatarAttributes::ClassType2],
							) && AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemType,
								ItemType::Equippable,
							) && (equippable_item == EquippableItemType::ArmorBase ||
								EquippableItemType::is_weapon(equippable_item))
						}) {
							ForgeType::Equip
						} else if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_set_with_same_values_as(
								input_leader,
								sacrifice,
								&[
									AvatarAttributes::ItemType,
									AvatarAttributes::ItemSubType,
									AvatarAttributes::RarityTier,
								],
							)
						}) {
							ForgeType::Mate
						} else if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_with_same_value_as(
								input_leader,
								sacrifice,
								&AvatarAttributes::ItemType,
							) && AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemSubType,
								PetItemType::Egg,
							) && AvatarUtils::read_attribute_as::<RarityTier>(
								sacrifice,
								&AvatarAttributes::RarityTier,
							) < RarityTier::Legendary
						}) {
							ForgeType::Feed
						} else {
							ForgeType::None
						}
					},
					PetItemType::PetPart => {
						if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemSubType,
								PetItemType::PetPart,
							) && AvatarUtils::has_attribute_with_same_value_as(
								sacrifice,
								input_leader,
								&AvatarAttributes::ClassType2,
							)
						}) {
							ForgeType::Stack
						} else if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemType,
								ItemType::Material,
							)
						}) {
							ForgeType::Tinker
						} else {
							ForgeType::None
						}
					},
					PetItemType::Egg => {

						let leader_rarity = AvatarUtils::read_attribute_as::<RarityTier>(
							input_leader,
							&AvatarAttributes::RarityTier,
						);
	
						if leader_rarity <= RarityTier::Epic && input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemType,
								ItemType::Pet,
							) && AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemSubType,
								PetItemType::Egg,
							)
						}) {
							ForgeType::Breed
						} else {
							ForgeType::None
						}
					},
				}
			},
			ItemType::Material => {
				if input_sacrifices.iter().all(|sacrifice| {
					AvatarUtils::has_attribute_with_same_value_as(
						input_leader,
						sacrifice,
						&AvatarAttributes::ItemSubType,
					)
				}) {
					ForgeType::Stack
				} else {
					ForgeType::None
				}
			},
			ItemType::Essence => {
				let leader_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
					input_leader,
					&AvatarAttributes::ItemSubType,
				);

				match leader_sub_type {
					EssenceItemType::Glimmer => {
						if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemType,
								ItemType::Material,
							) && AvatarUtils::read_attribute(sacrifice, &AvatarAttributes::Quantity) >=
								4
						}) && AvatarUtils::read_attribute(
							input_leader,
							&AvatarAttributes::Quantity,
						) as usize >= input_sacrifices.len()
						{
							ForgeType::Glimmer
						} else if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_set_with_same_values_as(
								sacrifice,
								input_leader,
								&[AvatarAttributes::ItemType, AvatarAttributes::ItemSubType],
							)
						}) {
							ForgeType::Stack
						} else {
							ForgeType::None
						}
					},
					EssenceItemType::ColorSpark | EssenceItemType::GlowSpark => {
						if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_set_with_same_values_as(
								input_leader,
								sacrifice,
								&[AvatarAttributes::ItemType, AvatarAttributes::ItemSubType],
							)
						}) {
							ForgeType::Spark
						} else {
							ForgeType::None
						}
					},
					EssenceItemType::PaintFlask | EssenceItemType::GlowFlask => ForgeType::None,
				}
			},
			ItemType::Equippable => {
				let leader_rarity = AvatarUtils::read_attribute_as::<RarityTier>(
					input_leader,
					&AvatarAttributes::RarityTier,
				);

				let leader_equipable_item_type = AvatarUtils::read_attribute_as::<EquippableItemType>(
					input_leader,
					&AvatarAttributes::ItemSubType,
				);

				let any_same_assemble_version = input_sacrifices
					.iter()
					.any(|sacrifice| AvatarUtils::same_assemble_version(input_leader, sacrifice));

				let all_sacrifice_are_armor_or_toolbox = input_sacrifices.iter().all(|sacrifice| {
					let same_assemble_version =
						AvatarUtils::same_assemble_version(input_leader, sacrifice);

					let equipable_sacrifice_item = AvatarUtils::read_attribute_as::<
						EquippableItemType,
					>(input_leader, &AvatarAttributes::ItemSubType);

					let is_toolbox = AvatarUtils::has_attribute_set_with_values(
						sacrifice,
						&[
							(AvatarAttributes::ItemType, ItemType::Special.as_byte()),
							(AvatarAttributes::ItemSubType, SpecialItemType::ToolBox.as_byte()),
						],
					);

					same_assemble_version &&
						(EquippableItemType::is_armor(equipable_sacrifice_item) || is_toolbox)
				});

				if EquippableItemType::is_armor(leader_equipable_item_type) &&
					any_same_assemble_version &&
					all_sacrifice_are_armor_or_toolbox
				{
					ForgeType::Assemble
				} else if leader_rarity == RarityTier::Epic &&
					leader_equipable_item_type == EquippableItemType::ArmorBase
				{
					let has_one_paint_flask_or_glow = input_sacrifices
						.iter()
						.filter(|sacrifice| {
							let is_essence = AvatarUtils::has_attribute_with_value(
								sacrifice,
								&AvatarAttributes::ItemType,
								ItemType::Essence,
							);

							let is_flask_or_glow = {
								let item_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
									sacrifice,
									&AvatarAttributes::ItemSubType,
								);

								item_sub_type == EssenceItemType::PaintFlask ||
									item_sub_type == EssenceItemType::GlowFlask
							};

							is_essence && is_flask_or_glow
						})
						.count() == 1;

					let all_are_glimmer_paint_or_force = input_sacrifices.iter().all(|sacrifice| {
						let is_essence = AvatarUtils::has_attribute_with_value(
							sacrifice,
							&AvatarAttributes::ItemType,
							ItemType::Essence,
						);

						let is_glimmer_flask_or_force = {
							let item_sub_type = AvatarUtils::read_attribute_as::<EssenceItemType>(
								sacrifice,
								&AvatarAttributes::ItemSubType,
							);

							item_sub_type == EssenceItemType::Glimmer ||
								item_sub_type == EssenceItemType::PaintFlask ||
								item_sub_type == EssenceItemType::GlowFlask
						};

						is_essence && is_glimmer_flask_or_force
					});

					if has_one_paint_flask_or_glow && all_are_glimmer_paint_or_force {
						ForgeType::Flask
					} else {
						ForgeType::None
					}
				} else {
					ForgeType::None
				}
			},
			ItemType::Blueprint => {
				if input_sacrifices.iter().all(|sacrifice| {
					AvatarUtils::has_attribute_set_with_same_values_as(
						sacrifice,
						input_leader,
						&[
							AvatarAttributes::ItemType,
							AvatarAttributes::ItemSubType,
							AvatarAttributes::ClassType1,
							AvatarAttributes::ClassType2,
						],
					) && AvatarUtils::has_same_spec_byte_as(
						sacrifice,
						input_leader,
						&AvatarSpecBytes::SpecByte3,
					)
				}) {
					ForgeType::Stack
				} else if input_sacrifices.iter().all(|sacrifice| {
					AvatarUtils::has_attribute_with_value(
						sacrifice,
						&AvatarAttributes::ItemType,
						ItemType::Material,
					)
				}) {
					ForgeType::Build
				} else {
					ForgeType::None
				}
			},
			ItemType::Special => {
				let leader_sub_type = AvatarUtils::read_attribute_as::<SpecialItemType>(
					input_leader,
					&AvatarAttributes::ItemSubType,
				);

				match leader_sub_type {
					SpecialItemType::Dust =>
						if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_set_with_same_values_as(
								sacrifice,
								input_leader,
								&[AvatarAttributes::ItemType, AvatarAttributes::ItemSubType],
							)
						}) {
							ForgeType::Stack
						} else {
							ForgeType::None
						},
					_ => ForgeType::None,
				}
			},
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_can_be_forged() {
		ExtBuilder::default().build().execute_with(|| {
			let season = Season::default();

			let leader = create_random_material(&ALICE, &MaterialItemType::Polymers, 10);
			let sacrifices = [
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
			];

			// Can forge with V2 avatar and correct number of sacrifices
			assert!(ForgerV2::<Test>::forge(
				&ALICE,
				1,
				&season,
				leader.clone(),
				sacrifices[0..4].to_vec()
			)
			.is_ok());

			// Can't forge with more than MAX_SACRIFICE amount
			assert!(ForgerV2::<Test>::forge(
				&ALICE,
				1,
				&season,
				leader.clone(),
				sacrifices.to_vec()
			)
			.is_err());

			// Can't forge with less than MIN_SACRIFICE amount
			assert!(ForgerV2::<Test>::forge(&ALICE, 1, &season, leader, [].to_vec()).is_err());
		});
	}

	#[test]
	fn test_determine_forge_types_assemble() {
		ExtBuilder::default().build().execute_with(|| {
			// Assemble with armor and essence
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);
			let (_, leader) = create_random_armor_component(
				[0xA2; 32],
				&ALICE,
				&PetType::TankyBullwog,
				&SlotType::ArmBack,
				&RarityTier::Uncommon,
				&[EquippableItemType::ArmorComponent2],
				&(ColorType::ColorA, ColorType::None),
				&Force::Thermal,
				2,
				&mut hash_provider,
			);
			let sacrifices = [&create_random_armor_component(
				[0x2A; 32],
				&ALICE,
				&PetType::TankyBullwog,
				&SlotType::ArmBack,
				&RarityTier::Common,
				&[EquippableItemType::ArmorComponent2],
				&(ColorType::None, ColorType::ColorD),
				&Force::Astral,
				2,
				&mut hash_provider,
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Assemble
			);

			// Assemble without armor-parts or essence fails
			let sacrifices_err =
				[&create_random_material(&ALICE, &MaterialItemType::Polymers, 4).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Assemble with incompatible armor component fails
			let sacrifices_err = [&create_random_armor_component(
				[0x2A; 32],
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::ArmBack,
				&RarityTier::Common,
				&[EquippableItemType::ArmorComponent2],
				&(ColorType::None, ColorType::ColorD),
				&Force::Astral,
				2,
				&mut hash_provider,
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_breed() {
		ExtBuilder::default().build().execute_with(|| {
			// Breed
			let (_, leader) =
				create_random_egg(None, &ALICE, &RarityTier::Rare, 0b0001_1110, 10, [0; 11]);
			let sacrifices =
				[&create_random_egg(None, &ALICE, &RarityTier::Common, 0b0001_0010, 10, [2; 11]).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Breed
			);

			// Breed with Legendary egg leader fails
			let (_, leader_err) =
				create_random_egg(None, &ALICE, &RarityTier::Legendary, 0b0101_0010, 10, [9; 11]);
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_err, &sacrifices),
				ForgeType::None
			);

			// Breed with non-eggs fails
			let sacrifices_err = [&create_random_material(&ALICE, &MaterialItemType::Metals, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_build() {
		ExtBuilder::default().build().execute_with(|| {
			let pet_type = PetType::TankyBullwog;
			let slot_type = SlotType::ArmBack;
			let equip_type = EquippableItemType::ArmorComponent2;
			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
			);

			// Build
			let (_, leader) =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 2);
			let sacrifices = [&create_random_material(&ALICE, &MaterialItemType::Ceramics, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Build
			);

			// Build with non-materials fails
			let sacrifices_err =
				[&create_random_egg(None, &ALICE, &RarityTier::Common, 0b0001_0010, 10, [2; 11]).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_equip() {
		ExtBuilder::default().build().execute_with(|| {
			ExtBuilder::default().build().execute_with(|| {
				let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);
				// Equip
				let (_, leader) = create_random_pet(
					&ALICE,
					&PetType::TankyBullwog,
					0b0001_0001,
					[0; 16],
					[0; 11],
					2,
				);
				let sacrifices = [&create_random_armor_component(
					[0x2A; 32],
					&ALICE,
					&PetType::TankyBullwog,
					&SlotType::ArmBack,
					&RarityTier::Legendary,
					&[EquippableItemType::ArmorBase],
					&(ColorType::None, ColorType::ColorD),
					&Force::Astral,
					2,
					&mut hash_provider,
				)
				.1];
				assert_eq!(
					ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
					ForgeType::Equip
				);

				// Equip without armor-parts fails
				let sacrifices_err =
					[&create_random_material(&ALICE, &MaterialItemType::Polymers, 4).1];
				assert_eq!(
					ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
					ForgeType::None
				);

				// Equip with incompatible armor component fails
				let sacrifices_err = [&create_random_armor_component(
					[0x2A; 32],
					&ALICE,
					&PetType::FoxishDude,
					&SlotType::ArmBack,
					&RarityTier::Common,
					&[EquippableItemType::ArmorComponent2],
					&(ColorType::None, ColorType::ColorD),
					&Force::Astral,
					2,
					&mut hash_provider,
				)
				.1];
				assert_eq!(
					ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
					ForgeType::None
				);
			});
		});
	}

	#[test]
	fn test_determine_forge_types_feed() {
		ExtBuilder::default().build().execute_with(|| {
			// Feed
			let (_, leader) =
				create_random_pet(&ALICE, &PetType::TankyBullwog, 0b0001_0001, [0; 16], [0; 11], 2);
			let sacrifices =
				[&create_random_egg(None, &ALICE, &RarityTier::Common, 0b0001_0010, 10, [2; 11]).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Feed
			);

			// Feed with Legendary egg sacrifices fails
			let sacrifices_err = [&create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0001_0010,
				10,
				[2; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Feed with non-eggs fails
			let sacrifices_err = [&create_random_material(&ALICE, &MaterialItemType::Metals, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_flask() {
		ExtBuilder::default().build().execute_with(|| {
			let mut hash_provider = HashProvider::new_with_bytes(HASH_BYTES);
			// Assemble with armor and essence
			let (_, leader) = create_random_armor_component(
				[0xA2; 32],
				&ALICE,
				&PetType::TankyBullwog,
				&SlotType::ArmBack,
				&RarityTier::Epic,
				&[EquippableItemType::ArmorBase],
				&(ColorType::ColorA, ColorType::None),
				&Force::Thermal,
				2,
				&mut hash_provider,
			);
			let sacrifices = [
				&create_random_glimmer(&ALICE, 1).1,
				&create_random_paint_flask(
					&ALICE,
					&(ColorType::ColorC, ColorType::ColorD),
					3,
					[0; 11],
				)
				.1,
			];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Flask
			);

			// Assemble without flask fails
			let sacrifices_err = [&create_random_glimmer(&ALICE, 1).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Assemble with incompatible component fails
			let sacrifices_err = [&create_random_armor_component(
				[0x2A; 32],
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::ArmBack,
				&RarityTier::Common,
				&[EquippableItemType::ArmorComponent2],
				&(ColorType::None, ColorType::ColorD),
				&Force::Astral,
				2,
				&mut hash_provider,
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_glimmer() {
		ExtBuilder::default().build().execute_with(|| {
			// Glimmer
			let (_, leader) = create_random_glimmer(&ALICE, 5);
			let sacrifices = [
				&create_random_material(&ALICE, &MaterialItemType::Ceramics, 4).1,
				&create_random_material(&ALICE, &MaterialItemType::Nanomaterials, 5).1,
			];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Glimmer
			);

			// Glimmer without enough materials fails
			let sacrifices_err = [
				&create_random_material(&ALICE, &MaterialItemType::Polymers, 2).1,
				&create_random_material(&ALICE, &MaterialItemType::Optics, 4).1,
			];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Glimmer without enough glimmer amount fails
			let (_, leader_err) = create_random_glimmer(&ALICE, 1);
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_err, &sacrifices),
				ForgeType::None
			);

			// Glimmer without material sacrifices fails
			let sacrifices_err = [&create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0001_0010,
				10,
				[2; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_err, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_mate() {
		ExtBuilder::default().build().execute_with(|| {
			// Mate
			let (_, leader) =
				create_random_pet(&ALICE, &PetType::TankyBullwog, 0b0001_0001, [0; 16], [0; 11], 2);
			let sacrifices =
				[&create_random_pet(&ALICE, &PetType::CrazyDude, 0b0001_0001, [0; 16], [0; 11], 2)
					.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Mate
			);

			// Mate with non-pet fails
			let sacrifices_err = [&create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0001_0010,
				10,
				[2; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_spark() {
		ExtBuilder::default().build().execute_with(|| {
			// Spark with ColorSpark
			let (_, leader_color) = create_random_color_spark(
				None,
				&ALICE,
				&(ColorType::ColorA, ColorType::ColorC),
				100,
				[0; 11],
			);
			let sacrifices_color = [&create_random_color_spark(
				None,
				&ALICE,
				&(ColorType::ColorC, ColorType::ColorD),
				3,
				[0; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_color, &sacrifices_color),
				ForgeType::Spark
			);

			// Spark with GlowSpark
			let (_, leader_glow) =
				create_random_glow_spark(None, &ALICE, &Force::Kinetic, 100, [0; 11]);
			let sacrifices_glow =
				[&create_random_glow_spark(None, &ALICE, &Force::Thermal, 100, [0; 11]).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_glow, &sacrifices_glow),
				ForgeType::Spark
			);

			// Spark with incompatible spark types fails
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_glow, &sacrifices_color),
				ForgeType::None
			);
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_color, &sacrifices_glow),
				ForgeType::None
			);

			// Spark without GlowSpark or ColorSpark fails
			let sacrifices_err = [&create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0001_0010,
				10,
				[2; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_color, &sacrifices_err),
				ForgeType::None
			);
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader_glow, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_stack() {
		ExtBuilder::default().build().execute_with(|| {
			// Stack Materials
			let (_, leader) = create_random_material(&ALICE, &MaterialItemType::Polymers, 10);
			let sacrifices = [&create_random_material(&ALICE, &MaterialItemType::Polymers, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Stack
			);
			// Stack different Materials fails
			let sacrifices_err = [&create_random_material(&ALICE, &MaterialItemType::Metals, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Stack Dust
			let (_, leader) = create_random_dust(&ALICE, 10);
			let sacrifices = [&create_random_dust(&ALICE, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Stack
			);
			// Stack dust with non-dust fails
			let sacrifices_err = [&create_random_material(&ALICE, &MaterialItemType::Metals, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Stack Glimmer
			let (_, leader) = create_random_glimmer(&ALICE, 1);
			let sacrifices = [&create_random_glimmer(&ALICE, 2).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Stack
			);
			// Stack Glimmer with different non-glimmer fails
			let sacrifices_err = [&create_random_dust(&ALICE, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Stack PetPart
			let (_, leader) =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::Head, 1);
			let sacrifices =
				[&create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::Head, 1).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Stack
			);
			// Stack PetPart with different PetType fails
			let sacrifices_err =
				[&create_random_pet_part(&ALICE, &PetType::BigHybrid, &SlotType::Head, 1).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);

			// Stack Blueprint
			let (_, leader) = create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&[MaterialItemType::Metals],
				10,
			);
			let sacrifices = [&create_random_blueprint(
				&ALICE,
				&PetType::FoxishDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&[MaterialItemType::Metals],
				10,
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Stack
			);
			// Stack different Blueprints fails
			let sacrifices_err = [&create_random_blueprint(
				&ALICE,
				&PetType::CrazyDude,
				&SlotType::Head,
				&EquippableItemType::ArmorBase,
				&[MaterialItemType::Metals],
				10,
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}

	#[test]
	fn test_determine_forge_types_tinker() {
		ExtBuilder::default().build().execute_with(|| {
			// Tinker
			let (_, leader) =
				create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::Head, 1);
			let sacrifices = [&create_random_material(&ALICE, &MaterialItemType::Polymers, 10).1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Tinker
			);

			// Tinker with non-materials fails
			let sacrifices_err = [&create_random_color_spark(
				None,
				&ALICE,
				&(ColorType::ColorA, ColorType::ColorC),
				10,
				[0; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::None
			);
		});
	}
}
