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

//! Autogenerated weights for pallet_ajuna_awesome_avatars
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-26, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `weight-calculation`, CPU: `DO-Regular`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/bajun-para
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet-ajuna-awesome-avatars
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./.maintain/frame-weight-template.hbs
// --output=./pallets/ajuna-awesome-avatars/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_ajuna_awesome_avatars.
pub trait WeightInfo {
	fn mint_free(n: u32, ) -> Weight;
	fn mint_normal(n: u32, ) -> Weight;
	fn forge(n: u32, ) -> Weight;
	fn transfer_avatar_normal(n: u32, ) -> Weight;
	fn transfer_avatar_organizer(n: u32, ) -> Weight;
	fn transfer_free_mints() -> Weight;
	fn set_price() -> Weight;
	fn remove_price() -> Weight;
	fn buy(n: u32, ) -> Weight;
	fn upgrade_storage() -> Weight;
	fn set_organizer() -> Weight;
	fn set_treasurer() -> Weight;
	fn set_season() -> Weight;
	fn update_global_config() -> Weight;
	fn issue_free_mints() -> Weight;
	fn withdraw_free_mints() -> Weight;
	fn set_free_mints() -> Weight;
}

/// Weights for pallet_ajuna_awesome_avatars using the Substrate node and recommended hardware.
pub struct AjunaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for AjunaWeight<T> {
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonStatus (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: Randomness RandomMaterial (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:1 w:1)
	// Storage: AwesomeAvatars SeasonStats (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:0 w:6)
	/// The range of component `n` is `[0, 94]`.
	fn mint_free(n: u32, ) -> Weight {
		// Minimum execution time: 214_739 nanoseconds.
		Weight::from_ref_time(472_210_934 as u64)
			// Standard Error: 197_637
			.saturating_add(Weight::from_ref_time(1_250_789 as u64).saturating_mul(n as u64))
			.saturating_add(T::DbWeight::get().reads(9 as u64))
			.saturating_add(T::DbWeight::get().writes(10 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonStatus (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: Randomness RandomMaterial (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:1 w:1)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars SeasonStats (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:0 w:6)
	/// The range of component `n` is `[0, 94]`.
	fn mint_normal(_n: u32, ) -> Weight {
		// Minimum execution time: 228_975 nanoseconds.
		Weight::from_ref_time(568_149_012 as u64)
			.saturating_add(T::DbWeight::get().reads(10 as u64))
			.saturating_add(T::DbWeight::get().writes(11 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonStatus (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:5 w:0)
	// Storage: AwesomeAvatars LockedAvatars (r:5 w:0)
	// Storage: AwesomeAvatars Avatars (r:5 w:5)
	// Storage: Randomness RandomMaterial (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:1 w:1)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars SeasonStats (r:1 w:1)
	/// The range of component `n` is `[5, 100]`.
	fn forge(n: u32, ) -> Weight {
		// Minimum execution time: 193_867 nanoseconds.
		Weight::from_ref_time(304_438_552 as u64)
			// Standard Error: 111_302
			.saturating_add(Weight::from_ref_time(1_992_673 as u64).saturating_mul(n as u64))
			.saturating_add(T::DbWeight::get().reads(24 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Avatars (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:2 w:2)
	// Storage: AwesomeAvatars Accounts (r:1 w:0)
	/// The range of component `n` is `[1, 100]`.
	fn transfer_avatar_normal(_n: u32, ) -> Weight {
		// Minimum execution time: 219_036 nanoseconds.
		Weight::from_ref_time(426_762_540 as u64)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Avatars (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:2 w:2)
	// Storage: AwesomeAvatars Accounts (r:1 w:0)
	/// The range of component `n` is `[1, 100]`.
	fn transfer_avatar_organizer(_n: u32, ) -> Weight {
		// Minimum execution time: 234_625 nanoseconds.
		Weight::from_ref_time(465_201_302 as u64)
			.saturating_add(T::DbWeight::get().reads(8 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:2 w:2)
	fn transfer_free_mints() -> Weight {
		// Minimum execution time: 148_095 nanoseconds.
		Weight::from_ref_time(150_038_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Avatars (r:1 w:0)
	// Storage: AwesomeAvatars LockedAvatars (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:0 w:1)
	fn set_price() -> Weight {
		// Minimum execution time: 147_991 nanoseconds.
		Weight::from_ref_time(179_570_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:1 w:0)
	fn remove_price() -> Weight {
		// Minimum execution time: 154_410 nanoseconds.
		Weight::from_ref_time(189_406_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:2 w:2)
	// Storage: AwesomeAvatars Accounts (r:2 w:2)
	/// The range of component `n` is `[1, 100]`.
	fn buy(n: u32, ) -> Weight {
		// Minimum execution time: 244_037 nanoseconds.
		Weight::from_ref_time(399_290_892 as u64)
			// Standard Error: 161_241
			.saturating_add(Weight::from_ref_time(1_587_372 as u64).saturating_mul(n as u64))
			.saturating_add(T::DbWeight::get().reads(12 as u64))
			.saturating_add(T::DbWeight::get().writes(9 as u64))
	}
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	fn upgrade_storage() -> Weight {
		// Minimum execution time: 180_705 nanoseconds.
		Weight::from_ref_time(194_385_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:0 w:1)
	fn set_organizer() -> Weight {
		// Minimum execution time: 57_600 nanoseconds.
		Weight::from_ref_time(65_666_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Treasurer (r:0 w:1)
	fn set_treasurer() -> Weight {
		// Minimum execution time: 28_293 nanoseconds.
		Weight::from_ref_time(65_475_000 as u64)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:1)
	fn set_season() -> Weight {
		// Minimum execution time: 97_784 nanoseconds.
		Weight::from_ref_time(109_318_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars GlobalConfigs (r:0 w:1)
	fn update_global_config() -> Weight {
		// Minimum execution time: 61_535 nanoseconds.
		Weight::from_ref_time(76_545_000 as u64)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	fn issue_free_mints() -> Weight {
		// Minimum execution time: 67_488 nanoseconds.
		Weight::from_ref_time(97_020_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	fn withdraw_free_mints() -> Weight {
		// Minimum execution time: 67_457 nanoseconds.
		Weight::from_ref_time(78_544_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	fn set_free_mints() -> Weight {
		// Minimum execution time: 82_640 nanoseconds.
		Weight::from_ref_time(88_717_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonStatus (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: Randomness RandomMaterial (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:1 w:1)
	// Storage: AwesomeAvatars SeasonStats (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:0 w:6)
	/// The range of component `n` is `[0, 94]`.
	fn mint_free(n: u32, ) -> Weight {
		// Minimum execution time: 214_739 nanoseconds.
		Weight::from_ref_time(472_210_934 as u64)
			// Standard Error: 197_637
			.saturating_add(Weight::from_ref_time(1_250_789 as u64).saturating_mul(n as u64))
			.saturating_add(RocksDbWeight::get().reads(9 as u64))
			.saturating_add(RocksDbWeight::get().writes(10 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonStatus (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: Randomness RandomMaterial (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:1 w:1)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars SeasonStats (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:0 w:6)
	/// The range of component `n` is `[0, 94]`.
	fn mint_normal(_n: u32, ) -> Weight {
		// Minimum execution time: 228_975 nanoseconds.
		Weight::from_ref_time(568_149_012 as u64)
			.saturating_add(RocksDbWeight::get().reads(10 as u64))
			.saturating_add(RocksDbWeight::get().writes(11 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: AwesomeAvatars CurrentSeasonStatus (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:5 w:0)
	// Storage: AwesomeAvatars LockedAvatars (r:5 w:0)
	// Storage: AwesomeAvatars Avatars (r:5 w:5)
	// Storage: Randomness RandomMaterial (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:1 w:1)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars SeasonStats (r:1 w:1)
	/// The range of component `n` is `[5, 100]`.
	fn forge(n: u32, ) -> Weight {
		// Minimum execution time: 193_867 nanoseconds.
		Weight::from_ref_time(304_438_552 as u64)
			// Standard Error: 111_302
			.saturating_add(Weight::from_ref_time(1_992_673 as u64).saturating_mul(n as u64))
			.saturating_add(RocksDbWeight::get().reads(24 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Avatars (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:2 w:2)
	// Storage: AwesomeAvatars Accounts (r:1 w:0)
	/// The range of component `n` is `[1, 100]`.
	fn transfer_avatar_normal(_n: u32, ) -> Weight {
		// Minimum execution time: 219_036 nanoseconds.
		Weight::from_ref_time(426_762_540 as u64)
			.saturating_add(RocksDbWeight::get().reads(8 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Avatars (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:2 w:2)
	// Storage: AwesomeAvatars Accounts (r:1 w:0)
	/// The range of component `n` is `[1, 100]`.
	fn transfer_avatar_organizer(_n: u32, ) -> Weight {
		// Minimum execution time: 234_625 nanoseconds.
		Weight::from_ref_time(465_201_302 as u64)
			.saturating_add(RocksDbWeight::get().reads(8 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:2 w:2)
	fn transfer_free_mints() -> Weight {
		// Minimum execution time: 148_095 nanoseconds.
		Weight::from_ref_time(150_038_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Avatars (r:1 w:0)
	// Storage: AwesomeAvatars LockedAvatars (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:0 w:1)
	fn set_price() -> Weight {
		// Minimum execution time: 147_991 nanoseconds.
		Weight::from_ref_time(179_570_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:1 w:0)
	fn remove_price() -> Weight {
		// Minimum execution time: 154_410 nanoseconds.
		Weight::from_ref_time(189_406_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: AwesomeAvatars Trade (r:1 w:1)
	// Storage: AwesomeAvatars Avatars (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:0)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	// Storage: AwesomeAvatars Owners (r:2 w:2)
	// Storage: AwesomeAvatars Accounts (r:2 w:2)
	/// The range of component `n` is `[1, 100]`.
	fn buy(n: u32, ) -> Weight {
		// Minimum execution time: 244_037 nanoseconds.
		Weight::from_ref_time(399_290_892 as u64)
			// Standard Error: 161_241
			.saturating_add(Weight::from_ref_time(1_587_372 as u64).saturating_mul(n as u64))
			.saturating_add(RocksDbWeight::get().reads(12 as u64))
			.saturating_add(RocksDbWeight::get().writes(9 as u64))
	}
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	// Storage: AwesomeAvatars GlobalConfigs (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: AwesomeAvatars CurrentSeasonId (r:1 w:0)
	// Storage: AwesomeAvatars Treasury (r:1 w:1)
	fn upgrade_storage() -> Weight {
		// Minimum execution time: 180_705 nanoseconds.
		Weight::from_ref_time(194_385_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:0 w:1)
	fn set_organizer() -> Weight {
		// Minimum execution time: 57_600 nanoseconds.
		Weight::from_ref_time(65_666_000 as u64)
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Treasurer (r:0 w:1)
	fn set_treasurer() -> Weight {
		// Minimum execution time: 28_293 nanoseconds.
		Weight::from_ref_time(65_475_000 as u64)
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Seasons (r:1 w:1)
	fn set_season() -> Weight {
		// Minimum execution time: 97_784 nanoseconds.
		Weight::from_ref_time(109_318_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars GlobalConfigs (r:0 w:1)
	fn update_global_config() -> Weight {
		// Minimum execution time: 61_535 nanoseconds.
		Weight::from_ref_time(76_545_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	fn issue_free_mints() -> Weight {
		// Minimum execution time: 67_488 nanoseconds.
		Weight::from_ref_time(97_020_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	fn withdraw_free_mints() -> Weight {
		// Minimum execution time: 67_457 nanoseconds.
		Weight::from_ref_time(78_544_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: AwesomeAvatars Organizer (r:1 w:0)
	// Storage: AwesomeAvatars Accounts (r:1 w:1)
	fn set_free_mints() -> Weight {
		// Minimum execution time: 82_640 nanoseconds.
		Weight::from_ref_time(88_717_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}
