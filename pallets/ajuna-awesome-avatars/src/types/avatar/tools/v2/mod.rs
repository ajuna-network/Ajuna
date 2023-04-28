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
use sp_std::prelude::*;

pub(super) struct AttributeMapperV2;

impl AttributeMapper for AttributeMapperV2 {
	fn min_tier(&self, target: &Avatar) -> u8 {
		AvatarUtils::read_attribute(target, AvatarAttributes::RarityType).saturating_sub(1)
	}

	fn last_variation(&self, target: &Avatar) -> u8 {
		// TODO: Determine proper mapping
		AvatarUtils::read_spec_byte(target, AvatarSpecBytes::SpecByte1).saturating_sub(1)
	}
}

pub(super) struct AvatarMinterV2<'a, T: Config>(pub PhantomData<&'a T>);

impl<'a, T> Minter<T> for AvatarMinterV2<'a, T>
where
	T: Config,
{
	fn mint_avatar_set(
		&self,
		player: &T::AccountId,
		season_id: &SeasonId,
		_season: &SeasonOf<T>,
		mint_option: &MintOption,
	) -> Result<Vec<MintOutput<T>>, DispatchError> {
		self.mint_avatar_set_for(player, season_id, mint_option)
	}
}

impl<'a, T> AvatarMinterV2<'a, T>
where
	T: Config,
{
	pub(super) fn generate_base_avatar_dna(
		&self,
		hash_provider: &mut HashProvider<T, 32>,
		starting_index: usize,
	) -> Result<Dna, DispatchError> {
		let base_hash = hash_provider.full_hash(starting_index);

		Dna::try_from(base_hash.as_ref()[0..32].to_vec())
			.map_err(|_| Error::<T>::IncorrectDna.into())
	}

	fn get_mutator_from_item_type(
		&self,
		pack_type: PackType,
		item_type: ItemType,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Box<dyn AvatarMutator<T>> {
		match item_type {
			ItemType::Pet => Box::new(SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_PET_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_PET_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_PET_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)),
			ItemType::Material => Box::new(SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_MATERIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_MATERIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_MATERIAL_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)),
			ItemType::Essence => Box::new(SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_ESSENCE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_ESSENCE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_ESSENCE_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)),
			ItemType::Equipable => Box::new(SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_EQUIPABLE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_EQUIPABLE_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_EQUIPABLE_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)),
			ItemType::Blueprint => Box::new(SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_BLUEPRINT_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_BLUEPRINT_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_BLUEPRINT_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)),
			ItemType::Special => Box::new(SlotRoller::<T>::roll_on_pack_type(
				pack_type,
				&PACK_TYPE_MATERIAL_SPECIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_SPECIAL_ITEM_TYPE_PROBABILITIES,
				&PACK_TYPE_SPECIAL_SPECIAL_ITEM_TYPE_PROBABILITIES,
				hash_provider,
			)),
		}
	}

	fn mint_avatar_set_for(
		&self,
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<MintOutput<T>>, DispatchError> {
		let mut hash_provider =
			HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(b"avatar_minter_v2", player));

		let roll_amount = mint_option.count as usize;
		let mut minted_avatars = Vec::with_capacity(roll_amount);

		for i in 0..roll_amount {
			let rolled_item_type = SlotRoller::<T>::roll_on_pack_type(
				mint_option.mint_pack,
				&PACK_TYPE_MATERIAL_ITEM_PROBABILITIES,
				&PACK_TYPE_EQUIPMENT_ITEM_PROBABILITIES,
				&PACK_TYPE_SPECIAL_ITEM_PROBABILITIES,
				&mut hash_provider,
			);

			let avatar_id = Pallet::<T>::random_hash(b"avatar_minter_v2", player);

			let dna_mutation = (i * 13) % 31;
			let base_dna = self.generate_base_avatar_dna(&mut hash_provider, dna_mutation)?;
			let base_avatar = Avatar {
				season_id: *season_id,
				version: AvatarVersion::V2,
				dna: base_dna,
				souls: SoulCount::zero(),
			};

			let avatar = self
				.get_mutator_from_item_type(
					mint_option.mint_pack,
					rolled_item_type,
					&mut hash_provider,
				)
				.mutate_from_base(base_avatar, &mut hash_provider);

			minted_avatars.push((avatar_id, avatar));
		}

		Ok(minted_avatars)
	}
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ForgeType {
	None = 0,
	Stack = 1,
	Tinker = 2,
	Build = 3,
	Assemble = 4,
	Breed = 5,
	Equip = 6,
	Mate = 7,
	Feed = 8,
	Glimmer = 9,
	Spark = 10,
	Special = 11,
}

