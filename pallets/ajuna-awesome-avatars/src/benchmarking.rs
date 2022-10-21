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
use crate::{types::*, Pallet as AAvatars};
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
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
