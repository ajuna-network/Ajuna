use crate::{
	mock::{MockAccountId, Test},
	pallet::AvatarIdOf,
	types::{
		avatar::versions::v2::{
			avatar_utils::{AvatarBuilder, HashProvider, WrappedAvatar},
			types::{
				BlueprintItemType, ColorType, EquippableItemType, MaterialItemType, PetType,
				SlotType,
			},
		},
		Avatar, DnaEncoding, ForgeOutput, LeaderForgeOutput, RarityTier, SoulCount,
	},
	Config, Force, Pallet,
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
) -> (AvatarIdOf<T>, WrappedAvatar)
where
	F: FnOnce(Avatar) -> WrappedAvatar,
	T: Config,
{
	let base_avatar = Avatar {
		season_id: 0,
		encoding: DnaEncoding::V2,
		dna: BoundedVec::try_from(initial_dna.unwrap_or([0_u8; 32]).to_vec())
			.expect("Should create DNA!"),
		souls: 0,
	};

	let avatar = match avatar_build_fn {
		None => WrappedAvatar::new(base_avatar),
		Some(f) => f(base_avatar),
	};
	(Pallet::<T>::random_hash(b"mock_avatar", creator), avatar)
}

pub(crate) fn create_random_material(
	account: &MockAccountId,
	material_type: &MaterialItemType,
	quantity: u8,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_material(material_type, quantity)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_pet_part(
	account: &MockAccountId,
	pet_type: &PetType,
	slot_type: &SlotType,
	quantity: u8,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_pet_part(pet_type, slot_type, quantity)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_generic_part(
	account: &MockAccountId,
	slot_types: &[SlotType],
	quantity: u8,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_generic_pet_part(slot_types, quantity)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_pet(
	account: &MockAccountId,
	pet_type: &PetType,
	pet_variation: u8,
	spec_bytes: [u8; 16],
	progress_array: [u8; 11],
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_pet(pet_type, pet_variation, spec_bytes, progress_array, soul_points)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_blueprint(
	account: &MockAccountId,
	pet_type: &PetType,
	slot_type: &SlotType,
	equippable_type: &EquippableItemType,
	material_pattern: &[MaterialItemType],
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_blueprint(
					&BlueprintItemType::Blueprint,
					pet_type,
					slot_type,
					equippable_type,
					material_pattern,
					soul_points as u8,
				)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_armor_component(
	base_dna: [u8; 32],
	account: &MockAccountId,
	pet_type: &PetType,
	slot_type: &SlotType,
	rarity: &RarityTier,
	equippable_type: &[EquippableItemType],
	color_pair: &(ColorType, ColorType),
	force: &Force,
	soul_points: SoulCount,
	hash_provider: &mut HashProvider<Test, 32>,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		Some(base_dna),
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.try_into_armor_and_component(
					pet_type,
					slot_type,
					equippable_type,
					rarity,
					color_pair,
					force,
					soul_points,
					hash_provider,
				)
				.unwrap()
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_weapon(
	base_dna: [u8; 32],
	account: &MockAccountId,
	pet_type: &PetType,
	slot_type: &SlotType,
	equippable_type: &EquippableItemType,
	color_pair: &(ColorType, ColorType),
	force: &Force,
	soul_points: SoulCount,
	hash_provider: &mut HashProvider<Test, 32>,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		Some(base_dna),
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.try_into_weapon(
					pet_type,
					slot_type,
					equippable_type,
					color_pair,
					force,
					soul_points,
					hash_provider,
				)
				.unwrap()
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_toolbox(
	base_dna: [u8; 32],
	account: &MockAccountId,
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		Some(base_dna),
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_toolbox(soul_points)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_egg(
	base_dna: Option<[u8; 32]>,
	account: &MockAccountId,
	rarity: &RarityTier,
	pet_variation: u8,
	soul_points: SoulCount,
	progress_array: [u8; 11],
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		base_dna,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_egg(rarity, pet_variation, soul_points, progress_array)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_glow_spark(
	base_dna: Option<[u8; 32]>,
	account: &MockAccountId,
	force: &Force,
	soul_points: SoulCount,
	progress_array: [u8; 11],
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		base_dna,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_glow_spark(force, soul_points, progress_array)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_glimmer(
	account: &MockAccountId,
	quantity: u8,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar).into_glimmer(quantity).build_wrapped()
		}),
	)
}

pub(crate) fn create_random_paint_flask(
	account: &MockAccountId,
	color_pair: &(ColorType, ColorType),
	soul_points: SoulCount,
	progress_array: [u8; 11],
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_paint_flask(color_pair, soul_points, progress_array)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_glow_flask(
	account: &MockAccountId,
	force_type: &Force,
	soul_points: SoulCount,
	progress_array: [u8; 11],
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_glow_flask(force_type, soul_points, progress_array)
				.build_wrapped()
		}),
	)
}

pub(crate) fn create_random_dust(
	account: &MockAccountId,
	soul_points: SoulCount,
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar).into_dust(soul_points).build_wrapped()
		}),
	)
}

pub(crate) fn create_random_color_spark(
	base_dna: Option<[u8; 32]>,
	account: &MockAccountId,
	color_pair: &(ColorType, ColorType),
	soul_points: SoulCount,
	progress_array: [u8; 11],
) -> (AvatarIdOf<Test>, WrappedAvatar) {
	create_random_avatar::<Test, _>(
		account,
		base_dna,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.into_color_spark(color_pair, soul_points, progress_array)
				.build_wrapped()
		}),
	)
}

pub(crate) fn is_leader_forged<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Forged(_, _))
}

pub(crate) fn is_leader_consumed<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Consumed(_))
}

pub(crate) fn is_forged<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Forged(_, _))
}

pub(crate) fn is_minted<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Minted(_))
}

pub(crate) fn is_consumed<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Consumed(_))
}
