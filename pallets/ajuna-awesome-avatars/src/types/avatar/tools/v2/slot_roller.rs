use crate::{
	types::{avatar::tools::v2::avatar_utils::HashProvider, PackType},
	Config,
};
use sp_std::marker::PhantomData;

/// Represents a ‰ value, which goes from 1 to 1000
pub type SlotPerMille = u16;
pub type Slot<T> = (T, SlotPerMille);
pub type ProbabilitySlots<T, const N: usize> = [Slot<T>; N];

pub(crate) struct SlotRoller<'a, T: Config>(pub PhantomData<&'a T>);

impl<'a, T> SlotRoller<'a, T>
where
	T: Config,
{
	/// Rolls number between 1 and 1000, representing a range of 0.1% increments in probability.
	pub(crate) fn roll_number<const HS: usize>(hash_provider: &mut HashProvider<T, HS>) -> u16 {
		let first_number = (hash_provider.get_hash_byte() as u16) << 8;
		let second_number = hash_provider.get_hash_byte() as u16;
		((first_number | second_number) % 1000) + 1
	}

	pub(crate) fn roll_on<S, const N: usize, const HS: usize>(
		slots: &ProbabilitySlots<S, N>,
		hash_provider: &mut HashProvider<T, HS>,
	) -> S
	where
		S: Clone + Default,
	{
		let mut item_rolled = S::default();
		let mut roll = Self::roll_number(hash_provider);

		for (slot_item, slot_probability) in slots {
			roll = roll.saturating_sub(*slot_probability);

			if roll == 0 {
				item_rolled = slot_item.clone();
				break
			}
		}

		item_rolled
	}

	/// Rolls and picks from one of the three slots used as arguments, based on the value of
	/// pack_type
	pub(crate) fn roll_on_pack_type<S, const N: usize, const HS: usize>(
		pack_type: PackType,
		on_material: &ProbabilitySlots<S, N>,
		on_equipment: &ProbabilitySlots<S, N>,
		on_special: &ProbabilitySlots<S, N>,
		hash_provider: &mut HashProvider<T, HS>,
	) -> S
	where
		S: Clone + Default,
	{
		let slots = match pack_type {
			PackType::Material => on_material,
			PackType::Equipment => on_equipment,
			PackType::Special => on_special,
		};

		Self::roll_on(slots, hash_provider)
	}
}

// TODO: Add probability verification tests