pub(super) struct AvatarForgerV2<'a, T: Config>(pub PhantomData<&'a T>);

impl<'a, T> Forger<T> for AvatarForgerV2<'a, T>
where
	T: Config,
{
	fn forge_with(
		&self,
		player: &T::AccountId,
		season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let mut hash_provider =
			HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(b"avatar_forger_v2", player));

		self.can_be_forged(season, &input_leader, &input_sacrifices)?;

		let sacrifices =
			input_sacrifices.iter().map(|(_, sacrifice)| sacrifice).collect::<Vec<_>>();

		let forge_type = self.determine_forge_type(&input_leader.1, sacrifices.as_slice());

		AvatarCombinator::<T>::combine_avatars_in(
			forge_type,
			player,
			season_id,
			season,
			input_leader,
			input_sacrifices,
			&mut hash_provider,
		)
	}

	fn can_be_forged(
		&self,
		_season: &SeasonOf<T>,
		input_leader: &ForgeItem<T>,
		input_sacrifices: &[ForgeItem<T>],
	) -> Result<(), DispatchError> {
		if (input_leader.1.version != AvatarVersion::V2) ||
			(input_sacrifices.len() < MIN_SACRIFICE || input_sacrifices.len() > MAX_SACRIFICE) ||
			(input_sacrifices.iter().any(|(_, avatar)| avatar.version != AvatarVersion::V2))
		{
			Err(Error::<T>::IncompatibleAvatarVersions.into())
		} else {
			Ok(())
		}
	}
}

