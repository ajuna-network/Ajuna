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

#![cfg(feature = "runtime-benchmarks")]
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

mod mock;

use frame_benchmarking::{benchmarks, vec};
use frame_support::{
	pallet_prelude::{DispatchError, DispatchResult},
	traits::{Currency, Get},
};
use frame_system::RawOrigin;
use pallet_ajuna_awesome_avatars::{types::*, Config as AvatarsConfig, Pallet as AAvatars, *};
use pallet_ajuna_nft_transfer::traits::NftHandler;
use sp_runtime::{
	traits::{Saturating, StaticLookup, UniqueSaturatedFrom, UniqueSaturatedInto, Zero},
	BoundedVec,
};

pub struct Pallet<T: Config>(pallet_ajuna_awesome_avatars::Pallet<T>);
pub trait Config: AvatarsConfig + pallet_nfts::Config + pallet_balances::Config {}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::Balance;
type CurrencyOf<T> = <T as AvatarsConfig>::Currency;
type CollectionIdOf<T> = <<T as AvatarsConfig>::NftHandler as NftHandler<
	AccountIdOf<T>,
	AvatarIdOf<T>,
	Avatar,
>>::CollectionId;

type NftCollectionConfigOf<T> =
	pallet_nfts::CollectionConfig<
		<<T as pallet_nfts::Config>::Currency as Currency<
			<T as frame_system::Config>::AccountId,
		>>::Balance,
		<T as frame_system::Config>::BlockNumber,
		<T as pallet_nfts::Config>::CollectionId,
	>;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn create_seasons<T: Config>(n: usize) -> Result<(), &'static str> {
	CurrentSeasonStatus::<T>::put(SeasonStatus {
		season_id: 0,
		early: false,
		active: true,
		early_ended: false,
		max_tier_avatars: 0,
	});
	for i in 0..n {
		CurrentSeasonStatus::<T>::mutate(|status| status.season_id = i as SeasonId + 1);
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
				tiers: vec![
					RarityTier::Common,
					RarityTier::Uncommon,
					RarityTier::Rare,
					RarityTier::Epic,
					RarityTier::Legendary,
					RarityTier::Mythical,
				]
				.try_into()
				.unwrap(),
				single_mint_probs: vec![70, 20, 5, 4, 1].try_into().unwrap(),
				batch_mint_probs: vec![40, 30, 15, 10, 5].try_into().unwrap(),
				base_prob: 0,
				per_period: T::BlockNumber::from(10_u32),
				periods: 12,
				trade_filters: BoundedVec::default(),
				fee: Fee {
					mint: MintFees {
						one: 550_000_000_000_u64.unique_saturated_into(), // 0.55 BAJU
						three: 500_000_000_000_u64.unique_saturated_into(), // 0.5 BAJU
						six: 450_000_000_000_u64.unique_saturated_into(), // 0.45 BAJU
					},
					transfer_avatar: 1_000_000_000_000_u64.unique_saturated_into(), // 1 BAJU
					buy_minimum: 1_000_000_000_u64.unique_saturated_into(),
					buy_percent: 1,
				},
			},
		);
	}
	frame_system::Pallet::<T>::set_block_number(
		Seasons::<T>::get(CurrentSeasonStatus::<T>::get().season_id).unwrap().start,
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

	GlobalConfigs::<T>::mutate(|config| {
		config.mint.open = true;
		config.forge.open = true;
		config.transfer.open = true;
		config.trade.open = true;
		config.nft_transfer.open = true;
	});
	for _ in 0..n {
		AAvatars::<T>::mint(
			RawOrigin::Signed(player.clone()).into(),
			MintOption {
				payment: MintPayment::Free,
				pack_size: MintPackSize::One,
				pack_type: PackType::Material,
				version: AvatarVersion::V1,
			},
		)?;
		Accounts::<T>::mutate(&player, |account| account.stats.mint.last = Zero::zero());
	}
	Ok(())
}

