use crate::*;
use sp_runtime::{traits::Zero, DispatchError, Saturating};
use sp_std::{collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};

pub(crate) struct AttributeMapperV1;

impl AttributeMapper for AttributeMapperV1 {
	fn min_tier(&self, target: &Avatar) -> u8 {
		target.dna.iter().map(|x| *x >> 4).min().unwrap_or_default()
	}

	fn last_variation(&self, target: &Avatar) -> u8 {
		target.dna.last().unwrap_or(&0) & 0b0000_1111
	}
}

pub(super) struct AvatarMinterV1<T: Config>(pub PhantomData<T>);

impl<T: Config> Minter<T> for AvatarMinterV1<T> {
	fn mint_avatar_set(
		&self,
		player: &T::AccountId,
		season_id: &SeasonId,
		season: &SeasonOf<T>,
		mint_option: &MintOption,
	) -> Result<Vec<MintOutput<T>>, DispatchError> {
		let is_batched = mint_option.count.is_batched();
		(0..mint_option.count.clone() as usize)
			.map(|_| {
				let avatar_id = Pallet::<T>::random_hash(b"create_avatar", player);
				let dna = self.random_dna(&avatar_id, season, is_batched)?;
				let souls = (dna.iter().map(|x| *x as SoulCount).sum::<SoulCount>() % 100) + 1;
				let avatar =
					Avatar { season_id: *season_id, version: AvatarVersion::V1, dna, souls };
				Ok((avatar_id, avatar))
			})
			.collect::<Result<Vec<MintOutput<T>>, _>>()
	}
}

impl<T: Config> AvatarMinterV1<T> {
	fn random_dna(
		&self,
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
			let mut random_tier = season.tiers[0].clone() as u8;
			for i in 0..probs.len() {
				let new_cumulative_sum = cumulative_sum + probs[i];
				if random_prob >= cumulative_sum && random_prob < new_cumulative_sum {
					random_tier = season.tiers[i].clone() as u8;
					break
				}
				cumulative_sum = new_cumulative_sum;
			}
			random_tier
		};
		let random_variation = hash[index + 1] % season.max_variations;
		(random_tier, random_variation)
	}
}

pub(super) struct AvatarForgerV1<T: Config>(pub PhantomData<T>);

impl<T: Config> Forger<T> for AvatarForgerV1<T> {
	fn forge_with(
		&self,
		player: &T::AccountId,
		_season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		self.can_be_forged(season, &input_leader, &input_sacrifices)?;

		let (leader_id, mut leader) = input_leader;

		let max_tier = season.max_tier() as u8;
		let current_block = <frame_system::Pallet<T>>::block_number();

		let (sacrifice_ids, sacrifice_avatars): (Vec<AvatarIdOf<T>>, Vec<Avatar>) =
			input_sacrifices.into_iter().unzip();

		let (mut unique_matched_indexes, matches, soul_count) = self.compare_all(
			&leader,
			sacrifice_avatars.as_slice(),
			season.max_variations,
			max_tier,
		)?;

		leader.souls += soul_count;

		let mut upgraded_components = 0;
		let prob = self.forge_probability(&leader, season, &current_block, matches);
		let rolls = sacrifice_avatars.len();
		let random_hash = Pallet::<T>::random_hash(b"forging avatar", player);

		for hash in random_hash.as_ref().iter().take(rolls) {
			let roll = hash % MAX_PERCENTAGE;
			if roll <= prob {
				if let Some(first_matched_index) = unique_matched_indexes.pop_first() {
					let nucleotide = leader.dna[first_matched_index];
					let current_tier_index = season
						.tiers
						.clone()
						.into_iter()
						.position(|tier| tier as u8 == nucleotide >> 4)
						.ok_or(Error::<T>::UnknownTier)?;

					let already_maxed_out = current_tier_index == (season.tiers.len() - 1);
					if !already_maxed_out {
						let next_tier = season.tiers[current_tier_index + 1].clone() as u8;
						let upgraded_nucleotide = (next_tier << 4) | (nucleotide & 0b0000_1111);
						leader.dna[first_matched_index] = upgraded_nucleotide;
						upgraded_components += 1;
					}
				}
			}
		}

		Ok((
			LeaderForgeOutput::Forged((leader_id, leader), upgraded_components),
			sacrifice_ids
				.into_iter()
				.map(|sacrifice_id| ForgeOutput::Consumed(sacrifice_id))
				.collect(),
		))
	}

