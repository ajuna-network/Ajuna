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
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::{UniqueSaturatedFrom, Zero};

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn create_seasons<T: Config>(n: usize) -> Result<(), &'static str> {
	CurrentSeasonStatus::<T>::put(SeasonStatus {
		early: false,
		active: true,
		early_ended: false,
		max_tier_avatars: 0,
	});
	for i in 0..n {
		CurrentSeasonId::<T>::put(i as SeasonId + 1);
		Seasons::<T>::insert(
			(i + 1) as SeasonId,
			Season {
				name: [u8::MAX; 100].to_vec().try_into().unwrap(),
				description: [u8::MAX; 1_000].to_vec().try_into().unwrap(),
				early_start: T::BlockNumber::from((i * 10 + 1) as u32),
				start: T::BlockNumber::from((i * 10 + 5) as u32),
				end: T::BlockNumber::from((i * 10 + 10) as u32),
				max_tier_forges: u32::MAX,
				max_variations: 15,
				max_components: 16,
				min_sacrifices: 1,
				max_sacrifices: 4,
				tiers: vec![Common, Uncommon, Rare, Epic, Legendary, Mythical].try_into().unwrap(),
				single_mint_probs: vec![70, 20, 5, 4, 1].try_into().unwrap(),
				batch_mint_probs: vec![40, 30, 15, 10, 5].try_into().unwrap(),
				base_prob: 0,
				per_period: T::BlockNumber::from(10_u32),
				periods: 12,
			},
		);
	}
	frame_system::Pallet::<T>::set_block_number(
		AAvatars::<T>::seasons(AAvatars::<T>::current_season_id()).unwrap().start,
	);
	Ok(())
}