impl<'a, T> AvatarForgerV2<'a, T>
where
	T: Config,
{
	fn determine_forge_type(
		&self,
		input_leader: &Avatar,
		input_sacrifices: &[&Avatar],
	) -> ForgeType {
		let input_leader_item_type =
			AvatarUtils::read_attribute_as::<ItemType>(input_leader, AvatarAttributes::ItemType);

		match input_leader_item_type {
			ItemType::Pet => {
				let leader_rarity = AvatarUtils::read_attribute_as::<RarityType>(
					input_leader,
					AvatarAttributes::RarityType,
				);

				let leader_sub_type = AvatarUtils::read_attribute_as::<PetItemType>(
					input_leader,
					AvatarAttributes::ItemSubType,
				);

				match leader_rarity {
					RarityType::Legendary => match leader_sub_type {
						PetItemType::Pet => {
							if input_sacrifices.iter().all(|sacrifice| {
								let equipable_item = AvatarUtils::read_attribute_as(
									sacrifice,
									AvatarAttributes::ItemSubType,
								);

								AvatarUtils::has_attribute_set_with_same_values_as(
									input_leader,
									sacrifice,
									&[AvatarAttributes::RarityType, AvatarAttributes::ClassType2],
								) && AvatarUtils::has_attribute_with_value(
									sacrifice,
									AvatarAttributes::ItemType,
									ItemType::Equipable,
								) && (equipable_item == EquipableItemType::ArmorBase ||
									EquipableItemType::is_weapon(equipable_item))
							}) {
								ForgeType::Equip
							} else if input_sacrifices.iter().all(|sacrifice| {
								AvatarUtils::has_attribute_set_with_same_values_as(
									input_leader,
									sacrifice,
									&[
										AvatarAttributes::ItemType,
										AvatarAttributes::ItemSubType,
										AvatarAttributes::RarityType,
									],
								)
							}) {
								ForgeType::Mate
							} else if input_sacrifices.iter().all(|sacrifice| {
								AvatarUtils::has_attribute_with_same_value_as(
									input_leader,
									sacrifice,
									AvatarAttributes::ItemType,
								) && AvatarUtils::has_attribute_with_value(
									sacrifice,
									AvatarAttributes::ItemSubType,
									PetItemType::Egg,
								) && AvatarUtils::read_attribute_as::<RarityType>(
									sacrifice,
									AvatarAttributes::RarityType,
								) < RarityType::Legendary
							}) {
								ForgeType::Feed
							} else {
								ForgeType::None
							}
						},
						PetItemType::PetPart => ForgeType::None,
						PetItemType::Egg => ForgeType::None,
					},
					RarityType::Mythical => ForgeType::None,
					_ => match leader_sub_type {
						PetItemType::Pet => ForgeType::None,
						PetItemType::PetPart => {
							if input_sacrifices.iter().all(|sacrifice| {
								AvatarUtils::has_attribute_with_value(
									sacrifice,
									AvatarAttributes::ItemSubType,
									PetItemType::PetPart,
								) && AvatarUtils::has_attribute_with_same_value_as(
									sacrifice,
									input_leader,
									AvatarAttributes::ClassType2,
								)
							}) {
								ForgeType::Stack
							} else if input_sacrifices.iter().all(|sacrifice| {
								AvatarUtils::has_attribute_with_value(
									sacrifice,
									AvatarAttributes::ItemType,
									ItemType::Material,
								)
							}) {
								ForgeType::Tinker
							} else {
								ForgeType::None
							}
						},
						PetItemType::Egg => {
							if input_sacrifices.iter().all(|sacrifice| {
								AvatarUtils::has_attribute_with_value(
									sacrifice,
									AvatarAttributes::ItemType,
									ItemType::Pet,
								) && AvatarUtils::has_attribute_with_value(
									sacrifice,
									AvatarAttributes::ItemSubType,
									PetItemType::Egg,
								)
							}) {
								ForgeType::Breed
							} else {
								ForgeType::None
							}
						},
					},
				}
			},
			ItemType::Material => {
				if input_sacrifices.iter().all(|sacrifice| {
					AvatarUtils::has_attribute_with_same_value_as(
						input_leader,
						sacrifice,
						AvatarAttributes::ItemSubType,
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
					AvatarAttributes::ItemSubType,
				);

				match leader_sub_type {
					EssenceItemType::Glimmer => {
						if input_sacrifices.iter().all(|sacrifice| {
							AvatarUtils::has_attribute_with_value(
								sacrifice,
								AvatarAttributes::ItemType,
								ItemType::Material,
							) && AvatarUtils::read_attribute(sacrifice, AvatarAttributes::Quantity) >=
								4
						}) && AvatarUtils::read_attribute(
							input_leader,
							AvatarAttributes::Quantity,
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
					EssenceItemType::PaintFlask | EssenceItemType::ForceGlow => ForgeType::None,
				}
			},
			ItemType::Equipable => {
				let leader_rarity = AvatarUtils::read_attribute_as::<RarityType>(
					input_leader,
					AvatarAttributes::RarityType,
				);

				match leader_rarity {
					RarityType::Legendary | RarityType::Mythical => ForgeType::None,
					_ => {
						let equipable_item = AvatarUtils::read_attribute_as::<EquipableItemType>(
							input_leader,
							AvatarAttributes::ItemSubType,
						);

						let any_sacrifice_full_match_leader =
							input_sacrifices.iter().any(|sacrifice| {
								AvatarUtils::has_attribute_set_with_same_values_as(
									input_leader,
									sacrifice,
									&[
										AvatarAttributes::ItemType,
										AvatarAttributes::ItemSubType,
										AvatarAttributes::ClassType1,
										AvatarAttributes::ClassType2,
									],
								)
							});

						let all_sacrifice_are_armor_part_or_essence =
							input_sacrifices.iter().all(|sacrifice| {
								let equipable_sacrifice_item =
									AvatarUtils::read_attribute_as::<EquipableItemType>(
										input_leader,
										AvatarAttributes::ItemSubType,
									);

								(AvatarUtils::has_attribute_set_with_same_values_as(
									sacrifice,
									input_leader,
									&[
										AvatarAttributes::ItemType,
										AvatarAttributes::ClassType1,
										AvatarAttributes::ClassType2,
									],
								) && EquipableItemType::is_armor(equipable_sacrifice_item)) ||
									AvatarUtils::has_attribute_with_value(
										sacrifice,
										AvatarAttributes::ItemType,
										ItemType::Essence,
									)
							});

						if EquipableItemType::is_armor(equipable_item) &&
							any_sacrifice_full_match_leader &&
							all_sacrifice_are_armor_part_or_essence
						{
							ForgeType::Assemble
						} else {
							ForgeType::None
						}
					},
				}
			},
			ItemType::Blueprint => {
				if input_sacrifices.iter().all(|sacrifice| {
					AvatarUtils::has_attribute_with_value(
						sacrifice,
						AvatarAttributes::ItemType,
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
					AvatarAttributes::ItemSubType,
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
					SpecialItemType::Unidentified => ForgeType::None,
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

			let forger = AvatarForgerV2::<Test>(PhantomData);

			let mut leader = create_random_material(&ALICE, MaterialItemType::Polymers, 10);
			let sacrifices = [
				create_random_material(&ALICE, MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, MaterialItemType::Polymers, 10),
			];

			// Can't forge non V2 avatar
			leader.1.version = AvatarVersion::V1;
			assert!(forger.can_be_forged(&season, &leader, &sacrifices[0..4]).is_err());
			leader.1.version = AvatarVersion::V2;
			// Can forge with V2 avatar and correct number of sacrifices
			assert!(forger.can_be_forged(&season, &leader, &sacrifices[0..4]).is_ok());

			// Can't forge with more than MAX_SACRIFICE amount
			assert!(forger.can_be_forged(&season, &leader, &sacrifices).is_err());

			// Can't forge with less than MIN_SACRIFICE amount
			assert!(forger.can_be_forged(&season, &leader, &[]).is_err());
		});
	}

	#[test]
	fn test_determine_forge_types_assemble() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Assemble with armor and essence
			let (_, leader) = create_random_armor_component(
				[0xA2; 32],
				&ALICE,
				PetType::TankyBullwog,
				SlotType::ArmBack,
				RarityType::Uncommon,
				vec![EquipableItemType::ArmorComponent2],
				(ColorType::ColorA, ColorType::None),
				ForceType::Thermal,
				2,
			);
			let sacrifices = [
				&create_random_armor_component(
					[0x2A; 32],
					&ALICE,
					PetType::TankyBullwog,
					SlotType::ArmBack,
					RarityType::Common,
					vec![EquipableItemType::ArmorComponent2],
					(ColorType::None, ColorType::ColorD),
					ForceType::Astral,
					2,
				)
				.1,
				&create_random_glimmer(&ALICE, 10).1,
			];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Assemble);

			// Assemble without armor-parts or essence fails
			let sacrifices_err = [&create_random_material(&ALICE, MaterialItemType::Polymers, 4).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

			// Assemble with incompatible armor component fails
			let sacrifices_err = [&create_random_armor_component(
				[0x2A; 32],
				&ALICE,
				PetType::FoxishDude,
				SlotType::ArmBack,
				RarityType::Common,
				vec![EquipableItemType::ArmorComponent2],
				(ColorType::None, ColorType::ColorD),
				ForceType::Astral,
				2,
			)
			.1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_breed() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Breed
			let (_, leader) =
				create_random_egg(None, &ALICE, RarityType::Rare, 0b0001_1110, 10, [0; 11]);
			let sacrifices =
				[&create_random_egg(None, &ALICE, RarityType::Common, 0b0001_0010, 10, [2; 11]).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Breed);

			// Breed with Legendary egg leader fails
			let (_, leader_err) =
				create_random_egg(None, &ALICE, RarityType::Legendary, 0b0101_0010, 10, [9; 11]);
			assert_eq!(forger.determine_forge_type(&leader_err, &sacrifices), ForgeType::None);

			// Breed with non-eggs fails
			let sacrifices_err = [&create_random_material(&ALICE, MaterialItemType::Metals, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_build() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			let pet_type = PetType::TankyBullwog;
			let slot_type = SlotType::ArmBack;
			let equip_type = EquipableItemType::ArmorComponent2;
			let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
			let pattern = AvatarUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
			);

			// Build
			let (_, leader) = create_random_blueprint(
				&ALICE,
				pet_type,
				slot_type,
				equip_type,
				pattern.clone(),
				2,
			);
			let sacrifices = [&create_random_material(&ALICE, MaterialItemType::Ceramics, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Build);

			// Build with non-materials fails
			let sacrifices_err =
				[&create_random_blueprint(&ALICE, pet_type, slot_type, equip_type, pattern, 4).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_equip() {
		ExtBuilder::default().build().execute_with(|| {
			ExtBuilder::default().build().execute_with(|| {
				let forger = AvatarForgerV2::<Test>(PhantomData);

				// Equip
				let (_, leader) = create_random_pet(
					&ALICE,
					PetType::TankyBullwog,
					0b0001_0001,
					[0; 16],
					[0; 11],
					2,
				);
				let sacrifices = [&create_random_armor_component(
					[0x2A; 32],
					&ALICE,
					PetType::TankyBullwog,
					SlotType::ArmBack,
					RarityType::Legendary,
					vec![EquipableItemType::ArmorBase],
					(ColorType::None, ColorType::ColorD),
					ForceType::Astral,
					2,
				)
				.1];
				assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Equip);

				// Equip without armor-parts fails
				let sacrifices_err =
					[&create_random_material(&ALICE, MaterialItemType::Polymers, 4).1];
				assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

				// Equip with incompatible armor component fails
				let sacrifices_err = [&create_random_armor_component(
					[0x2A; 32],
					&ALICE,
					PetType::FoxishDude,
					SlotType::ArmBack,
					RarityType::Common,
					vec![EquipableItemType::ArmorComponent2],
					(ColorType::None, ColorType::ColorD),
					ForceType::Astral,
					2,
				)
				.1];
				assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
			});
		});
	}

	#[test]
	fn test_determine_forge_types_feed() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Feed
			let (_, leader) =
				create_random_pet(&ALICE, PetType::TankyBullwog, 0b0001_0001, [0; 16], [0; 11], 2);
			let sacrifices =
				[&create_random_egg(None, &ALICE, RarityType::Common, 0b0001_0010, 10, [2; 11]).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Feed);

			// Feed with Legendary egg sacrifices fails
			let sacrifices_err =
				[&create_random_egg(None, &ALICE, RarityType::Legendary, 0b0001_0010, 10, [2; 11])
					.1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

			// Feed with non-eggs fails
			let sacrifices_err = [&create_random_material(&ALICE, MaterialItemType::Metals, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_glimmer() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Glimmer
			let (_, leader) = create_random_glimmer(&ALICE, 5);
			let sacrifices = [
				&create_random_material(&ALICE, MaterialItemType::Ceramics, 4).1,
				&create_random_material(&ALICE, MaterialItemType::Nanomaterials, 5).1,
			];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Glimmer);

			// Glimmer without enough materials fails
			let sacrifices_err = [
				&create_random_material(&ALICE, MaterialItemType::Polymers, 2).1,
				&create_random_material(&ALICE, MaterialItemType::Optics, 4).1,
			];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

			// Glimmer without enough glimmer amount fails
			let (_, leader_err) = create_random_glimmer(&ALICE, 1);
			assert_eq!(forger.determine_forge_type(&leader_err, &sacrifices), ForgeType::None);

			// Glimmer without material sacrifices fails
			let sacrifices_err =
				[&create_random_egg(None, &ALICE, RarityType::Legendary, 0b0001_0010, 10, [2; 11])
					.1];
			assert_eq!(forger.determine_forge_type(&leader_err, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_mate() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Mate
			let (_, leader) =
				create_random_pet(&ALICE, PetType::TankyBullwog, 0b0001_0001, [0; 16], [0; 11], 2);
			let sacrifices =
				[&create_random_pet(&ALICE, PetType::CrazyDude, 0b0001_0001, [0; 16], [0; 11], 2)
					.1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Mate);

			// Mate with non-pet fails
			let sacrifices_err =
				[&create_random_egg(None, &ALICE, RarityType::Legendary, 0b0001_0010, 10, [2; 11])
					.1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_spark() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Spark with ColorSpark
			let (_, leader_color) = create_random_color_spark(
				None,
				&ALICE,
				(ColorType::ColorA, ColorType::ColorC),
				100,
				None,
			);
			let sacrifices_color = [&create_random_color_spark(
				None,
				&ALICE,
				(ColorType::ColorC, ColorType::ColorD),
				3,
				None,
			)
			.1];
			assert_eq!(
				forger.determine_forge_type(&leader_color, &sacrifices_color),
				ForgeType::Spark
			);

			// Spark with GlowSpark
			let (_, leader_glow) =
				create_random_glow_spark(None, &ALICE, ForceType::Kinetic, 100, None);
			let sacrifices_glow =
				[&create_random_glow_spark(None, &ALICE, ForceType::Thermal, 100, None).1];
			assert_eq!(
				forger.determine_forge_type(&leader_glow, &sacrifices_glow),
				ForgeType::Spark
			);

			// Spark with incompatible spark types fails
			assert_eq!(
				forger.determine_forge_type(&leader_glow, &sacrifices_color),
				ForgeType::None
			);
			assert_eq!(
				forger.determine_forge_type(&leader_color, &sacrifices_glow),
				ForgeType::None
			);

			// Spark without GlowSpark or ColorSpark fails
			let sacrifices_err =
				[&create_random_egg(None, &ALICE, RarityType::Legendary, 0b0001_0010, 10, [2; 11])
					.1];
			assert_eq!(
				forger.determine_forge_type(&leader_color, &sacrifices_err),
				ForgeType::None
			);
			assert_eq!(forger.determine_forge_type(&leader_glow, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_stack() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Stack Materials
			let (_, leader) = create_random_material(&ALICE, MaterialItemType::Polymers, 10);
			let sacrifices = [&create_random_material(&ALICE, MaterialItemType::Polymers, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Stack);
			// Stack different Materials fails
			let sacrifices_err = [&create_random_material(&ALICE, MaterialItemType::Metals, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

			// Stack Dust
			let (_, leader) = create_random_dust(&ALICE, 10);
			let sacrifices = [&create_random_dust(&ALICE, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Stack);
			// Stack dust with non-dust fails
			let sacrifices_err = [&create_random_material(&ALICE, MaterialItemType::Metals, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

			// Stack Glimmer
			let (_, leader) = create_random_glimmer(&ALICE, 1);
			let sacrifices = [&create_random_glimmer(&ALICE, 2).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Stack);
			// Stack Glimmer with different non-glimmer fails
			let sacrifices_err = [&create_random_dust(&ALICE, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);

			// Stack PetPart
			let (_, leader) =
				create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::Head, 1);
			let sacrifices =
				[&create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::Head, 1).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Stack);
			// Stack PetPart with different PetType fails
			let sacrifices_err =
				[&create_random_pet_part(&ALICE, PetType::BigHybrid, SlotType::Head, 1).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}

	#[test]
	fn test_determine_forge_types_tinker() {
		ExtBuilder::default().build().execute_with(|| {
			let forger = AvatarForgerV2::<Test>(PhantomData);

			// Tinker
			let (_, leader) =
				create_random_pet_part(&ALICE, PetType::FoxishDude, SlotType::Head, 1);
			let sacrifices = [&create_random_material(&ALICE, MaterialItemType::Polymers, 10).1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices), ForgeType::Tinker);

			// Tinker with non-materials fails
			let sacrifices_err = [&create_random_color_spark(
				None,
				&ALICE,
				(ColorType::ColorA, ColorType::ColorC),
				10,
				None,
			)
			.1];
			assert_eq!(forger.determine_forge_type(&leader, &sacrifices_err), ForgeType::None);
		});
	}
}