	fn can_be_forged(
		&self,
		_season: &SeasonOf<T>,
		input_leader: &ForgeItem<T>,
		input_sacrifices: &[ForgeItem<T>],
	) -> DispatchResult {
		ensure!(
			input_sacrifices
				.iter()
				.all(|(_, avatar)| avatar.version == input_leader.1.version),
			Error::<T>::IncompatibleAvatarVersions
		);
		Ok(())
	}
}

impl<T: Config> AvatarForgerV1<T> {
	fn compare_all(
		&self,
		target: &Avatar,
		others: &[Avatar],
		max_variations: u8,
		max_tier: u8,
	) -> Result<(BTreeSet<usize>, u8, SoulCount), DispatchError> {
		let upgradable_indexes = self.upgradable_indexes_for_target(target)?;
		let leader_tier = AttributeMapperV1.min_tier(target);
		others.iter().try_fold(
			(BTreeSet::<usize>::new(), 0, SoulCount::zero()),
			|(mut matched_components, mut matches, mut souls), other| {
				let sacrifice_tier = AttributeMapperV1.min_tier(other);
				if sacrifice_tier >= leader_tier {
					let (is_match, matching_components) =
						self.compare(target, other, &upgradable_indexes, max_variations, max_tier);

					if is_match {
						matches += 1;
						matched_components.extend(matching_components.iter());
					}

					souls.saturating_accrue(other.souls)
				}
				Ok((matched_components, matches, souls))
			},
		)
	}

	fn upgradable_indexes_for_target(&self, target: &Avatar) -> Result<Vec<usize>, DispatchError> {
		let min_tier = AttributeMapperV1.min_tier(target);
		Ok(target
			.dna
			.iter()
			.enumerate()
			.filter(|(_, x)| (*x >> 4) == min_tier)
			.map(|(i, _)| i)
			.collect::<Vec<usize>>())
	}

	fn compare(
		&self,
		target: &Avatar,
		other: &Avatar,
		indexes: &[usize],
		max_variations: u8,
		max_tier: u8,
	) -> (bool, BTreeSet<usize>) {
		let compare_variation = |lhs: u8, rhs: u8| -> bool {
			let diff = if lhs > rhs { lhs - rhs } else { rhs - lhs };
			diff == 1 || diff == (max_variations - 1)
		};

		let (matching_indexes, matches, mirrors) =
			target.dna.clone().into_iter().zip(other.dna.clone()).enumerate().fold(
				(BTreeSet::new(), 0, 0),
				|(mut matching_indexes, mut matches, mut mirrors), (i, (lhs, rhs))| {
					let lhs_variation = lhs & 0b0000_1111;
					let rhs_variation = rhs & 0b0000_1111;
					if lhs_variation == rhs_variation {
						mirrors += 1;
					}

					if indexes.contains(&i) {
						let lhs_tier = lhs >> 4;
						let rhs_tier = rhs >> 4;
						let is_matching_tier = lhs_tier == rhs_tier;
						let is_maxed_tier = lhs_tier == max_tier;

						let is_similar_variation = compare_variation(lhs_variation, rhs_variation);

						if is_matching_tier && !is_maxed_tier && is_similar_variation {
							matching_indexes.insert(i);
							matches += 1;
						}
					}
					(matching_indexes, matches, mirrors)
				},
			);

		// 1 upgradable component requires 1 match + 4 mirrors
		// 2 upgradable component requires 2 match + 2 mirrors
		// 3 upgradable component requires 3 match + 0 mirrors
		let mirrors_required = (3_u8.saturating_sub(matches)) * 2;
		let is_match = matches >= 3 || (matches >= 1 && mirrors >= mirrors_required);
		(is_match, matching_indexes)
	}

	fn forge_probability(
		&self,
		target: &Avatar,
		season: &SeasonOf<T>,
		now: &T::BlockNumber,
		matches: u8,
	) -> u8 {
		let period_multiplier = self.forge_multiplier(target, season, now);
		// p = base_prob + (1 - base_prob) * (matches / max_sacrifices) * (1 / period_multiplier)
		season.base_prob +
			(((MAX_PERCENTAGE - season.base_prob) / season.max_sacrifices) * matches) /
				period_multiplier
	}

