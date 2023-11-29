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

//! Benchmarking setup for pallet-battle-mogs

#![cfg(feature = "runtime-benchmarks")]

use crate::{Config as BattleMogsConfig, *};
use frame_benchmarking::{benchmarks, whitelist_account, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::traits::UniqueSaturatedInto;

type CurrencyOf<T> = <T as BattleMogsConfig>::Currency;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let account = frame_benchmarking::account(name, Default::default(), Default::default());
	CurrencyOf::<T>::make_free_balance_be(&account, 999_999_999_u64.unique_saturated_into());
	account
}

fn force_hatch_mogwai<T: Config>(mogwai_id: &MogwaiIdOf<T>) {
	Mogwais::<T>::mutate(mogwai_id, |maybe_mogwai| {
		if let Some(ref mut mogwai) = maybe_mogwai {
			mogwai.phase = PhaseType::Hatched;
		}
	});
}

fn force_mogwai_rarity<T: Config>(mogwai_id: &MogwaiIdOf<T>, rarity: RarityType) {
	Mogwais::<T>::mutate(mogwai_id, |maybe_mogwai| {
		if let Some(ref mut mogwai) = maybe_mogwai {
			mogwai.rarity = rarity;
		}
	});
}

benchmarks! {
	set_organizer {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
	}: _(RawOrigin::Root, origin.clone())
	verify {
		assert_eq!(Pallet::<T>::organizer(), Some(origin))
	}

	update_config {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;
		let expected_config: [u8; 10] = [0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
	}: _(RawOrigin::Signed(origin.clone()), 1, Some(1))
	verify {
		assert_eq!(Pallet::<T>::account_config(origin), Some(expected_config));
	}

	set_price {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;

		let price = 1000_u32.into();
	}: _(RawOrigin::Signed(origin), mogwai_id, price)
	verify {
		assert_eq!(Pallet::<T>::mogwai_prices(mogwai_id), Some(price));
	}

	remove_price {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;
		let price = 1000_u32.into();
		Pallet::<T>::set_price(RawOrigin::Signed(origin.clone()).into(), mogwai_id, price)?;
	}: _(RawOrigin::Signed(origin), mogwai_id)
	verify {
		assert_eq!(Pallet::<T>::mogwai_prices(mogwai_id), None);
	}

	create_mogwai {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;
	}: _(RawOrigin::Signed(origin.clone()))
	verify {
		assert_eq!(Pallet::<T>::owned_mogwais_count(origin), 1_u64);
	}

	remove_mogwai {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;
	}: _(RawOrigin::Signed(origin), mogwai_id)
	verify {
		assert_eq!(Pallet::<T>::all_mogwais_count(), 0_u64);
	}

	transfer {
		let origin_1: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin_1, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin_1.clone())?;
		let origin_2: T::AccountId = account::<T>("origin_2");
		whitelist_account!(origin_2);

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin_1.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;
	}: _(RawOrigin::Signed(origin_1.clone()), origin_2.clone(), mogwai_id)
	verify {
		assert_eq!(Pallet::<T>::owned_mogwais_count(origin_1), 0_u64);
		assert_eq!(Pallet::<T>::owned_mogwais_count(origin_2), 1_u64);
	}

	hatch_mogwai {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;

		frame_system::Pallet::<T>::set_block_number(1000_u32.into());
	}: _(RawOrigin::Signed(origin.clone()), mogwai_id)
	verify {
		assert_eq!(Pallet::<T>::mogwai(mogwai_id).unwrap().phase, PhaseType::Hatched);
	}

	sacrifice {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;
		force_hatch_mogwai::<T>(&mogwai_id);
	}: _(RawOrigin::Signed(origin.clone()), mogwai_id)
	verify {
		assert_eq!(Pallet::<T>::mogwai(mogwai_id), None);
	}

	sacrifice_into {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mut mogwai_iter = Mogwais::<T>::iter_values();
		let mogwai_id_1 = mogwai_iter.next().unwrap().id;
		let mogwai_id_2 = mogwai_iter.next().unwrap().id;
		force_hatch_mogwai::<T>(&mogwai_id_1);
		force_hatch_mogwai::<T>(&mogwai_id_2);
		force_mogwai_rarity::<T>(&mogwai_id_1, RarityType::Epic);
		force_mogwai_rarity::<T>(&mogwai_id_2, RarityType::Epic);
	}: _(RawOrigin::Signed(origin.clone()), mogwai_id_1, mogwai_id_2)
	verify {
		assert_eq!(Pallet::<T>::mogwai(mogwai_id_1), None);
		assert!(Pallet::<T>::mogwai(mogwai_id_2).is_some());
	}

	buy_mogwai {
		let origin_1: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin_1, T::Currency::minimum_balance() * 20_000_000_u32.into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin_1.clone())?;
		let origin_2: T::AccountId = account::<T>("origin_2");
		whitelist_account!(origin_2);

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin_1.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;

		let price = 1000_u32.into();
		Pallet::<T>::set_price(RawOrigin::Signed(origin_1.clone()).into(), mogwai_id, price)?;
	}: _(RawOrigin::Signed(origin_2.clone()), mogwai_id, price)
	verify {
		assert_eq!(Pallet::<T>::owned_mogwais_count(origin_1), 0_u64);
		assert_eq!(Pallet::<T>::owned_mogwais_count(origin_2), 1_u64);
	}

	morph_mogwai {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 10_000_000_000_u64.unique_saturated_into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mogwai_id = Mogwais::<T>::iter_values().next().unwrap().id;
		force_hatch_mogwai::<T>(&mogwai_id);
	}: _(RawOrigin::Signed(origin.clone()), mogwai_id)

	breed_mogwai {
		let origin: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&origin, T::Currency::minimum_balance() * 10_000_000_000_u64.unique_saturated_into());
		Pallet::<T>::set_organizer(RawOrigin::Root.into(), origin.clone())?;

		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		Pallet::<T>::create_mogwai(RawOrigin::Signed(origin.clone()).into())?;
		let mut mogwai_iter = Mogwais::<T>::iter_values();
		let mogwai_id_1 = mogwai_iter.next().unwrap().id;
		let mogwai_id_2 = mogwai_iter.next().unwrap().id;
		force_hatch_mogwai::<T>(&mogwai_id_1);
		force_hatch_mogwai::<T>(&mogwai_id_2);
	}: _(RawOrigin::Signed(origin.clone()), mogwai_id_1, mogwai_id_2)
	verify {
		assert_eq!(Pallet::<T>::all_mogwais_count(), 3_u64);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