fn create_collection<T: Config>(organizer: T::AccountId) -> DispatchResult {
	let collection_deposit = <T as pallet_nfts::Config>::CollectionDeposit::get();
	<T as pallet_nfts::Config>::Currency::make_free_balance_be(
		&organizer,
		collection_deposit + <T as pallet_nfts::Config>::Currency::minimum_balance(),
	);

	let collection_setting = NftCollectionConfigOf::<T> {
		settings: pallet_nfts::CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: pallet_nfts::MintSettings::default(),
	};
	pallet_nfts::Pallet::<T>::create(
		RawOrigin::Signed(organizer.clone()).into(),
		T::Lookup::unlookup(organizer),
		collection_setting,
	)?;
	CollectionId::<T>::put(CollectionIdOf::<T>::from(0_u32));
	Ok(())
}

fn create_service_account<T: Config>() -> T::AccountId {
	let service_account = account::<T>("sa");
	ServiceAccount::<T>::put(&service_account);
	service_account
}

fn create_service_account_and_prepare_avatar<T: Config>(
	player: &T::AccountId,
	avatar_id: &AvatarIdOf<T>,
) -> Result<T::AccountId, DispatchError> {
	let service_account = create_service_account::<T>();
	let prepare_fee = GlobalConfigs::<T>::get().nft_transfer.prepare_fee;
	CurrencyOf::<T>::make_free_balance_be(player, prepare_fee);
	AAvatars::<T>::prepare_avatar(RawOrigin::Signed(player.clone()).into(), *avatar_id)?;
	Ok(service_account)
}

fn assert_last_event<T: Config>(avatars_event: Event<T>) {
	let event = <T as AvatarsConfig>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event.into());
}

