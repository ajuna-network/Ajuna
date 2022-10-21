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

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
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
