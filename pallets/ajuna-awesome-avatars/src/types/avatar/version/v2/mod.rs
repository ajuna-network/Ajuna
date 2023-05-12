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

mod types;

pub use types::*;

use super::*;
use crate::*;
use sp_std::mem::variant_count;

pub struct MinterV2<T: Config>(PhantomData<T>);

impl<T: Config> Minter<T> for MinterV2<T> {
	fn mint(
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError> {
		let MintOption { pack_size, pack, .. } = mint_option;
		let how_many = pack_size.as_mint_count();
		let mut avatars = Vec::with_capacity(how_many);

		for i in 0..how_many {
			let random_hash = WrappedHash::random_hash::<T>(b"mint_avatar_v2", player);

			// Select some bytes of random hash for later use.
			let (rand_int_1, rand_int_2, rand_short_1, rand_short_2, rand_short_3) =
				random_hash.select_bytes()?;

			// Randomly choose pet, armor and weapon slots.
			let (pet, armor_slot, weapon_slot) = (
				Pet::from(rand_short_1 % variant_count::<Pet>() as u16),
				ArmorSlot::from(rand_short_2 % variant_count::<ArmorSlot>() as u16),
				WeaponSlot::from(rand_short_3 % variant_count::<WeaponSlot>() as u16),
			);

			// Randomly choose quantity between 1 and `MAX_QUANTITY`.
			let quantity = (rand_short_3 % MAX_QUANTITY) + 1;

			// Randomly choose color and force.
			let color_byte = random_hash.cast_u8(24)?;
			let color_a = Color::from(WrappedHash::high_nibble(color_byte));
			let color_b = Color::from(WrappedHash::low_nibble(color_byte));
			let force = Force::from(random_hash.cast_u8(25)? % variant_count::<Force>() as u8);

			// Randomly choose souls between 1 and `MAX_INITIAL_SOUL`.
			let souls = ((random_hash.cast_u8(26)? % MAX_INITIAL_SOUL) + 1) as SoulCount;

			// Select bytes for use as progress bytes.
			let progress_bytes = random_hash.bytes::<T, 11>(22)?;

			let item_prob = rand_int_1;
			let sub_item_prob = rand_int_2;
			let mut item_prob_acc = 0;
			let mut sub_item_prob_acc = 0;
			for (item, prob) in pack.probs() {
				if prob.is_zero() {
					continue
				}

				item_prob_acc.saturating_accrue(prob);
				if item_prob < item_prob_acc {
					match item {
						Item::Pet => {
							avatars[i] = PetItem::build::<T>(
								pack,
								sub_item_prob,
								&mut sub_item_prob_acc,
								&pet,
								progress_bytes,
								*season_id,
								souls,
							)?;
						},
						Item::Material => {},
						Item::Essence => {},
						Item::Equippable => {},
						Item::Blueprint => {},
						Item::Special => {},
					}
				}
			}
		}

		todo!()
	}
}

impl_hash!((u8, cast_u8), (u16, cast_u16), (u32, cast_u32));
impl WrappedHash {
	fn select_bytes(&self) -> Result<(u32, u32, u16, u16, u16), DispatchError> {
		Ok((
			self.cast_u32(0)? % (PROB_SCALING_FACTOR),
			self.cast_u32(4)? % (PROB_SCALING_FACTOR),
			self.cast_u16(20)?,
			self.cast_u16(22)?,
			self.cast_u16(24)?,
		))
	}
	fn bytes<T: Config, const N: usize>(&self, from: usize) -> Result<[u8; N], DispatchError> {
		let x = (from..self.0.len())
			.map(|at| self.cast_u8(at))
			.collect::<Result<Vec<_>, DispatchError>>()?;
		x.try_into().map_err(|_| Error::<T>::IncorrectData.into())
	}
}

#[macro_export]
macro_rules! impl_hash {
	( $( ($ty:ty, $cast_ty:ident) ),* ) => {
		#[derive(Clone)]
		pub struct WrappedHash([u8; 32]);
		impl WrappedHash {
			fn random_hash<T: Config>(phrase: &[u8], who: &T::AccountId) -> Self {
				let random_hash = Pallet::<T>::random_hash(phrase, who);
				let random_hash = random_hash.as_ref();
				let mut hash = [0_u8; 32];
				hash.copy_from_slice(&random_hash[..32]);
				Self(hash)
			}
			fn high_nibble(value: u8) -> u8 {
				value >> 4
			}
			fn low_nibble(value: u8) -> u8 {
				value & 0x0F
			}
			$(
				fn $cast_ty(&self, at: usize) -> Result<$ty, DispatchError> {
					const N_BYTES: usize = <$ty>::BITS as usize / 8;
					ensure!(at <= 32 - N_BYTES, ArithmeticError::Overflow);
					let mut b = [0_u8; N_BYTES];
					b.copy_from_slice(&self.0[at..(at + N_BYTES)]);
					Ok(<$ty>::from_le_bytes(b))
				}
			)*
		}
	};
}
