// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

#[derive(Default)]
pub enum PetItem {
	#[default]
	Pet,
	Part,
	Egg,
}

impl From<PetItem> for u8 {
	fn from(value: PetItem) -> Self {
		match value {
			PetItem::Pet => 1,
			PetItem::Part => 2,
			PetItem::Egg => 3,
		}
	}
}

impl PetItem {
	const fn probs(pack: &MintPack) -> [(PetItem, Prob); 3] {
		match pack {
			MintPack::Material => [(PetItem::Pet, 0), (PetItem::Part, 980), (PetItem::Egg, 20)],
			MintPack::Equipment => [(PetItem::Pet, 20), (PetItem::Part, 800), (PetItem::Egg, 180)],
			MintPack::Special => [(PetItem::Pet, 10), (PetItem::Part, 390), (PetItem::Egg, 600)],
		}
	}

	pub(crate) fn build<T: Config>(
		pack: &MintPack,
		prob: Prob,
		prob_acc: &mut Prob,
		pet: &Pet,
		progress_bytes: [u8; 11],
		season_id: SeasonId,
		souls: SoulCount,
	) -> Result<Avatar, ()> {
		for (pet_item, prob) in Self::probs(pack) {
			if prob.is_zero() {
				continue
			}

			(*prob_acc).saturating_accrue(prob);
			if prob < *prob_acc {
				let avatar = match pet_item {
					PetItem::Pet => {
						Pet::create(pet, progress_bytes, season_id, souls);
					},
					PetItem::Part => {},
					PetItem::Egg => {},
				};
			}
		}
	}
}

#[derive(Default)]
pub enum Pet {
	#[default]
	TankyBullwog,
	FoxishDude,
	WeirdFerry,
	FireDino,
	BigHybrid,
	GiantWoodStick,
	CrazyDude,
}

impl From<u16> for Pet {
	fn from(value: u16) -> Self {
		match value {
			value if value == 0 => Self::TankyBullwog,
			value if value == 1 => Self::FoxishDude,
			value if value == 2 => Self::WeirdFerry,
			value if value == 3 => Self::FireDino,
			value if value == 4 => Self::BigHybrid,
			value if value == 5 => Self::GiantWoodStick,
			value if value == 6 => Self::CrazyDude,
			_ => Self::default(),
		}
	}
}

impl From<&Pet> for u8 {
	fn from(value: &Pet) -> Self {
		match value {
			Pet::TankyBullwog => 1,
			Pet::FoxishDude => 2,
			Pet::WeirdFerry => 3,
			Pet::FireDino => 4,
			Pet::BigHybrid => 5,
			Pet::GiantWoodStick => 6,
			Pet::CrazyDude => 7,
		}
	}
}

impl Pet {
	fn create(
		pet: &Pet,
		progress_bytes: [u8; 11],
		season_id: SeasonId,
		souls: SoulCount,
	) -> Result<Avatar, ()> {
		let rarity = &Rarity::Legendary;
		let mut dna_strand = DnaStrand::default();
		dna_strand
			.set_item_type(Item::Pet.into())?
			.set_sub_item_type(PetItem::Pet.into())?
			// TODO: see if there's a better way than HexType
			.set_class_1(0)?
			.set_class_2(pet.into())?
			// TODO: see if there's a better way than HexType
			.set_custom_1(0)? // pets are not stackable
			.set_rarity(rarity.into())?
			.set_quantity(1)?
			.set_custom_2(pet.into())?
			.set_spec_bytes(Default::default())?
			.set_progress_bytes(progress_bytes, rarity, PROGRESS_PROB_PERC)?;
		let dna = Dna::from(dna_strand);
		Ok(Avatar { season_id, version: AvatarVersion::V2, dna, souls })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;

	#[test]
	fn pet_create_works() {
		let random_bytes =
			hex::decode("E56C530CC0BD3BC9C47E74789B1119822C1ACE5B538B6665DBD209DF80F220E8")
				.unwrap();

		let mut progress_bytes = [0u8; 11];
		progress_bytes.copy_from_slice(&random_bytes[21..]);

		let pet = Pet::create(&Pet::FoxishDude, progress_bytes, 1, 99).unwrap();
		assert_eq!(
			hex::encode(pet.dna.clone().into_inner()),
			"1102050102000000000000000000000000000000005150555350635152526254"
		);
		assert_ok!(DnaStrand::from(pet.dna).custom_2(), 0b0000_0010);
	}
}
