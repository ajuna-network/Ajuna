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
		DnaUtils::read_attribute_raw(target, AvatarAttr::RarityTier)
	}

	fn force(target: &Avatar) -> u8 {
		// TODO: Determine proper mapping
		DnaUtils::read_spec_raw(target, SpecIdx::Byte1)
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
			.map(|i| {
				let rolled_item_type = SlotRoller::<T>::roll_on_pack_type(
					mint_option.pack_type.clone(),
					&PACK_TYPE_MATERIAL_ITEM_PROBABILITIES,
					&PACK_TYPE_EQUIPMENT_ITEM_PROBABILITIES,
					&PACK_TYPE_SPECIAL_ITEM_PROBABILITIES,
					&mut hash_provider,
				);

				let avatar_id = hash_provider.full_hash(i);

				let base_dna = Self::generate_empty_dna::<32>()?;
				let base_avatar = Avatar {
					season_id: *season_id,
					encoding: DnaEncoding::V2,
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

				hash_provider = HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(
					b"avatar_minter_v2",
					player,
				));

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

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
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
	Statue,
	Flask,
}

impl ForgeType {
	pub(crate) fn is_restricted(&self) -> bool {
		matches!(self, ForgeType::Tinker | ForgeType::Build | ForgeType::Mate | ForgeType::Glimmer)
	}
}

pub(crate) struct ForgerV2<T: Config>(pub PhantomData<T>);

impl<T: Config> Forger<T> for ForgerV2<T> {
	fn forge(
		player: &T::AccountId,
		season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		restricted: bool,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let mut hash_provider =
			HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(b"avatar_forger_v2", player));

		ensure!(
			input_sacrifices.len() >= MIN_SACRIFICE && input_sacrifices.len() <= MAX_SACRIFICE,
			Error::<T>::IncompatibleAvatarVersions
		);

		let (leader_id, leader) = input_leader;
		let wrapped_leader = WrappedAvatar::new(leader);

		let sacrifices = input_sacrifices
			.into_iter()
			.map(|(id, sacrifice)| (id, WrappedAvatar::new(sacrifice)))
			.collect::<Vec<_>>();
		let wrapped_sacrifices = sacrifices.iter().map(|(_, avatar)| avatar).collect::<Vec<_>>();

		let forge_type = Self::determine_forge_type(&wrapped_leader, wrapped_sacrifices.as_slice());

		ensure!(
			!restricted || !forge_type.is_restricted(),
			Error::<T>::InsufficientStorageForForging
		);

		AvatarCombinator::<T>::combine_avatars_in(
			forge_type,
			season_id,
			season,
			(leader_id, wrapped_leader),
			sacrifices,
			&mut hash_provider,
		)
	}
}

impl<T: Config> ForgerV2<T> {
	fn determine_forge_type(leader: &WrappedAvatar, sacrifices: &[&WrappedAvatar]) -> ForgeType {
		match leader.get_item_type() {
			ItemType::Pet => match leader.get_item_sub_type::<PetItemType>() {
				PetItemType::Pet => {
					if sacrifices.iter().all(|sacrifice| {
						let equippable_item = sacrifice.get_item_sub_type::<EquippableItemType>();
						sacrifice.has_type(ItemType::Equippable) &&
							sacrifice.same_rarity(leader) &&
							sacrifice.same_class_type_2(leader) &&
							(equippable_item.is_armor_base() || equippable_item.is_weapon())
					}) {
						ForgeType::Equip
					} else if sacrifices.iter().all(|sacrifice| {
						sacrifice.same_full_type(leader) && sacrifice.same_rarity(leader)
					}) {
						ForgeType::Mate
					} else if sacrifices.iter().all(|sacrifice| {
						sacrifice.same_item_type(leader) && sacrifice.has_subtype(PetItemType::Egg)
					}) {
						ForgeType::Feed
					} else {
						ForgeType::None
					}
				},
				PetItemType::PetPart => {
					if leader.has_zeroed_class_types() &&
						sacrifices.iter().all(|sacrifice| {
							sacrifice.same_full_type(leader) && sacrifice.has_zeroed_class_types()
						}) {
						ForgeType::Statue
					} else if sacrifices.iter().all(|sacrifice| {
						sacrifice.same_full_type(leader) &&
							sacrifice.same_class_type_2(leader) &&
							sacrifice.get_class_type_2::<HexType>() != HexType::X0
					}) {
						ForgeType::Stack
					} else if leader.get_class_type_1::<HexType>() != HexType::X0 &&
						leader.get_class_type_2::<HexType>() != HexType::X0 &&
						sacrifices.iter().all(|sacrifice| sacrifice.has_type(ItemType::Material))
					{
						ForgeType::Tinker
					} else {
						ForgeType::None
					}
				},
				PetItemType::Egg => {
					let leader_rarity = leader.get_rarity();

					if leader_rarity <= RarityTier::Epic &&
						sacrifices.iter().all(|sacrifice| sacrifice.same_full_type(leader))
					{
						ForgeType::Breed
					} else {
						ForgeType::None
					}
				},
			},
			ItemType::Material => {
				if sacrifices.iter().all(|sacrifice| sacrifice.same_full_type(leader)) {
					ForgeType::Stack
				} else {
					ForgeType::None
				}
			},
			ItemType::Essence => match leader.get_item_sub_type::<EssenceItemType>() {
				EssenceItemType::Glimmer => {
					if leader.get_quantity() as usize >= sacrifices.len() &&
						sacrifices.iter().all(|sacrifice| {
							sacrifice.has_type(ItemType::Material) && sacrifice.get_quantity() >= 4
						}) {
						ForgeType::Glimmer
					} else if sacrifices.iter().all(|sacrifice| sacrifice.same_full_type(leader)) {
						ForgeType::Stack
					} else {
						ForgeType::None
					}
				},
				EssenceItemType::ColorSpark | EssenceItemType::GlowSpark => {
					if sacrifices.iter().all(|sacrifice| sacrifice.same_full_type(leader)) {
						ForgeType::Spark
					} else {
						ForgeType::None
					}
				},
				EssenceItemType::PaintFlask | EssenceItemType::GlowFlask => ForgeType::None,
			},
			ItemType::Equippable => {
				let leader_sub_type = leader.get_item_sub_type::<EquippableItemType>();
				let leader_rarity = leader.get_rarity();

				let all_sacrifice_are_armor_or_toolbox = sacrifices.iter().all(|sacrifice| {
					let same_assemble_version = sacrifice.same_assemble_version(leader);
					let equipable_sacrifice_item =
						sacrifice.get_item_sub_type::<EquippableItemType>();

					let is_toolbox =
						sacrifice.has_full_type(ItemType::Special, SpecialItemType::ToolBox);

					(same_assemble_version && equipable_sacrifice_item.is_armor()) || is_toolbox
				});

				let sacrificed_toolboxes = sacrifices
					.iter()
					.filter(|sacrifice| {
						sacrifice.has_full_type(ItemType::Special, SpecialItemType::ToolBox)
					})
					.count();

				if leader_sub_type.is_armor() &&
					all_sacrifice_are_armor_or_toolbox &&
					sacrificed_toolboxes <= MAX_TOOLBOXES
				{
					ForgeType::Assemble
				} else if leader_rarity == RarityTier::Epic && leader_sub_type.is_armor_base() {
					let has_one_paint_flask_or_glow = sacrifices
						.iter()
						.filter(|sacrifice| {
							sacrifice
								.has_type(ItemType::Essence)
								.then(|| {
									let item_sub_type =
										sacrifice.get_item_sub_type::<EssenceItemType>();

									item_sub_type == EssenceItemType::PaintFlask ||
										item_sub_type == EssenceItemType::GlowFlask
								})
								.unwrap_or(false)
						})
						.count() == 1;

					let all_are_glimmer_paint_or_force = sacrifices.iter().all(|sacrifice| {
						sacrifice
							.has_type(ItemType::Essence)
							.then(|| {
								let item_sub_type =
									sacrifice.get_item_sub_type::<EssenceItemType>();

								item_sub_type == EssenceItemType::Glimmer ||
									item_sub_type == EssenceItemType::PaintFlask ||
									item_sub_type == EssenceItemType::GlowFlask
							})
							.unwrap_or(false)
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
				if sacrifices.iter().all(|sacrifice| {
					sacrifice.same_full_and_class_types(leader) &&
						sacrifice.same_spec_at(leader, SpecIdx::Byte3)
				}) {
					ForgeType::Stack
				} else if sacrifices.len() == 4 &&
					sacrifices.iter().all(|sacrifice| sacrifice.has_type(ItemType::Material))
				{
					ForgeType::Build
				} else {
					ForgeType::None
				}
			},
			ItemType::Special => match leader.get_item_sub_type::<SpecialItemType>() {
				SpecialItemType::Dust =>
					if sacrifices.iter().all(|sacrifice| sacrifice.same_full_type(leader)) {
						ForgeType::Stack
					} else {
						ForgeType::None
					},
				_ => ForgeType::None,
			},
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;
	use sp_std::collections::btree_map::BTreeMap;

	#[test]
	fn test_can_be_forged() {
		ExtBuilder::default().build().execute_with(|| {
			let season = Season::default();

			let leader = {
				let wrapped_leader =
					create_random_material(&ALICE, &MaterialItemType::Polymers, 10);
				(wrapped_leader.0, wrapped_leader.1.unwrap())
			};
			let sacrifices = [
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
				create_random_material(&ALICE, &MaterialItemType::Polymers, 10),
			]
			.into_iter()
			.map(|(sac_id, sac)| (sac_id, sac.unwrap()))
			.collect::<Vec<_>>();

			// Can forge with V2 avatar and correct number of sacrifices
			assert!(ForgerV2::<Test>::forge(
				&ALICE,
				1,
				&season,
				leader.clone(),
				sacrifices[0..4].to_vec(),
				false
			)
			.is_ok());

			// Can't forge with more than MAX_SACRIFICE amount
			assert!(ForgerV2::<Test>::forge(
				&ALICE,
				1,
				&season,
				leader.clone(),
				sacrifices.to_vec(),
				false
			)
			.is_err());

			// Can't forge with less than MIN_SACRIFICE amount
			assert!(
				ForgerV2::<Test>::forge(&ALICE, 1, &season, leader, [].to_vec(), false).is_err()
			);
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
				&(ColorType::ColorA, ColorType::Null),
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
				&(ColorType::Null, ColorType::ColorD),
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
				&(ColorType::Null, ColorType::ColorD),
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
			let pattern = DnaUtils::create_pattern::<MaterialItemType>(
				base_seed,
				equip_type.as_byte() as usize,
			);

			// Build
			let (_, leader) =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 2);
			let sacrifices = [
				&create_random_material(&ALICE, &MaterialItemType::Ceramics, 10).1,
				&create_random_material(&ALICE, &MaterialItemType::Nanomaterials, 10).1,
				&create_random_material(&ALICE, &MaterialItemType::Polymers, 10).1,
				&create_random_material(&ALICE, &MaterialItemType::Optics, 10).1,
			];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::Build
			);

			// Build without enough materials fails
			let (_, leader) =
				create_random_blueprint(&ALICE, &pet_type, &slot_type, &equip_type, &pattern, 2);
			let sacrifices = [
				&create_random_material(&ALICE, &MaterialItemType::Electronics, 10).1,
				&create_random_material(&ALICE, &MaterialItemType::Metals, 10).1,
			];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
				ForgeType::None
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
					&(ColorType::Null, ColorType::ColorD),
					&Force::Astral,
					2,
					&mut hash_provider,
				)
				.1];
				assert_eq!(
					ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices),
					ForgeType::Equip
				);

				// Equip with Mythical
				let (_, leader) = {
					let (id, mut leader) = create_random_pet(
						&ALICE,
						&PetType::TankyBullwog,
						0b0001_0001,
						[0; 16],
						[0; 11],
						2,
					);
					leader.set_rarity(RarityTier::Mythical);

					(id, leader)
				};
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
					&(ColorType::Null, ColorType::ColorD),
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

			// Feed with Legendary egg sacrifices doesn't fail anymore
			let sacrifices_err = [&create_random_egg(
				None,
				&ALICE,
				&RarityTier::Legendary,
				0b0000_0000,
				10,
				[2; 11],
			)
			.1];
			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &sacrifices_err),
				ForgeType::Feed
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
				&(ColorType::ColorA, ColorType::Null),
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
				&(ColorType::Null, ColorType::ColorD),
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

			// Mate Mythical
			let (_, leader) = {
				let (id, mut leader) = create_random_pet(
					&ALICE,
					&PetType::TankyBullwog,
					0b0001_0001,
					[0; 16],
					[0; 11],
					2,
				);

				leader.set_rarity(RarityTier::Mythical);

				(id, leader)
			};
			let sacrifices = {
				let mut avatar = create_random_pet(
					&ALICE,
					&PetType::CrazyDude,
					0b0001_0001,
					[0; 16],
					[0; 11],
					2,
				)
				.1;

				avatar.set_rarity(RarityTier::Mythical);

				avatar
			};

			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader, &[&sacrifices]),
				ForgeType::Mate
			);

			// Mate with non-pet fails
			let sacrifices_err =
				[&create_random_pet_part(&ALICE, &PetType::FoxishDude, &SlotType::Head, 1).1];
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

			let unit_fn = |avatar: Avatar| {
				let mut avatar = avatar;
				avatar.souls = 100;
				WrappedAvatar::new(avatar)
			};

			let leader = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x12, 0x12, 0x12, 0x01, 0x00, 0x6C, 0x78, 0xD8, 0x72, 0x55, 0x78, 0x66, 0x6C,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				]),
				Some(unit_fn),
			);
			let sac_1 = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x42, 0x12, 0x01, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x13, 0x14, 0x13, 0x14,
					0x11, 0x22, 0x10, 0x14, 0x22, 0x11,
				]),
				Some(unit_fn),
			);

			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader.1, &[&sac_1.1]),
				ForgeType::None
			);

			let leader = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x22, 0x00, 0x11, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				]),
				Some(unit_fn),
			);
			let sac_1 = create_random_avatar::<Test, _>(
				&ALICE,
				Some([
					0x42, 0x12, 0x01, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
					0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x13, 0x14, 0x13, 0x14,
					0x11, 0x22, 0x10, 0x14, 0x22, 0x11,
				]),
				Some(unit_fn),
			);

			assert_eq!(
				ForgerV2::<Test>::determine_forge_type(&leader.1, &[&sac_1.1]),
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

	#[test]
	fn test_forge_type_distribution() {
		ExtBuilder::default().build().execute_with(|| {
			let hash = Pallet::<Test>::random_hash(b"forge_type_distribution", &ALICE);
			let mut hash_provider = HashProvider::<Test, 32>::new(&hash);

			let leader_possible_item_types = vec![
				ItemType::Pet,
				ItemType::Material,
				ItemType::Essence,
				ItemType::Equippable,
				ItemType::Blueprint,
			];

			let item_type_selection_fn = |item_type: ItemType| match item_type {
				ItemType::Pet => vec![ItemType::Pet, ItemType::Equippable, ItemType::Material],
				ItemType::Material => vec![ItemType::Material],
				ItemType::Essence => vec![ItemType::Essence],
				ItemType::Equippable => vec![ItemType::Equippable, ItemType::Essence],
				ItemType::Blueprint => vec![ItemType::Material],
				ItemType::Special => vec![],
			};

			let avatar_creation_fn = |allowed_item_types: &Vec<ItemType>,
			                          hash_provider: &HashProvider<Test, 32>,
			                          i: usize| {
				let mut random_hash = HashProvider::<Test, 32>::new(&hash_provider.full_hash(i));

				let item_type =
					allowed_item_types[random_hash.next() as usize % allowed_item_types.len()];

				move |avatar: Avatar| {
					let mut avatar = avatar;

					let class_type_1 = SlotType::from_byte(
						(random_hash.next() % SlotType::range().end as u8) +
							SlotType::range().start as u8,
					);
					let class_type_2 = PetType::from_byte(
						(random_hash.next() % PetType::range().end as u8) +
							PetType::range().start as u8,
					);
					let rarity_type = RarityTier::from_byte(
						(random_hash.next() % RarityTier::Mythical.as_byte()) + 1,
					);

					let item_sub_type = match item_type {
						ItemType::Pet => HexType::from_byte(
							(random_hash.next() % PetItemType::Egg.as_byte()) + 1,
						),
						ItemType::Material => HexType::from_byte(
							(random_hash.next() % MaterialItemType::Nanomaterials.as_byte()) + 1,
						),
						ItemType::Essence => HexType::from_byte(
							(random_hash.next() % EssenceItemType::GlowFlask.as_byte()) + 1,
						),
						ItemType::Equippable => HexType::from_byte(
							(random_hash.next() % EquippableItemType::WeaponVersion3.as_byte()) + 1,
						),
						ItemType::Blueprint | ItemType::Special => HexType::X0,
					};

					DnaUtils::write_attribute(&mut avatar, AvatarAttr::ItemType, &item_type);
					DnaUtils::write_attribute(&mut avatar, AvatarAttr::ItemSubType, &item_sub_type);
					DnaUtils::write_attribute(&mut avatar, AvatarAttr::ClassType1, &class_type_1);
					DnaUtils::write_attribute(&mut avatar, AvatarAttr::ClassType2, &class_type_2);
					DnaUtils::write_attribute(&mut avatar, AvatarAttr::RarityTier, &rarity_type);

					WrappedAvatar::new(avatar)
				}
			};

			let max_iterations = 100_000_usize;

			let mut forge_type_map = BTreeMap::new();

			forge_type_map.insert(ForgeType::None, 0);
			forge_type_map.insert(ForgeType::Stack, 0);
			forge_type_map.insert(ForgeType::Tinker, 0);
			forge_type_map.insert(ForgeType::Build, 0);
			forge_type_map.insert(ForgeType::Assemble, 0);
			forge_type_map.insert(ForgeType::Breed, 0);
			forge_type_map.insert(ForgeType::Equip, 0);
			forge_type_map.insert(ForgeType::Mate, 0);
			forge_type_map.insert(ForgeType::Feed, 0);
			forge_type_map.insert(ForgeType::Glimmer, 0);
			forge_type_map.insert(ForgeType::Spark, 0);
			forge_type_map.insert(ForgeType::Statue, 0);
			forge_type_map.insert(ForgeType::Flask, 0);

			for i in 0..max_iterations {
				let leader = create_random_avatar::<Test, _>(
					&ALICE,
					None,
					Some(avatar_creation_fn(&leader_possible_item_types, &hash_provider, i)),
				)
				.1;

				let leader_item_type = leader.get_item_type();

				let sacrifice_count = (hash_provider.next() % 4) as usize + 1;
				let mut sacrifice_list = Vec::with_capacity(sacrifice_count);
				for _ in 0..sacrifice_count {
					let sacrifice = create_random_avatar::<Test, _>(
						&ALICE,
						None,
						Some(avatar_creation_fn(
							&item_type_selection_fn(leader_item_type),
							&hash_provider,
							i,
						)),
					);
					sacrifice_list.push(sacrifice.1);
				}

				let forge_type = ForgerV2::<Test>::determine_forge_type(
					&leader,
					sacrifice_list.iter().collect::<Vec<_>>().as_slice(),
				);

				forge_type_map
					.entry(forge_type)
					.and_modify(|value| *value += 1)
					.or_insert(1_u32);

				if i % 1000 == 999 {
					let hash_text = format!("hash_loop_{:#07X}", i);
					let hash = Pallet::<Test>::random_hash(hash_text.as_bytes(), &ALICE);
					hash_provider = HashProvider::new(&hash);
				}
			}

			assert_eq!(forge_type_map.get(&ForgeType::None).unwrap(), &48203);
			assert_eq!(forge_type_map.get(&ForgeType::Stack).unwrap(), &26687);
			assert_eq!(forge_type_map.get(&ForgeType::Tinker).unwrap(), &2407);
			assert_eq!(forge_type_map.get(&ForgeType::Build).unwrap(), &4723);
			assert_eq!(forge_type_map.get(&ForgeType::Assemble).unwrap(), &5620);
			assert_eq!(forge_type_map.get(&ForgeType::Breed).unwrap(), &1558);
			assert_eq!(forge_type_map.get(&ForgeType::Equip).unwrap(), &848);
			assert_eq!(forge_type_map.get(&ForgeType::Mate).unwrap(), &2122);
			assert_eq!(forge_type_map.get(&ForgeType::Feed).unwrap(), &0);
			assert_eq!(forge_type_map.get(&ForgeType::Glimmer).unwrap(), &0);
			assert_eq!(forge_type_map.get(&ForgeType::Spark).unwrap(), &7832);
			assert_eq!(forge_type_map.get(&ForgeType::Statue).unwrap(), &0);
			assert_eq!(forge_type_map.get(&ForgeType::Flask).unwrap(), &0);
		});
	}
}
