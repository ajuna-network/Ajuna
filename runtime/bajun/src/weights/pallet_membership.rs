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
//! Autogenerated weights for `pallet_membership`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-07-28, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/bajun-para
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet-membership
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./HEADER-AGPL
// --output=./runtime/bajun/src/weights/pallet_membership.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_membership`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_membership::WeightInfo for WeightInfo<T> {
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn add_member(m: u32, ) -> Weight {
		(32_452_000 as Weight)
			// Standard Error: 8_000
			.saturating_add((56_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: CouncilMembership Prime (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn remove_member(m: u32, ) -> Weight {
		(32_085_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((118_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: CouncilMembership Prime (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn swap_member(m: u32, ) -> Weight {
		(34_071_000 as Weight)
			// Standard Error: 8_000
			.saturating_add((125_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: CouncilMembership Prime (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn reset_member(m: u32, ) -> Weight {
		(34_723_000 as Weight)
			// Standard Error: 8_000
			.saturating_add((295_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: CouncilMembership Prime (r:1 w:1)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn change_key(m: u32, ) -> Weight {
		(40_608_000 as Weight)
			// Standard Error: 6_000
			.saturating_add((11_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: CouncilMembership Members (r:1 w:0)
	// Storage: CouncilMembership Prime (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn set_prime(m: u32, ) -> Weight {
		(9_539_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((52_000 as Weight).saturating_mul(m as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: CouncilMembership Prime (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn clear_prime(_m: u32, ) -> Weight {
		(4_404_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
