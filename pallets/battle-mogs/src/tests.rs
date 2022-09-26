use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	ExtBuilder::default().build().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(BattleMogs::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(BattleMogs::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	ExtBuilder::default().build().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(BattleMogs::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn test_dotmog_breeding() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(BattleMogs::all_mogwais_count(), 0);

		assert_ok!(BattleMogs::create_mogwai(Origin::signed(1)));
		assert_eq!(BattleMogs::all_mogwais_count(), 1);

		// test create
		assert_ok!(BattleMogs::create_mogwai(Origin::signed(1)));
		assert_eq!(BattleMogs::all_mogwais_count(), 2);

		let mogwai_hash_1 = BattleMogs::mogwai_by_index(0);
		let mogwai_hash_2 = BattleMogs::mogwai_by_index(1);
		let mogwai_1 = BattleMogs::mogwai(mogwai_hash_1);
		let mogwai_2 = BattleMogs::mogwai(mogwai_hash_2);

		assert_eq!(mogwai_1.gen, 1);
		assert_eq!(mogwai_2.gen, 2);

		assert_eq!(System::block_number(), 1);
		run_to_block(101);
		assert_eq!(System::block_number(), 101);

		// test morph
		assert_ok!(BattleMogs::hatch_mogwai(Origin::signed(1), mogwai_hash_1));
		assert_ok!(BattleMogs::hatch_mogwai(Origin::signed(1), mogwai_hash_2));

		// test morph
		//assert_ok!(BattleMogs::morph_mogwai(Origin::signed(1), mogwai_hash_1));

		// test breed
		//assert_ok!(BattleMogs::breed_mogwai(Origin::signed(1), mogwai_hash_1, mogwai_hash_2));
		//assert_eq!(BattleMogs::all_mogwais_count(), 3);

		// create real mogwai by breeding
		//let mogwai_hash_3 = BattleMogs::mogwai_by_index(2);
		//let mogwai_3 = BattleMogs::mogwai(mogwai_hash_3);
		//assert_eq!(mogwai_3.gen, 1);

		// run forward 100 blocks to make the egg hatch
		//assert_eq!(System::block_number(), 0);
		//run_to_block(101);
		//assert_eq!(System::block_number(), 101);
	});
}

#[cfg(test)]
mod update_config {
	use super::*;

	#[test]
	fn config_is_updated_properly() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(BattleMogs::account_config(ALICE), None);

			assert_ok!(BattleMogs::update_config(Origin::signed(ALICE), 1, Some(1)));
			System::assert_last_event(Event::BattleMogs(crate::Event::AccountConfigChanged(
				ALICE,
				[0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
			)));
			assert_ok!(BattleMogs::update_config(Origin::signed(ALICE), 1, Some(2)));
			System::assert_last_event(Event::BattleMogs(crate::Event::AccountConfigChanged(
				ALICE,
				[0, 2, 0, 0, 0, 0, 0, 0, 0, 0],
			)));
			assert_ok!(BattleMogs::update_config(Origin::signed(ALICE), 1, Some(3)));
			System::assert_last_event(Event::BattleMogs(crate::Event::AccountConfigChanged(
				ALICE,
				[0, 3, 0, 0, 0, 0, 0, 0, 0, 0],
			)));
		});
	}

	#[test]
	fn config_update_fails_validation() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(BattleMogs::account_config(ALICE), None);

			assert_noop!(
				BattleMogs::update_config(Origin::signed(ALICE), 1, Some(6)),
				Error::<Test>::ConfigUpdateInvalid
			);

			assert_noop!(
				BattleMogs::update_config(Origin::signed(ALICE), 3, Some(1)),
				Error::<Test>::ConfigUpdateInvalid
			);

			assert_noop!(
				BattleMogs::update_config(Origin::signed(ALICE), 6, Some(4)),
				Error::<Test>::ConfigUpdateInvalid
			);
		});
	}

	#[test]
	fn config_update_fails_with_no_founder() {
		ExtBuilder::new(None).build().execute_with(|| {
			assert_eq!(BattleMogs::account_config(ALICE), None);

			let result = std::panic::catch_unwind(|| {
				let _ = BattleMogs::update_config(Origin::signed(ALICE), 1, Some(1));
			});

			assert!(result.is_err());
		});
	}
}

fn create_mogwai(owner: MockAccountId) -> <Test as frame_system::Config>::Hash {
	BattleMogs::create_mogwai(Origin::signed(owner)).expect("Failed mogwai creation!");
	BattleMogs::mogwai_of_owner_by_index((owner, 0))
}

#[cfg(test)]
mod set_price {
	use super::*;

	#[test]
	fn set_price_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);
			let sell_price = 1_000;

			assert_ok!(BattleMogs::set_price(Origin::signed(BOB), mogwai_id, sell_price));

			System::assert_last_event(Event::BattleMogs(crate::Event::ForSale(
				BOB, mogwai_id, sell_price,
			)));
		});
	}

	#[test]
	fn set_price_should_fail_for_non_owned_mogwai() {
		ExtBuilder::default().build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);

			assert_noop!(
				BattleMogs::set_price(Origin::signed(ALICE), mogwai_id, 1_000),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}
}

#[cfg(test)]
mod remove_price {
	use super::*;

	#[test]
	fn remove_price_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);
			let sell_price = 1_000;

			BattleMogs::set_price(Origin::signed(BOB), mogwai_id, sell_price)
				.expect("Failed to set price");

			assert_ok!(BattleMogs::remove_price(Origin::signed(BOB), mogwai_id));

			System::assert_last_event(Event::BattleMogs(crate::Event::NotForSale(BOB, mogwai_id)));
		});
	}

	#[test]
	fn remove_price_should_fail_for_non_owned_mogwai() {
		ExtBuilder::default().build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);
			let sell_price = 1_000;

			BattleMogs::set_price(Origin::signed(BOB), mogwai_id, sell_price)
				.expect("Failed to set price");

			assert_noop!(
				BattleMogs::remove_price(Origin::signed(ALICE), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn remove_price_should_fail_for_mogwai_not_on_sale() {
		ExtBuilder::default().build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);

			assert_noop!(
				BattleMogs::remove_price(Origin::signed(BOB), mogwai_id),
				Error::<Test>::MogwaiNotForSale
			);
		});
	}
}