fn create_avatars<T: Config>(name: &'static str, n: u32) -> Result<(), &'static str> {
	create_seasons::<T>(3)?;

	let player = account::<T>(name);

	Accounts::<T>::mutate(&player, |account| {
		account.free_mints = n as MintCount;
		account.storage_tier = StorageTier::Max;
	});

	for _ in 0..n {
		AAvatars::<T>::do_mint(
			&player,
			&MintOption { mint_type: MintType::Free, count: MintPackSize::One },
		)?;
		Accounts::<T>::mutate(&player, |account| account.stats.mint.last = Zero::zero());
	}
	Ok(())
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	mint_free {
		let name = "player";
		let n in 0 .. (MaxAvatarsPerPlayer::get() - 6);
		create_avatars::<T>(name, n)?;

		let caller = account::<T>(name);
		Accounts::<T>::mutate(&caller, |account| account.free_mints =  MintCount::MAX);

		let mint_option = MintOption { mint_type: MintType::Free, count: MintPackSize::Six };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let n = n as usize;
		let avatar_ids = AAvatars::<T>::owners(caller)[n..(n + 6)].to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids }.into())
	}

	mint_normal {
		let name = "player";
		let n in 0 .. (MaxAvatarsPerPlayer::get() - 6);
		create_avatars::<T>(name, n)?;

		let caller = account::<T>(name);
		let mint_fee = AAvatars::<T>::global_configs().mint.fees.fee_for(&MintPackSize::Six);
		T::Currency::make_free_balance_be(&caller, mint_fee);

		let mint_option = MintOption { mint_type: MintType::Normal, count: MintPackSize::Six };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let n = n as usize;
		let avatar_ids = AAvatars::<T>::owners(caller)[n..(n + 6)].to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids }.into())
	}

	forge {
		let name = "player";
		let n in 5 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(name, n)?;

		let player = account::<T>(name);
		let avatar_ids = AAvatars::<T>::owners(&player);
		let avatar_id = avatar_ids[0];
		let (_owner, original_avatar) = AAvatars::<T>::avatars(avatar_id).unwrap();
	}: _(RawOrigin::Signed(player), avatar_id, avatar_ids[1..5].to_vec())
	verify {
		let (_owner, upgraded_avatar) = AAvatars::<T>::avatars(avatar_id).unwrap();
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

	transfer_avatar_normal {
		let from = account::<T>("from");
		let to = account::<T>("to");
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>("from", MaxAvatarsPerPlayer::get())?;
		create_avatars::<T>("to", MaxAvatarsPerPlayer::get() - n)?;
		let avatar_id = AAvatars::<T>::owners(&from)[n as usize - 1];

		let GlobalConfig { transfer, .. } = AAvatars::<T>::global_configs();
		T::Currency::make_free_balance_be(&from, transfer.avatar_transfer_fee);
	}: transfer_avatar(RawOrigin::Signed(from.clone()), to.clone(), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTransferred { from, to, avatar_id }.into())
	}

	transfer_avatar_organizer {
		let organizer = account::<T>("organizer");
		let to = account::<T>("to");
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>("organizer", MaxAvatarsPerPlayer::get())?;
		create_avatars::<T>("to", MaxAvatarsPerPlayer::get() - n)?;
		let avatar_id = AAvatars::<T>::owners(&organizer)[n as usize - 1];

		let GlobalConfig { transfer, .. } = AAvatars::<T>::global_configs();
		T::Currency::make_free_balance_be(&organizer, transfer.avatar_transfer_fee);
	}: transfer_avatar(RawOrigin::Signed(organizer.clone()), to.clone(), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTransferred { from: organizer, to, avatar_id }.into())
	}

	transfer_free_mints {
		let from = account::<T>("from");
		let to = account::<T>("to");
		let GlobalConfig { transfer, .. } = AAvatars::<T>::global_configs();
		let free_mint_transfer_fee = transfer.free_mint_transfer_fee;
		let how_many = MintCount::MAX - free_mint_transfer_fee as MintCount;
		Accounts::<T>::mutate(&from, |account| account.free_mints =  MintCount::MAX);
	}: _(RawOrigin::Signed(from.clone()), to.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsTransferred { from, to, how_many }.into())
	}

	set_price {
		let name = "player";
		create_avatars::<T>(name, MaxAvatarsPerPlayer::get())?;
		let caller = account::<T>(name);
		let avatar_id = AAvatars::<T>::owners(&caller)[0];
		let price = BalanceOf::<T>::unique_saturated_from(u128::MAX);
	}: _(RawOrigin::Signed(caller), avatar_id, price)
	verify {
		assert_last_event::<T>(Event::AvatarPriceSet { avatar_id, price }.into())
	}

	remove_price {
		let name = "player";
		create_avatars::<T>(name, MaxAvatarsPerPlayer::get())?;
		let caller = account::<T>(name);
		let avatar_id = AAvatars::<T>::owners(&caller)[0];
		Trade::<T>::insert(avatar_id, BalanceOf::<T>::unique_saturated_from(u128::MAX));
	}: _(RawOrigin::Signed(caller), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarPriceUnset { avatar_id }.into())
	}

	buy {
		let (buyer_name, seller_name) = ("buyer", "seller");
		let (buyer, seller) = (account::<T>(buyer_name), account::<T>(seller_name));
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(buyer_name, n- 1)?;
		create_avatars::<T>(seller_name, n)?;

		let min_fee = AAvatars::<T>::global_configs().trade.min_fee;
		let sell_fee = BalanceOf::<T>::unique_saturated_from(u64::MAX / 2);
		let trade_fee = sell_fee / BalanceOf::<T>::unique_saturated_from(100_u8);
		T::Currency::make_free_balance_be(&buyer, sell_fee + trade_fee);
		T::Currency::make_free_balance_be(&seller, sell_fee);

		let avatar_id = AAvatars::<T>::owners(&seller)[0];
		Trade::<T>::insert(avatar_id, sell_fee);
	}: _(RawOrigin::Signed(buyer.clone()), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTraded { avatar_id, from: seller, to: buyer }.into())
	}

	upgrade_storage {
		let player = account::<T>("player");
		let upgrade_fee = AAvatars::<T>::global_configs().account.storage_upgrade_fee;
		T::Currency::make_free_balance_be(&player, upgrade_fee);
	}: _(RawOrigin::Signed(player))
	verify {
		assert_last_event::<T>(Event::StorageTierUpgraded.into())
	}

	set_organizer {
		let organizer = account::<T>("organizer");
	}: _(RawOrigin::Root, organizer.clone())
	verify {
		assert_last_event::<T>(Event::OrganizerSet { organizer }.into())
	}

	set_collection_id {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);
		let collection_id = CollectionIdOf::<T>::from(u32::MAX);
	}: _(RawOrigin::Signed(organizer), collection_id.clone())
	verify {
		assert_last_event::<T>(Event::CollectionIdSet { collection_id }.into())
	}

	set_treasurer {
		let season_id = 369;
		let treasurer = account::<T>("treasurer");
	}: _(RawOrigin::Root, season_id, treasurer.clone())
	verify {
		assert_last_event::<T>(Event::TreasurerSet { season_id, treasurer }.into())
	}

	claim_treasury {
		create_seasons::<T>(3)?;
		let season_id = 1;
		let treasurer = account::<T>("treasurer");
		let amount = AAvatars::<T>::global_configs().transfer.avatar_transfer_fee;
		Treasurer::<T>::insert(season_id, treasurer.clone());
		AAvatars::<T>::deposit_into_treasury(&season_id, amount);
		T::Currency::make_free_balance_be(&treasurer, T::Currency::minimum_balance());
	}: _(RawOrigin::Signed(treasurer.clone()), season_id)
	verify {
		assert_last_event::<T>(Event::TreasuryClaimed { season_id, treasurer, amount }.into())
	}

	set_season {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let season_id = 1;
		let season = Season {
			name: [u8::MAX; 100].to_vec().try_into().unwrap(),
			description: [u8::MAX; 1_000].to_vec().try_into().unwrap(),
			early_start: T::BlockNumber::from(u32::MAX - 2),
			start: T::BlockNumber::from(u32::MAX - 1),
			end: T::BlockNumber::from(u32::MAX),
			max_tier_forges: u32::MAX,
			max_variations: 15,
			max_components: 16,
			min_sacrifices: SacrificeCount::MAX,
			max_sacrifices: SacrificeCount::MAX,
			tiers: vec![Common, Uncommon, Rare, Epic, Legendary, Mythical].try_into().unwrap(),
			single_mint_probs: vec![70, 20, 5, 4, 1].try_into().unwrap(),
			batch_mint_probs: vec![40, 30, 15, 10, 5].try_into().unwrap(),
			base_prob: u8::MAX,
			per_period: T::BlockNumber::from(1_u32),
			periods: u16::MAX,
		};
	}: _(RawOrigin::Signed(organizer), season_id, season.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedSeason { season_id, season }.into())
	}

	update_global_config {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let config = GlobalConfig {
			mint: MintConfig {
				open: true,
				fees: MintFees {
					one: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					three: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					six: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				},
				cooldown: T::BlockNumber::from(u32::MAX),
				free_mint_fee_multiplier: MintCount::MAX,
			},
			forge: ForgeConfig { open: true },
			transfer: TransferConfig {
				open:true,
				free_mint_transfer_fee: MintCount::MAX,
				min_free_mint_transfer: MintCount::MAX,
				avatar_transfer_fee: BalanceOf::<T>::unique_saturated_from(u128::MAX),
			},
			trade: TradeConfig {
				open: true,
				min_fee: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				percent_fee: u8::MAX,
			},
			account: AccountConfig {
				storage_upgrade_fee: BalanceOf::<T>::unique_saturated_from(u128::MAX),
			},
			nft_transfer: NftTransferConfig { open: true },
		};
	}: _(RawOrigin::Signed(organizer), config.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedGlobalConfig(config).into())
	}

	set_free_mints {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let target = account::<T>("target");
		let how_many = MintCount::MAX;
	}: _(RawOrigin::Signed(organizer), target.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsSet { target, how_many }.into())
	}

	fix_variation {
		let name = "player";
		create_avatars::<T>(name, 1)?;

		let player = account::<T>(name);
		let avatar_id = AAvatars::<T>::owners(&player)[0];
		let (_owner, original_avatar) = AAvatars::<T>::avatars(avatar_id).unwrap();
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		let (_owner, updated_avatar) = AAvatars::<T>::avatars(avatar_id).unwrap();
		assert!(original_avatar.dna[1] & 0b0000_1111 != original_avatar.dna[2] & 0b0000_1111);
		assert!(updated_avatar.dna[1] & 0b0000_1111 == updated_avatar.dna[2] & 0b0000_1111);
	}

	impl_benchmark_test_suite!(
		AAvatars, crate::mock::ExtBuilder::default().build(), crate::mock::Test
	);
}
