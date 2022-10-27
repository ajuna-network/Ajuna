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

//! Autogenerated weights for `pallet_preimage`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-09-27, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/bajun-para
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet-preimage
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./HEADER-AGPL
// --output=./runtime/bajun/src/weights/pallet_preimage.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_preimage`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_preimage::WeightInfo for WeightInfo<T> {
	// Storage: PreImage PreimageFor (r:1 w:1)
	// Storage: PreImage StatusFor (r:1 w:1)
	fn note_preimage(s: u32, ) -> Weight {
		(60_270_000 as Weight)
			// Standard Error: 9_000
			.saturating_add((72_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PreImage PreimageFor (r:1 w:1)
	// Storage: PreImage StatusFor (r:1 w:0)
	fn note_requested_preimage(s: u32, ) -> Weight {
		(37_302_000 as Weight)
			// Standard Error: 5_000
			.saturating_add((10_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PreImage PreimageFor (r:1 w:1)
	// Storage: PreImage StatusFor (r:1 w:0)
	fn note_no_deposit_preimage(_s: u32, ) -> Weight {
		(36_001_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	// Storage: PreImage PreimageFor (r:0 w:1)
	fn unnote_preimage() -> Weight {
		(56_636_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	// Storage: PreImage PreimageFor (r:0 w:1)
	fn unnote_no_deposit_preimage() -> Weight {
		(31_139_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	fn request_preimage() -> Weight {
		(53_119_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	fn request_no_deposit_preimage() -> Weight {
		(29_966_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	fn request_unnoted_preimage() -> Weight {
		(28_875_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	fn request_requested_preimage() -> Weight {
		(9_669_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	// Storage: PreImage PreimageFor (r:0 w:1)
	fn unrequest_preimage() -> Weight {
		(30_808_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	// Storage: PreImage PreimageFor (r:0 w:1)
	fn unrequest_unnoted_preimage() -> Weight {
		(31_499_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: PreImage StatusFor (r:1 w:1)
	fn unrequest_multi_referenced_preimage() -> Weight {
		(9_649_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
