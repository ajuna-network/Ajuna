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
use crate::*;

pub struct MinterV1<T: Config>(PhantomData<T>);

impl<T: Config> Minter<T> for MinterV1<T> {
	fn mint(
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError> {
		let season = Seasons::<T>::get(season_id).ok_or(Error::<T>::UnknownSeason)?;
		let is_batched = mint_option.count.is_batched();
		let mint_count = mint_option.count.as_mint_count();
		(0..mint_count)
			.map(|_| {
				let avatar_id = Pallet::<T>::random_hash(b"create_avatar", player);
				let dna = Self::random_dna(&avatar_id, &season, is_batched)?;
				let souls = (dna.iter().map(|x| *x as SoulCount).sum::<SoulCount>() % 100) + 1;
				let avatar =
					Avatar { season_id: *season_id, version: AvatarVersion::V1, dna, souls };
				Avatars::<T>::insert(avatar_id, (&player, avatar));
				Owners::<T>::try_append(&player, avatar_id)
					.map_err(|_| Error::<T>::MaxOwnershipReached)?;
				Ok(avatar_id)
			})
			.collect()
	}
}

impl<T: Config> MinterV1<T> {
	fn random_dna(
		hash: &T::Hash,
		season: &SeasonOf<T>,
		batched_mint: bool,
	) -> Result<Dna, DispatchError> {
		let dna = (0..season.max_components)
			.map(|i| {
				let (random_tier, random_variation) =
					Self::random_component(season, hash, i as usize * 2, batched_mint);
				((random_tier << 4) | random_variation) as u8
			})
			.collect::<Vec<_>>();
		Dna::try_from(dna).map_err(|_| Error::<T>::IncorrectDna.into())
	}

	fn random_component(
		season: &SeasonOf<T>,
		hash: &T::Hash,
		index: usize,
		batched_mint: bool,
	) -> (u8, u8) {
		let hash = hash.as_ref();
		let random_tier = {
			let random_prob = hash[index] % MAX_PERCENTAGE;
			let probs =
				if batched_mint { &season.batch_mint_probs } else { &season.single_mint_probs };
			let mut cumulative_sum = 0;
			let mut random_tier = &season.tiers[0];
			for i in 0..probs.len() {
				let new_cumulative_sum = cumulative_sum + probs[i];
				if random_prob >= cumulative_sum && random_prob < new_cumulative_sum {
					random_tier = &season.tiers[i];
					break
				}
				cumulative_sum = new_cumulative_sum;
			}
			random_tier
		};
		let random_variation = hash[index + 1] % season.max_variations;
		(random_tier.to_owned().into(), random_variation)
	}
}
