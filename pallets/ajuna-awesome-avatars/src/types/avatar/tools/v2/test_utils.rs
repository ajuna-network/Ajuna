use crate::{
	mock::{MockAccountId, Test},
	pallet::AvatarIdOf,
	types::{
		avatar::tools::v2::{
			avatar_utils::{AvatarAttributes, AvatarBuilder, AvatarUtils},
			types::{
				BlueprintItemType, ColorType, EquipableItemType, ForceType, MaterialItemType,
				PetType, RarityType, SlotType,
			},
		},
		Avatar, AvatarVersion, ForgeOutput, LeaderForgeOutput, SoulCount,
	},
	Config, Pallet,
};
use sp_core::bounded::BoundedVec;

pub const HASH_BYTES: [u8; 32] = [
	1, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
	97, 101, 103, 107, 109, 113, 127,
];

pub(crate) fn create_random_avatar<T, F>(
	creator: &T::AccountId,
	initial_dna: Option<[u8; 32]>,
	avatar_build_fn: Option<F>,
) -> (AvatarIdOf<T>, Avatar)
where
	F: FnOnce(Avatar) -> Avatar,
	T: Config,
{
	let base_avatar = Avatar {
		season_id: 0,
		version: AvatarVersion::V2,
		dna: BoundedVec::try_from(initial_dna.unwrap_or([0_u8; 32]).to_vec())
			.expect("Should create DNA!"),
		souls: 0,
	};

	let avatar = match avatar_build_fn {
		None => base_avatar,
		Some(f) => f(base_avatar),
	};
	(Pallet::<T>::random_hash(b"mock_avatar", creator), avatar)
}

pub(crate) fn create_random_material(
	account: &MockAccountId,
	material_type: MaterialItemType,
	quantity: u8,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_material(material_type, quantity)
				.build()
		}),
	)
}

pub(crate) fn create_random_pet_part(
	account: &MockAccountId,
	pet_type: PetType,
	slot_type: SlotType,
	quantity: u8,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_pet_part(pet_type, slot_type, quantity)
				.build()
		}),
	)
}

pub(crate) fn create_random_pet(
	account: &MockAccountId,
	pet_type: PetType,
	pet_variation: u8,
	spec_bytes: [u8; 16],
	progress_array: [u8; 11],
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_pet(pet_type, pet_variation, spec_bytes, Some(progress_array), soul_points)
				.build()
		}),
	)
}

pub(crate) fn create_random_blueprint(
	account: &MockAccountId,
	pet_type: PetType,
	slot_type: SlotType,
	equipable_type: EquipableItemType,
	material_pattern: Vec<MaterialItemType>,
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_blueprint(
					BlueprintItemType::Blueprint,
					pet_type,
					slot_type,
					equipable_type,
					material_pattern,
					soul_points,
				)
				.build()
		}),
	)
}

pub(crate) fn create_random_armor_component(
	base_dna: [u8; 32],
	account: &MockAccountId,
	pet_type: PetType,
	slot_type: SlotType,
	rarity_type: RarityType,
	equipable_type: Vec<EquipableItemType>,
	color_pair: (ColorType, ColorType),
	force_type: ForceType,
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		Some(base_dna),
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.try_into_armor_and_component(
					pet_type,
					slot_type,
					equipable_type,
					rarity_type,
					color_pair,
					force_type,
					soul_points,
				)
				.unwrap()
				.build()
		}),
	)
}

pub(crate) fn create_random_weapon(
	base_dna: [u8; 32],
	account: &MockAccountId,
	pet_type: PetType,
	slot_type: SlotType,
	equipable_type: EquipableItemType,
	color_pair: (ColorType, ColorType),
	force_type: ForceType,
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		Some(base_dna),
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.try_into_weapon(
					pet_type,
					slot_type,
					equipable_type,
					color_pair,
					force_type,
					soul_points,
				)
				.unwrap()
				.build()
		}),
	)
}

pub(crate) fn create_random_egg(
	base_dna: Option<[u8; 32]>,
	account: &MockAccountId,
	rarity_type: RarityType,
	pet_variation: u8,
	soul_points: SoulCount,
	progress_array: [u8; 11],
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		base_dna,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_egg(rarity_type, pet_variation, soul_points, Some(progress_array))
				.build()
		}),
	)
}

pub(crate) fn create_random_glow_spark(
	base_dna: Option<[u8; 32]>,
	account: &MockAccountId,
	force_type: ForceType,
	soul_points: SoulCount,
	progress_array: Option<[u8; 11]>,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		base_dna,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_glow_spark(force_type, soul_points, progress_array)
				.build()
		}),
	)
}

pub(crate) fn create_random_glimmer(
	account: &MockAccountId,
	quantity: u8,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| AvatarBuilder::with_base_avatar(avatar).into_glimmer(quantity).build()),
	)
}

pub(crate) fn create_random_dust(
	account: &MockAccountId,
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| AvatarBuilder::with_base_avatar(avatar).into_dust(soul_points).build()),
	)
}

pub(crate) fn create_random_color_spark(
	base_dna: Option<[u8; 32]>,
	account: &MockAccountId,
	color_pair: (ColorType, ColorType),
	soul_points: SoulCount,
	progress_array: Option<[u8; 11]>,
) -> (AvatarIdOf<Test>, Avatar) {
	create_random_avatar::<Test, _>(
		account,
		base_dna,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_color_spark(color_pair, soul_points, progress_array)
				.build()
		}),
	)
}

pub(crate) fn is_leader_forged<T>(output: &LeaderForgeOutput<T>) -> bool
where
	T: Config,
{
	matches!(output, LeaderForgeOutput::Forged(_, _))
}

pub(crate) fn is_leader_forged_with_attributes<T>(
	output: &LeaderForgeOutput<T>,
	attributes: &[(AvatarAttributes, u8)],
) -> bool
where
	T: Config,
{
	matches!(output, LeaderForgeOutput::Forged((_, avatar), _) if AvatarUtils::has_attribute_set_with_values(avatar, attributes))
}

pub(crate) fn is_leader_consumed<T>(output: &LeaderForgeOutput<T>) -> bool
where
	T: Config,
{
	matches!(output, LeaderForgeOutput::Consumed(_))
}

pub(crate) fn is_forged<T>(output: &ForgeOutput<T>) -> bool
where
	T: Config,
{
	matches!(output, ForgeOutput::Forged(_, _))
}

pub(crate) fn is_forged_with_attributes<T>(
	output: &ForgeOutput<T>,
	attributes: &[(AvatarAttributes, u8)],
) -> bool
where
	T: Config,
{
	matches!(output, ForgeOutput::Forged((_, avatar), _) if AvatarUtils::has_attribute_set_with_values(avatar, attributes))
}

pub(crate) fn is_minted<T>(output: &ForgeOutput<T>) -> bool
where
	T: Config,
{
	matches!(output, ForgeOutput::Minted(_))
}

pub(crate) fn is_minted_with_attributes<T>(
	output: &ForgeOutput<T>,
	attributes: &[(AvatarAttributes, u8)],
) -> bool
where
	T: Config,
{
	matches!(output, ForgeOutput::Minted(avatar) if AvatarUtils::has_attribute_set_with_values(avatar, attributes))
}

pub(crate) fn is_consumed<T>(output: &ForgeOutput<T>) -> bool
where
	T: Config,
{
	matches!(output, ForgeOutput::Consumed(_))
}
