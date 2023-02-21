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
use crate::Pallet as NftTransfer;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	set_organizer {
		let organizer = account::<T>("ALICE");
	}: _(RawOrigin::Root, organizer.clone())
	verify {
		assert_last_event::<T>(Event::OrganizerSet { organizer }.into())
	}

	set_locked_state {
		let organizer = account::<T>("ALICE");
		Organizer::<T>::put(&organizer);
	}: _(RawOrigin::Signed(organizer), PalletLockedState::Locked)
	verify {
		assert_last_event::<T>(Event::LockedStateSet { locked_state: PalletLockedState::Locked }.into())
	}

	impl_benchmark_test_suite!(
		NftTransfer, crate::mock::ExtBuilder::default().build(), crate::mock::Test
	);
}
