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

//! Autogenerated weights for `pallet_multisig`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-11-28, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `weight-calculation`, CPU: `DO-Regular`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/ajuna-para
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet-multisig
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./HEADER-AGPL
// --output=./runtime/ajuna/src/weights/pallet_multisig.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_multisig`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::WeightInfo for WeightInfo<T> {
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_threshold_1(z: u32, ) -> Weight {
		// Minimum execution time: 36_129 nanoseconds.
		Weight::from_ref_time(47_928_548 as u64)
			// Standard Error: 80
			.saturating_add(Weight::from_ref_time(1_072 as u64).saturating_mul(z as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_create(_s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 124_692 nanoseconds.
		Weight::from_ref_time(195_720_144 as u64)
			// Standard Error: 222
			.saturating_add(Weight::from_ref_time(2_021 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[3, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_approve(s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 96_061 nanoseconds.
		Weight::from_ref_time(95_501_178 as u64)
			// Standard Error: 212_785
			.saturating_add(Weight::from_ref_time(2_042_297 as u64).saturating_mul(s as u64))
			// Standard Error: 159
			.saturating_add(Weight::from_ref_time(4_455 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	/// The range of component `s` is `[2, 10]`.
	/// The range of component `z` is `[0, 10000]`.
	fn as_multi_complete(_s: u32, z: u32, ) -> Weight {
		// Minimum execution time: 125_358 nanoseconds.
		Weight::from_ref_time(198_861_729 as u64)
			// Standard Error: 152
			.saturating_add(Weight::from_ref_time(2_135 as u64).saturating_mul(z as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	/// The range of component `s` is `[2, 10]`.
	fn approve_as_multi_create(s: u32, ) -> Weight {
		// Minimum execution time: 113_840 nanoseconds.
		Weight::from_ref_time(113_101_066 as u64)
			// Standard Error: 211_381
			.saturating_add(Weight::from_ref_time(5_672_894 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 10]`.
	fn approve_as_multi_approve(s: u32, ) -> Weight {
		// Minimum execution time: 66_149 nanoseconds.
		Weight::from_ref_time(98_731_116 as u64)
			// Standard Error: 144_637
			.saturating_add(Weight::from_ref_time(2_057_564 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Multisig Multisigs (r:1 w:1)
	/// The range of component `s` is `[2, 10]`.
	fn cancel_as_multi(s: u32, ) -> Weight {
		// Minimum execution time: 107_867 nanoseconds.
		Weight::from_ref_time(159_709_915 as u64)
			// Standard Error: 167_174
			.saturating_add(Weight::from_ref_time(226_704 as u64).saturating_mul(s as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}