	fn forge_multiplier(&self, target: &Avatar, season: &SeasonOf<T>, now: &T::BlockNumber) -> u8 {
		let mut current_period = season.current_period(now);
		let mut last_variation = AttributeMapperV1.last_variation(target) as u16;

		current_period.saturating_inc();
		last_variation.saturating_inc();

		let max_variations = season.max_variations as u16;
		let is_in_period = if last_variation == max_variations {
			(current_period % max_variations).is_zero()
		} else {
			(current_period % max_variations) == last_variation
		};

		if (current_period == last_variation) || is_in_period {
			1
		} else {
			2 // TODO: move this to config
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	impl Avatar {
		pub(crate) fn season_id(mut self, season_id: SeasonId) -> Self {
			self.season_id = season_id;
			self
		}
		pub(crate) fn dna(mut self, dna: &[u8]) -> Self {
			self.dna = Dna::try_from(dna.to_vec()).unwrap();
			self
		}
	}

	#[test]
	fn forge_probability_works() {
		// | variation |  period |
		// + --------- + ------- +
		// |         1 |   1,  7 |
		// |         2 |   2,  8 |
		// |         3 |   3,  9 |
		// |         4 |   4, 10 |
		// |         5 |   5, 11 |
		// |         6 |   6, 12 |
		let per_period = 2;
		let periods = 6;
		let max_variations = 6;
		let max_sacrifices = 4;

		let season = Season::default()
			.per_period(per_period)
			.periods(periods)
			.max_variations(max_variations)
			.max_sacrifices(max_sacrifices)
			.base_prob(0);

		let avatar = Avatar::default().dna(&[1, 3, 3, 7, 0]);
		let forger = AvatarForgerV1::<Test>(PhantomData);

		// in period
		let now = 1;
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 1), 25);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 2), 50);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 3), 75);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 4), 100);

		// not in period
		let now = 2;
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 1), 12);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 2), 25);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 3), 37);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 4), 50);

		// increase base_prob to 10
		let season = season.base_prob(10);
		// in period
		let now = 1;
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 1), 32);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 2), 54);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 3), 76);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 4), 98);

		// not in period
		let now = 2;
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 1), 21);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 2), 32);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 3), 43);
		assert_eq!(forger.forge_probability(&avatar, &season, &now, 4), 54);
	}

	#[test]
	fn forge_multiplier_works() {
		// | variation |      period |
		// + --------- + ----------- +
		// |         1 | 1, 4, 7, 10 |
		// |         2 | 2, 5, 8, 11 |
		// |         3 | 3, 6, 9, 12 |
		let per_period = 4;
		let periods = 3;
		let max_variations = 3;

		let season = Season::default()
			.per_period(per_period)
			.periods(periods)
			.max_variations(max_variations);

		#[allow(clippy::erasing_op, clippy::identity_op)]
		for (range, dna, expected_period, expected_multiplier) in [
			// cycle 0, period 0, last_variation must be 0
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 0], 0, 1),
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 1], 0, 2),
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 2], 0, 2),
			// cycle 0, period 1, last_variation must be 1
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 0], 1, 2),
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 1], 1, 1),
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 2], 1, 2),
			// cycle 0, period 2, last_variation must be 2
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 0], 2, 2),
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 1], 2, 2),
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 2], 2, 1),
			// cycle 1, period 0, last_variation must be 0
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 0], 0, 1),
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 1], 0, 2),
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 2], 0, 2),
			// cycle 1, period 1, last_variation must be 1
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 0], 1, 2),
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 1], 1, 1),
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 2], 1, 2),
			// cycle 1, period 2, last_variation must be 2
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 0], 2, 2),
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 1], 2, 2),
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 2], 2, 1),
		] {
			for now in range {
				assert_eq!(season.current_period(&now), expected_period);

				let avatar = Avatar::default().dna(&dna);
				let forger = AvatarForgerV1::<Test>(PhantomData);
				assert_eq!(forger.forge_multiplier(&avatar, &season, &now), expected_multiplier);
			}
		}
	}

	#[test]
	fn compare_works() {
		let season = Season::default()
			.early_start(100)
			.start(200)
			.end(150_000)
			.max_tier_forges(100)
			.max_variations(6)
			.max_components(11)
			.min_sacrifices(1)
			.max_sacrifices(4)
			.tiers(&[RarityTier::Common, RarityTier::Rare, RarityTier::Legendary])
			.single_mint_probs(&[95, 5])
			.batch_mint_probs(&[80, 20])
			.base_prob(20)
			.per_period(20)
			.periods(12);

		let leader = Avatar::default()
			.dna(&[0x21, 0x05, 0x23, 0x24, 0x20, 0x22, 0x25, 0x23, 0x05, 0x04, 0x02]);
		let other = Avatar::default()
			.dna(&[0x04, 0x00, 0x00, 0x04, 0x02, 0x04, 0x02, 0x00, 0x05, 0x05, 0x04]);
		let forger = AvatarForgerV1::<Test>(PhantomData);

		assert_eq!(
			forger.compare(
				&leader,
				&other,
				&[1, 8, 9, 10],
				season.max_variations,
				season.max_tier() as u8,
			),
			(true, BTreeSet::from([1, 9]))
		);
	}

	#[test]
	fn forge_should_work_for_matches() {
		let tiers = &[RarityTier::Common, RarityTier::Legendary];
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[100])
			.max_components(5)
			.max_variations(3)
			.min_sacrifices(1)
			.max_sacrifices(2);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption {
						count: MintPackSize::Six,
						mint_type: MintType::Free,
						mint_version: AvatarVersion::V1,
						mint_pack: PackType::default(),
					}
				));

				// forge
				let owned_avatar_ids = Owners::<Test>::get(BOB);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_ids = &owned_avatar_ids[1..3];

				let original_leader: Avatar = Avatars::<Test>::get(leader_id).unwrap().1;
				let original_sacrifices = sacrifice_ids
					.iter()
					.map(|id| Avatars::<Test>::get(id).unwrap().1)
					.collect::<Vec<_>>();

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					sacrifice_ids.to_vec()
				));
				let forged_leader = Avatars::<Test>::get(leader_id).unwrap().1;

				// check the result of the compare method
				let forger = AvatarForgerV1::<Test>(PhantomData);
				let upgradable_indexes =
					forger.upgradable_indexes_for_target(&original_leader).unwrap();
				for (sacrifice, result) in original_sacrifices
					.iter()
					.zip([(true, BTreeSet::from([0, 2, 3])), (true, BTreeSet::from([0, 2, 3, 4]))])
				{
					assert_eq!(
						forger.compare(
							&original_leader,
							sacrifice,
							&upgradable_indexes,
							season.max_variations,
							season.max_tier() as u8,
						),
						result
					)
				}

				// check all sacrifices are burned
				for sacrifice_id in sacrifice_ids {
					assert!(!Avatars::<Test>::contains_key(sacrifice_id));
				}
				// check for souls accumulation
				assert_eq!(
					forged_leader.souls,
					original_leader.souls +
						original_sacrifices.iter().map(|x| x.souls).sum::<SoulCount>(),
				);

				// check for the upgraded DNA
				assert_ne!(original_leader.dna[0..=1], forged_leader.dna[0..=1]);
				assert_eq!(original_leader.dna.to_vec()[0] >> 4, RarityTier::Common as u8);
				assert_eq!(original_leader.dna.to_vec()[1] >> 4, RarityTier::Common as u8);
				assert_eq!(forged_leader.dna.to_vec()[0] >> 4, RarityTier::Legendary as u8);
				assert_eq!(forged_leader.dna.to_vec()[1] >> 4, RarityTier::Common as u8);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarsForged { avatar_ids: vec![(leader_id, 1)] },
				));

				// variations remain the same
				assert_eq!(
					original_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
					forged_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
				);
				// other components remain the same
				assert_eq!(
					original_leader.dna[2..season.max_components as usize],
					forged_leader.dna[2..season.max_components as usize]
				);
			});
	}

	#[test]
	fn forge_should_work_for_non_matches() {
		let tiers =
			&[RarityTier::Common, RarityTier::Uncommon, RarityTier::Rare, RarityTier::Legendary];
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[33, 33, 34])
			.max_components(10)
			.max_variations(12)
			.min_sacrifices(1)
			.max_sacrifices(5);

		ExtBuilder::default()
			.seasons(&[(1, season.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption {
						count: MintPackSize::Six,
						mint_type: MintType::Free,
						mint_version: AvatarVersion::V1,
						mint_pack: PackType::default(),
					}
				));

				// forge
				let owned_avatar_ids = Owners::<Test>::get(BOB);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_id = owned_avatar_ids[1];

				let original_leader: Avatar = Avatars::<Test>::get(leader_id).unwrap().1;
				let original_sacrifice = Avatars::<Test>::get(sacrifice_id).unwrap().1;

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					vec![sacrifice_id]
				));
				let forged_leader = Avatars::<Test>::get(leader_id).unwrap().1;

				// check the result of the compare method
				let forger = AvatarForgerV1::<Test>(PhantomData);
				let upgradable_indexes =
					forger.upgradable_indexes_for_target(&original_leader).unwrap();
				assert_eq!(
					forger.compare(
						&original_leader,
						&original_sacrifice,
						&upgradable_indexes,
						season.max_variations,
						season.max_tier() as u8,
					),
					(false, BTreeSet::from([2]))
				);
				// check all sacrifices are burned
				assert!(!Avatars::<Test>::contains_key(sacrifice_id));
				// check for souls accumulation
				assert_eq!(forged_leader.souls, original_leader.souls + original_sacrifice.souls);

				// check DNAs are the same
				assert_eq!(original_leader.dna, forged_leader.dna);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarsForged { avatar_ids: vec![(leader_id, 0)] },
				));
			});
	}
}
