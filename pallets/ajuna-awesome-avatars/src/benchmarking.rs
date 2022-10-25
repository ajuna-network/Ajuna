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

#[allow(unused)]
use crate::{
	types::{RarityTier::*, *},
	Pallet as AAvatars,
};
use frame_benchmarking::{benchmarks, vec};
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedFrom;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn create_seasons<T: Config>(n: usize) -> Result<(), &'static str> {
	CurrentSeasonId::<T>::put(1);
	CurrentSeasonStatus::<T>::put(SeasonStatus {
		active: true,
		early: false,
		prematurely_ended: false,
	});
	for i in 0..n {
		Seasons::<T>::insert(
			(i + 1) as SeasonId,
			Season {
				name: [u8::MAX; 100].to_vec().try_into().unwrap(),
				description: [u8::MAX; 1_000].to_vec().try_into().unwrap(),
				early_start: T::BlockNumber::from((i * 10 + 1) as u32),
				start: T::BlockNumber::from((i * 10 + 5) as u32),
				end: T::BlockNumber::from((i * 10 + 10) as u32),
				max_tier_forges: u32::MAX,
				max_variations: u8::MAX,
				max_components: u8::MAX,
				tiers: vec![Common, Uncommon, Rare, Epic, Legendary, Mythical].try_into().unwrap(),
				p_single_mint: vec![70, 20, 5, 4, 1].try_into().unwrap(),
				p_batch_mint: vec![40, 30, 15, 10, 5].try_into().unwrap(),
			},
		);
	}
	frame_system::Pallet::<T>::set_block_number(
		AAvatars::<T>::seasons(AAvatars::<T>::current_season_id()).unwrap().start,
	);
	Ok(())
}

