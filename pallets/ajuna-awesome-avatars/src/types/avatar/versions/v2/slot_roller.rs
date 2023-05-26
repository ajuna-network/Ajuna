use crate::{
	types::{avatar::versions::v2::avatar_utils::HashProvider, PackType},
	Config,
};
use sp_std::marker::PhantomData;

/// Represents a ‰ value, which goes from 1 to 1000
pub type SlotPerMille = u16;
pub type Slot<T> = (T, SlotPerMille);
pub type ProbabilitySlots<T, const N: usize> = [Slot<T>; N];

pub(crate) struct SlotRoller<T: Config>(pub PhantomData<T>);

impl<T: Config> SlotRoller<T> {
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

#[cfg(test)]
mod test {
	use super::{super::types::*, *};
	use crate::{mock::*, types::ByteConvertible, Pallet};

	#[test]
	fn statistics_verification_test() {
		ExtBuilder::default().build().execute_with(|| {
			let hash = Pallet::<Test>::random_hash(b"statistics_test", &ALICE);
			let mut hash_provider: HashProvider<Test, 32> = HashProvider::new(&hash);

			let packs: [ProbabilitySlots<MaterialItemType, 2>; 3] = [
				[(MaterialItemType::Polymers, 500), (MaterialItemType::Electronics, 500)],
				[(MaterialItemType::Polymers, 300), (MaterialItemType::Electronics, 700)],
				[(MaterialItemType::Polymers, 900), (MaterialItemType::Electronics, 100)],
			];

			let mut probability_array = [[0_u32; 2]; 3];

			let loop_count = 1_000_000_u32;
			// We expect 10% deviation on the expected probabilities
			let prob_epsilon = (10 * loop_count) / 1_000;

			for (pack_index, pack_type) in
				[PackType::Material, PackType::Equipment, PackType::Special]
					.into_iter()
					.enumerate()
			{
				for i in 0..loop_count {
					if i % 1000 == 999 {
						let hash_text = format!("loop_{:#07X}", i);
						let hash = Pallet::<Test>::random_hash(hash_text.as_bytes(), &ALICE);
						hash_provider = HashProvider::new(&hash);
					}
					let rolled_entry = SlotRoller::<Test>::roll_on_pack_type(
						pack_type.clone(),
						&packs[0],
						&packs[1],
						&packs[2],
						&mut hash_provider,
					);
					let rolled_index = rolled_entry.as_byte() as usize - 1;

					probability_array[pack_index][rolled_index] += 1;
				}
			}

			for (i, entry) in probability_array.iter().enumerate() {
				for (j, item) in entry.iter().enumerate() {
					let entry_prob = packs[i][j].1 as u32;
					let entry_avg_value = (entry_prob * loop_count) / 1_000;

					// All rolls fall between the expected value + epsilon%
					assert!((entry_avg_value + prob_epsilon) > *item);
					assert!((entry_avg_value - prob_epsilon) < *item);
				}
			}
		});
	}
}