benchmarks! {
	mint_free {
		let name = "player";
		let n in 0 .. (MaxAvatarsPerPlayer::get() - 6);
		create_avatars::<T>(name, n)?;

		let caller = account::<T>(name);
		Accounts::<T>::mutate(&caller, |account| account.free_mints = MintCount::MAX);

		let mint_option = MintOption { payment: MintPayment::Free, pack_size: MintPackSize::Six,
			pack_type: PackType::Material, version: AvatarVersion::V1 };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let n = n as usize;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(caller, season_id)[n..(n + 6)].to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids })
	}

	mint_normal {
		let name = "player";
		let n in 0 .. (MaxAvatarsPerPlayer::get() - 6);
		create_avatars::<T>(name, n)?;

		let caller = account::<T>(name);
		let season = Seasons::<T>::get(CurrentSeasonStatus::<T>::get().season_id).unwrap();
		let mint_fee = season.fee.mint.fee_for(&MintPackSize::Six);
		CurrencyOf::<T>::make_free_balance_be(&caller, mint_fee);

		let mint_option = MintOption { payment: MintPayment::Normal, pack_size: MintPackSize::Six,
			pack_type: PackType::Material, version: AvatarVersion::V1 };
	}: mint(RawOrigin::Signed(caller.clone()), mint_option)
	verify {
		let n = n as usize;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(caller, season_id)[n..(n + 6)].to_vec();
		assert_last_event::<T>(Event::AvatarsMinted { avatar_ids })
	}

	forge {
		let name = "player";
		let n in 5 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(name, n)?;

		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(&player, season_id);
		let avatar_id = avatar_ids[0];
		let (_owner, original_avatar) = Avatars::<T>::get(avatar_id).unwrap();
	}: _(RawOrigin::Signed(player), avatar_id, avatar_ids[1..5].to_vec())
	verify {
		let (_owner, upgraded_avatar) = Avatars::<T>::get(avatar_id).unwrap();
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
		assert_last_event::<T>(Event::AvatarsForged { avatar_ids: vec![(avatar_id, upgraded_components)] })
	}

	transfer_avatar_normal {
		let from = account::<T>("from");
		let to = account::<T>("to");
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>("from", MaxAvatarsPerPlayer::get())?;
		create_avatars::<T>("to", MaxAvatarsPerPlayer::get() - n)?;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&from, season_id)[n as usize - 1];

		let Season { fee, .. } = Seasons::<T>::get(season_id).unwrap();
		CurrencyOf::<T>::make_free_balance_be(&from, fee.transfer_avatar);
	}: transfer_avatar(RawOrigin::Signed(from.clone()), to.clone(), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTransferred { from, to, avatar_id })
	}

	transfer_avatar_organizer {
		let organizer = account::<T>("organizer");
		let to = account::<T>("to");
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>("organizer", MaxAvatarsPerPlayer::get())?;
		create_avatars::<T>("to", MaxAvatarsPerPlayer::get() - n)?;
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&organizer, season_id)[n as usize - 1];

		let Season { fee, .. } = Seasons::<T>::get(season_id).unwrap();
		CurrencyOf::<T>::make_free_balance_be(&organizer, fee.transfer_avatar);
	}: transfer_avatar(RawOrigin::Signed(organizer.clone()), to.clone(), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTransferred { from: organizer, to, avatar_id })
	}

	transfer_free_mints {
		let from = account::<T>("from");
		let to = account::<T>("to");
		let GlobalConfig { transfer, .. } = GlobalConfigs::<T>::get();
		let free_mint_transfer_fee = transfer.free_mint_transfer_fee;
		let how_many = MintCount::MAX - free_mint_transfer_fee as MintCount;
		Accounts::<T>::mutate(&from, |account| account.free_mints = MintCount::MAX);
	}: _(RawOrigin::Signed(from.clone()), to.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsTransferred { from, to, how_many })
	}

	set_price {
		let name = "player";
		create_avatars::<T>(name, MaxAvatarsPerPlayer::get())?;
		let caller = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&caller, season_id)[0];
		let price = BalanceOf::<T>::unique_saturated_from(u128::MAX);
	}: _(RawOrigin::Signed(caller), avatar_id, price)
	verify {
		assert_last_event::<T>(Event::AvatarPriceSet { avatar_id, price })
	}

	remove_price {
		let name = "player";
		create_avatars::<T>(name, MaxAvatarsPerPlayer::get())?;
		let caller = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&caller, season_id)[0];
		Trade::<T>::insert(season_id, avatar_id, BalanceOf::<T>::unique_saturated_from(u128::MAX));
	}: _(RawOrigin::Signed(caller), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarPriceUnset { avatar_id })
	}

	buy {
		let (buyer_name, seller_name) = ("buyer", "seller");
		let (buyer, seller) = (account::<T>(buyer_name), account::<T>(seller_name));
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(buyer_name, n- 1)?;
		create_avatars::<T>(seller_name, n)?;

		let sell_fee = BalanceOf::<T>::unique_saturated_from(u64::MAX / 2);
		let trade_fee = sell_fee / BalanceOf::<T>::unique_saturated_from(100_u8);
		CurrencyOf::<T>::make_free_balance_be(&buyer, sell_fee + trade_fee);
		CurrencyOf::<T>::make_free_balance_be(&seller, sell_fee);

		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&seller, season_id)[0];
		Trade::<T>::insert(season_id, avatar_id, sell_fee);
	}: _(RawOrigin::Signed(buyer.clone()), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarTraded { avatar_id, from: seller, to: buyer })
	}

	upgrade_storage {
		let player = account::<T>("player");
		let upgrade_fee = GlobalConfigs::<T>::get().account.storage_upgrade_fee;
		CurrencyOf::<T>::make_free_balance_be(&player, upgrade_fee);
	}: _(RawOrigin::Signed(player))
	verify {
		assert_last_event::<T>(Event::StorageTierUpgraded)
	}

	set_organizer {
		let organizer = account::<T>("organizer");
	}: _(RawOrigin::Root, organizer.clone())
	verify {
		assert_last_event::<T>(Event::<T>::OrganizerSet { organizer })
	}

	set_collection_id {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);
		let collection_id = CollectionIdOf::<T>::from(u32::MAX);
	}: _(RawOrigin::Signed(organizer), collection_id.clone())
	verify {
		assert_last_event::<T>(Event::CollectionIdSet { collection_id })
	}

	set_treasurer {
		let season_id = 369;
		let treasurer = account::<T>("treasurer");
	}: _(RawOrigin::Root, season_id, treasurer.clone())
	verify {
		assert_last_event::<T>(Event::TreasurerSet { season_id, treasurer })
	}

	claim_treasury {
		create_seasons::<T>(3)?;
		let season_id = 1;
		let treasurer = account::<T>("treasurer");
		let amount = 1_000_000_000_000_u64.unique_saturated_into();
		Treasurer::<T>::insert(season_id, treasurer.clone());
		Treasury::<T>::mutate(season_id, |balance| balance.saturating_accrue(amount));
		CurrencyOf::<T>::deposit_creating(&AAvatars::<T>::treasury_account_id(), amount);
		CurrencyOf::<T>::make_free_balance_be(&treasurer, CurrencyOf::<T>::minimum_balance());
	}: _(RawOrigin::Signed(treasurer.clone()), season_id)
	verify {
		assert_last_event::<T>(Event::TreasuryClaimed { season_id, treasurer, amount })
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
			tiers: vec![
				RarityTier::Common,
				RarityTier::Uncommon,
				RarityTier::Rare,
				RarityTier::Epic,
				RarityTier::Legendary,
				RarityTier::Mythical,
			]
			.try_into()
			.unwrap(),
			single_mint_probs: vec![70, 20, 5, 4, 1].try_into().unwrap(),
			batch_mint_probs: vec![40, 30, 15, 10, 5].try_into().unwrap(),
			base_prob: 99,
			per_period: T::BlockNumber::from(1_u32),
			periods: u16::MAX,
			trade_filters: BoundedVec::default(),
			fee: Fee {
				mint: MintFees {
					one: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					three: BalanceOf::<T>::unique_saturated_from(u128::MAX),
					six: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				},
				transfer_avatar: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				buy_minimum: BalanceOf::<T>::unique_saturated_from(u128::MAX),
				buy_percent: u8::MAX,
			},
		};
	}: _(RawOrigin::Signed(organizer), season_id, season.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedSeason { season_id, season })
	}

	update_global_config {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let config = GlobalConfig {
			mint: MintConfig {
				open: true,
				cooldown: T::BlockNumber::from(u32::MAX),
				free_mint_fee_multiplier: MintCount::MAX,
			},
			forge: ForgeConfig { open: true },
			transfer: TransferConfig {
				open:true,
				free_mint_transfer_fee: MintCount::MAX,
				min_free_mint_transfer: MintCount::MAX,
			},
			trade: TradeConfig { open: true },
			account: AccountConfig {
				storage_upgrade_fee: BalanceOf::<T>::unique_saturated_from(u128::MAX),
			},
			nft_transfer: NftTransferConfig {
				open: true,
				prepare_fee: BalanceOf::<T>::unique_saturated_from(u128::MAX),
			},
		};
	}: _(RawOrigin::Signed(organizer), config.clone())
	verify {
		assert_last_event::<T>(Event::UpdatedGlobalConfig(config))
	}

	set_free_mints {
		let organizer = account::<T>("organizer");
		Organizer::<T>::put(&organizer);

		let target = account::<T>("target");
		let how_many = MintCount::MAX;
	}: _(RawOrigin::Signed(organizer), target.clone(), how_many)
	verify {
		assert_last_event::<T>(Event::FreeMintsSet { target, how_many });
	}

	lock_avatar {
		let name = "player";
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(name, n)?;

		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(&player, season_id);
		let avatar_id = avatar_ids[avatar_ids.len() - 1];

		let organizer = account::<T>("organizer");
		create_collection::<T>(organizer)?;

		let service_account = create_service_account_and_prepare_avatar::<T>(&player, &avatar_id)?;
		let url = IpfsUrl::try_from(b"ipfs://test".to_vec()).unwrap();
		AAvatars::<T>::prepare_ipfs(RawOrigin::Signed(service_account).into(), avatar_id, url)?;

		let item_deposit = <T as pallet_nfts::Config>::ItemDeposit::get();
		let ed = <T as pallet_nfts::Config>::Currency::minimum_balance();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(&player, item_deposit + ed);
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarLocked { avatar_id })
	}

	unlock_avatar {
		let name = "player";
		let n in 1 .. MaxAvatarsPerPlayer::get();
		create_avatars::<T>(name, n)?;

		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(&player, season_id);
		let avatar_id = avatar_ids[avatar_ids.len() - 1];

		let organizer = account::<T>("organizer");
		create_collection::<T>(organizer)?;

		let service_account = create_service_account_and_prepare_avatar::<T>(&player, &avatar_id)?;
		let url = IpfsUrl::try_from(b"ipfs://test".to_vec()).unwrap();
		AAvatars::<T>::prepare_ipfs(RawOrigin::Signed(service_account).into(), avatar_id, url)?;

		let item_deposit = <T as pallet_nfts::Config>::ItemDeposit::get();
		let ed = <T as pallet_nfts::Config>::Currency::minimum_balance();
		<T as pallet_nfts::Config>::Currency::make_free_balance_be(&player, item_deposit + ed);
		AAvatars::<T>::lock_avatar(RawOrigin::Signed(player.clone()).into(), avatar_id)?;
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		assert_last_event::<T>(Event::AvatarUnlocked { avatar_id })
	}

	fix_variation {
		let name = "player";
		create_avatars::<T>(name, 1)?;

		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&player, season_id)[0];
		let (_owner, original_avatar) = Avatars::<T>::get(avatar_id).unwrap();
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		let (_owner, updated_avatar) = Avatars::<T>::get(avatar_id).unwrap();
		assert!(original_avatar.dna[1] & 0b0000_1111 != original_avatar.dna[2] & 0b0000_1111);
		assert!(updated_avatar.dna[1] & 0b0000_1111 == updated_avatar.dna[2] & 0b0000_1111);
	}

	set_service_account {
		let service_account = account::<T>("sa");
	}: _(RawOrigin::Root, service_account.clone())
	verify {
		assert_last_event::<T>(Event::<T>::ServiceAccountSet { service_account })
	}

	prepare_avatar {
		let name = "player";
		create_avatars::<T>(name, 1)?;
		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&player, season_id)[0];
		let _ = create_service_account::<T>();
		let prepare_fee = GlobalConfigs::<T>::get().nft_transfer.prepare_fee;
		CurrencyOf::<T>::make_free_balance_be(&player, prepare_fee);
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		assert_last_event::<T>(Event::<T>::PreparedAvatar { avatar_id })
	}

	unprepare_avatar {
		let name = "player";
		create_avatars::<T>(name, 1)?;
		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&player, season_id)[0];
		let _ = create_service_account_and_prepare_avatar::<T>(&player, &avatar_id)?;
	}: _(RawOrigin::Signed(player), avatar_id)
	verify {
		assert_last_event::<T>(Event::<T>::UnpreparedAvatar { avatar_id })
	}

	prepare_ipfs {
		let name = "player";
		create_avatars::<T>(name, 1)?;
		let player = account::<T>(name);
		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_id = Owners::<T>::get(&player, season_id)[0];
		let service_account = create_service_account_and_prepare_avatar::<T>(&player, &avatar_id)?;
		let url = IpfsUrl::try_from(b"ipfs://".to_vec()).unwrap();
	}: _(RawOrigin::Signed(service_account), avatar_id, url.clone())
	verify {
		assert_last_event::<T>(Event::<T>::PreparedIpfsUrl { url })
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