fn create_avatars<T: Config>(name: &'static str, n: usize) -> Result<(), &'static str> {
	create_seasons::<T>(3)?;

	let player = account::<T>(name);
	let season_id = 1;
	FreeMints::<T>::insert(&player, n as MintCount);
	for _ in 0..n {
		AAvatars::<T>::do_mint(
			&player,
			&MintOption { mint_type: MintType::Free, count: MintPackSize::One },
			season_id,
		)?
	}
	Ok(())
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	mint_free {
		let name = "player";
		let n in 0 .. AAvatars::<T>::global_configs().max_avatars_per_player;
		create_avatars::<T>(name, n as usize)?;

		let caller = account::<T>(name);
		let mint_option = MintOption { mint_type: MintType::Free, count: MintPackSize::Six };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let avatar_ids = AAvatars::<T>::owners(caller).to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids }.into())
	}

	mint_normal {
		let name = "player";
		let n in 0 .. AAvatars::<T>::global_configs().max_avatars_per_player;
		create_avatars::<T>(name, n as usize)?;

		let caller = account::<T>(name);
		let mint_option = MintOption { mint_type: MintType::Normal, count: MintPackSize::Six };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let avatar_ids = AAvatars::<T>::owners(caller).to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids }.into())
	}

	forge {
		let name = "player";
		let max_avatars = AAvatars::<T>::global_configs().max_avatars_per_player as usize;
		create_avatars::<T>(name, max_avatars)?;

		let player = account::<T>(name);
		let avatar_ids = AAvatars::<T>::owners(&player);
		let avatar_id = avatar_ids[0];
		let (_owner, original_avatar) = AAvatars::<T>::avatars(&avatar_id).unwrap();
	}: _(RawOrigin::Signed(player), avatar_id, avatar_ids[1..5].to_vec())
	verify {
		let (_owner, upgraded_avatar) = AAvatars::<T>::avatars(&avatar_id).unwrap();
		let original_tiers = original_avatar.dna.into_iter().map(|x| x >> 4);
		let upgraded_tiers = upgraded_avatar.dna.into_iter().map(|x| x >> 4);
		let upgraded_components = original_tiers.zip(upgraded_tiers).fold(
			0, |mut count, (lhs, rhs)| {
				if lhs != rhs {
					count+=1;
				}
				count
			}
		);
		assert_last_event::<T>(Event::AvatarForged { avatar_id, upgraded_components }.into())
	}

	transfer_free_mints {
		let from = account::<T>("from");
		let to = account::<T>("to");
		let free_mint_transfer_fee = AAvatars::<T>::global_configs().mint.free_mint_transfer_fee;
		let how_many = MintCount::MAX + free_mint_transfer_fee as MintCount;
	}: _(RawOrigin::Signed(from.clone()), to.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsTransferred { from, to, how_many }.into())
	}

	set_price {
		let name = "player";
		let max_avatars = AAvatars::<T>::global_configs().max_avatars_per_player as usize;
		create_avatars::<T>(name, max_avatars)?;
		let caller = account::<T>(name);
		let avatar_id = AAvatars::<T>::owners(&caller)[0];
		let price = BalanceOf::<T>::unique_saturated_from(u128::MAX);
	}: _(RawOrigin::Signed(caller), avatar_id, price)
	verify {
		assert_last_event::<T>(Event::AvatarPriceSet { avatar_id, price }.into())
	}

	set_organizer {
		let caller = account::<T>("caller");
		let organizer = account::<T>("organizer");
	}: _(RawOrigin::Signed(caller), organizer.clone())
	verify {
		assert_last_event::<T>(Event::OrganizerSet { organizer }.into())
	}

	set_treasurer {
		let caller = account::<T>("caller");
		let treasurer = account::<T>("treasurer");
	}: _(RawOrigin::Signed(caller), treasurer.clone())
	verify {
		assert_last_event::<T>(Event::TreasurerSet { treasurer }.into())
	}

	set_season {
		let caller = account::<T>("caller");
		let season_id = SeasonId::MAX;
		let season = Season {
			name: [u8::MAX; 100].to_vec().try_into().unwrap(),
			description: [u8::MAX; 1_000].to_vec().try_into().unwrap(),
			early_start: T::BlockNumber::from(u32::MAX - 2),
			start: T::BlockNumber::from(u32::MAX - 1),
			end: T::BlockNumber::from(u32::MAX),
			max_tier_forges: u32::MAX,
			max_variations: u8::MAX,
			max_components: u8::MAX,
			tiers: vec![Common, Uncommon, Rare, Epic, Legendary, Mythical].try_into().unwrap(),
			p_single_mint: vec![70, 20, 5, 4, 1].try_into().unwrap(),
			p_batch_mint: vec![40, 30, 15, 10, 5].try_into().unwrap(),
		};
	}: _(RawOrigin::Signed(caller), season_id, season.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedSeason { season_id, season }.into())
	}

	update_global_config {
		let caller = account::<T>("caller");
		let config = GlobalConfig {
			max_avatars_per_player: u32::MAX,
			mint: MintConfig {
				open: true,
				fees: MintFees {
					one: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					three: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					six: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				},
				cooldown: T::BlockNumber::from(u32::MAX),
				free_mint_fee_multiplier: MintCount::MAX,
				free_mint_transfer_fee: MintCount::MAX,
			},
			forge: ForgeConfig {
				open: true,
				min_sacrifices: u8::MAX,
				max_sacrifices: u8::MAX,
			},
			trade: TradeConfig {
				open: true,
				buy_fee: BalanceOf::<T>::unique_saturated_from(u128::MAX),
			}
		};
	}: _(RawOrigin::Signed(caller), config.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedGlobalConfig(config).into())
	}

	issue_free_mints {
		let caller = account::<T>("caller");
		let to = account::<T>("to");
		let how_many = MintCount::MAX;
	}: _(RawOrigin::Signed(caller), to.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsIssued { to, how_many }.into())
	}

	impl_benchmark_test_suite!(
		AAvatars, crate::mock::ExtBuilder::default().build(), crate::mock::Test
	);
}
