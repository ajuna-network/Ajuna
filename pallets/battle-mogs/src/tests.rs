use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency};

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

fn create_mogwai(owner: MockAccountId, index: u64) -> <Test as frame_system::Config>::Hash {
	BattleMogs::create_mogwai(Origin::signed(owner)).expect("Failed mogwai creation!");
	BattleMogs::mogwai_of_owner_by_index((owner, index))
}

#[cfg(test)]
mod set_price {
	use super::*;

	#[test]
	fn set_price_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB, 0);
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
			let mogwai_id = create_mogwai(BOB, 0);

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
			let mogwai_id = create_mogwai(BOB, 0);
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
			let mogwai_id = create_mogwai(BOB, 0);
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
			let mogwai_id = create_mogwai(BOB, 0);

			assert_noop!(
				BattleMogs::remove_price(Origin::signed(BOB), mogwai_id),
				Error::<Test>::MogwaiNotForSale
			);
		});
	}
}

#[cfg(test)]
mod create_mogwai {
	use super::*;

	#[test]
	fn create_mogwai_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let owner = ALICE;
			assert_ok!(BattleMogs::create_mogwai(Origin::signed(owner)));
			let mogwai_id = BattleMogs::mogwai_of_owner_by_index((owner, 0));

			assert_eq!(BattleMogs::mogwai(mogwai_id).id, mogwai_id);
			assert_eq!(BattleMogs::owner_of(mogwai_id), Some(owner));

			assert_eq!(BattleMogs::mogwai_by_index(0), mogwai_id);
			assert_eq!(BattleMogs::all_mogwais_count(), 1);
			assert_eq!(BattleMogs::all_mogwais_hash(mogwai_id), 0);

			assert_eq!(BattleMogs::mogwai_of_owner_by_index((owner, 0)), mogwai_id);
			assert_eq!(BattleMogs::owned_mogwais_count(owner), 1);
			assert_eq!(BattleMogs::owned_mogwais_hash(mogwai_id), 0);

			System::assert_last_event(Event::BattleMogs(crate::Event::MogwaiCreated(
				owner, mogwai_id,
			)));
		});
	}

	#[test]
	fn create_mogwai_cannot_go_over_limit() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let mogwai_limit = BattleMogs::config_value(ALICE, 1);

			for _ in 0..mogwai_limit {
				assert_ok!(BattleMogs::create_mogwai(Origin::signed(account)));
			}

			assert_noop!(
				BattleMogs::create_mogwai(Origin::signed(account)),
				Error::<Test>::MaxMogwaisInAccount
			);

			assert_ok!(BattleMogs::update_config(Origin::signed(ALICE), 1, Some(1)));

			let new_mogwai_limit = BattleMogs::config_value(ALICE, 1);

			for _ in mogwai_limit..new_mogwai_limit {
				assert_ok!(BattleMogs::create_mogwai(Origin::signed(account)));
			}

			assert_noop!(
				BattleMogs::create_mogwai(Origin::signed(account)),
				Error::<Test>::MaxMogwaisInAccount
			);
		});
	}
}

#[cfg(test)]
mod remove_mogwai {
	use super::*;

	#[test]
	fn remove_mogwai_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let mogwai_id = create_mogwai(account, 0);
			let default_hash = <Test as frame_system::Config>::Hash::default();

			assert_ok!(BattleMogs::remove_mogwai(Origin::signed(account), mogwai_id));

			assert_eq!(BattleMogs::mogwai(mogwai_id).id, default_hash);
			assert_eq!(BattleMogs::owner_of(mogwai_id), None);

			assert_eq!(BattleMogs::mogwai_by_index(0), default_hash);
			assert_eq!(BattleMogs::all_mogwais_count(), 0);
			assert_eq!(BattleMogs::all_mogwais_hash(mogwai_id), 0);

			assert_eq!(BattleMogs::mogwai_of_owner_by_index((account, 0)), default_hash);
			assert_eq!(BattleMogs::owned_mogwais_count(account), 0);
			assert_eq!(BattleMogs::owned_mogwais_hash(mogwai_id), 0);

			System::assert_last_event(Event::BattleMogs(crate::Event::MogwaiRemoved(
				account, mogwai_id,
			)));
		});
	}

	#[test]
	fn remove_mogwai_only_founder_can_remove() {
		let founder = BOB;
		ExtBuilder::new(Some(founder)).build().execute_with(|| {
			let account = ALICE;
			let mogwai_id = create_mogwai(account, 0);

			assert_noop!(
				BattleMogs::remove_mogwai(Origin::signed(account), mogwai_id),
				Error::<Test>::FounderAction
			);
		});
	}

	#[test]
	fn remove_mogwai_only_owner_can_remove() {
		let founder = BOB;
		ExtBuilder::new(Some(founder)).build().execute_with(|| {
			let account = ALICE;
			let mogwai_id = create_mogwai(account, 0);

			assert_noop!(
				BattleMogs::remove_mogwai(Origin::signed(founder), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}
}

#[cfg(test)]
mod transfer {
	use super::*;

	#[test]
	fn transfer_successfully() {
		let founder = ALICE;
		ExtBuilder::new(Some(founder)).build().execute_with(|| {
			let target = BOB;
			let mogwai_id = create_mogwai(founder, 0);
			let default_hash = <Test as frame_system::Config>::Hash::default();

			assert_ok!(BattleMogs::transfer(Origin::signed(founder), target, mogwai_id));

			assert_eq!(BattleMogs::owner_of(mogwai_id), Some(target));

			assert_eq!(BattleMogs::mogwai_of_owner_by_index((target, 0)), mogwai_id);
			assert_eq!(BattleMogs::mogwai_of_owner_by_index((founder, 0)), default_hash);
			assert_eq!(BattleMogs::owned_mogwais_count(target), 1);
			assert_eq!(BattleMogs::owned_mogwais_count(founder), 0);

			System::assert_last_event(Event::BattleMogs(crate::Event::MogwaiTransfered(
				founder, target, mogwai_id,
			)));
		});
	}

	#[test]
	fn transfer_only_founder_can_transfer() {
		let founder = BOB;
		ExtBuilder::new(Some(founder)).build().execute_with(|| {
			let target = ALICE;
			let sender = CHARLIE;
			let mogwai_id = create_mogwai(sender, 0);

			assert_noop!(
				BattleMogs::transfer(Origin::signed(sender), target, mogwai_id),
				Error::<Test>::FounderAction
			);
		});
	}

	#[test]
	fn transfer_cannot_transfer_above_limit() {
		let founder = ALICE;
		ExtBuilder::new(Some(founder)).build().execute_with(|| {
			let target = BOB;
			let mogwai_limit = BattleMogs::config_value(target, 1);

			for _ in 0..mogwai_limit {
				let _ = create_mogwai(target, 0);
			}

			let mogwai_id = create_mogwai(founder, 0);

			assert_noop!(
				BattleMogs::transfer(Origin::signed(founder), target, mogwai_id),
				Error::<Test>::MaxMogwaisInAccount
			);
		});
	}
}

#[cfg(test)]
mod hatch_mogwai {
	use super::*;
	use crate::{GameEventType, PhaseType};

	#[test]
	fn hatch_mogwai_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account, 0);

			let mogwai = BattleMogs::mogwai(mogwai_id);
			assert_eq!(mogwai.phase, PhaseType::Breeded);

			run_to_block(
				System::block_number() + GameEventType::time_till(GameEventType::Hatch) as u64,
			);

			assert_ok!(BattleMogs::hatch_mogwai(Origin::signed(account), mogwai_id));

			let mogwai = BattleMogs::mogwai(mogwai_id);
			assert_eq!(mogwai.phase, PhaseType::Hatched);
		});
	}

	#[test]
	fn hatch_mogwai_cannot_hatch_non_owned_mogwai() {
		ExtBuilder::default().build().execute_with(|| {
			let account = BOB;
			let other = CHARLIE;
			let mogwai_id = create_mogwai(account, 0);

			run_to_block(
				System::block_number() + GameEventType::time_till(GameEventType::Hatch) as u64,
			);

			assert_noop!(
				BattleMogs::hatch_mogwai(Origin::signed(other), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn hatch_mogwai_cannot_hatch_until_enough_time_has_passed() {
		ExtBuilder::default().build().execute_with(|| {
			ExtBuilder::default().build().execute_with(|| {
				let account = BOB;
				let mogwai_id = create_mogwai(account, 0);

				let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

				run_to_block(System::block_number() + time_till_hatch / 2);

				assert_noop!(
					BattleMogs::hatch_mogwai(Origin::signed(account), mogwai_id),
					Error::<Test>::MogwaiNoHatch
				);
			});
		});
	}
}

#[cfg(test)]
mod sacrifice {
	use super::*;

	#[test]
	fn sacrifice_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id = create_mogwai(account, 0);
			let default_hash = <Test as frame_system::Config>::Hash::default();

			assert_ok!(BattleMogs::sacrifice(Origin::signed(account), mogwai_id));

			assert_eq!(BattleMogs::mogwai(mogwai_id).id, default_hash);
			assert_eq!(BattleMogs::owner_of(mogwai_id), None);

			assert_eq!(BattleMogs::mogwai_by_index(0), default_hash);
			assert_eq!(BattleMogs::all_mogwais_count(), 0);
			assert_eq!(BattleMogs::all_mogwais_hash(mogwai_id), 0);

			assert_eq!(BattleMogs::mogwai_of_owner_by_index((account, 0)), default_hash);
			assert_eq!(BattleMogs::owned_mogwais_count(account), 0);
			assert_eq!(BattleMogs::owned_mogwais_hash(mogwai_id), 0);
		});
	}

	#[test]
	fn sacrifice_can_only_be_done_by_owner() {
		ExtBuilder::default().build().execute_with(|| {
			let account = CHARLIE;
			let other = BOB;
			let mogwai_id = create_mogwai(account, 0);

			assert_noop!(
				BattleMogs::sacrifice(Origin::signed(other), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}
}

#[cfg(test)]
mod sacrifice_into {
	use super::*;
	use crate::{Mogwais, RarityType};

	#[test]
	fn sacrifice_into_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account, 0);
			let mogwai_id_2 = create_mogwai(account, 1);
			let default_hash = <Test as frame_system::Config>::Hash::default();

			Mogwais::<Test>::mutate(mogwai_id_1, |mogwai| {
				mogwai.rarity = RarityType::Epic;
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |mogwai| {
				mogwai.rarity = RarityType::Epic;
			});

			assert_ok!(BattleMogs::sacrifice_into(
				Origin::signed(account),
				mogwai_id_1,
				mogwai_id_2
			));

			assert_eq!(BattleMogs::mogwai(mogwai_id_1).id, default_hash);
			assert_eq!(BattleMogs::owner_of(mogwai_id_1), None);

			assert_eq!(BattleMogs::mogwai_by_index(0), mogwai_id_2);
			assert_eq!(BattleMogs::all_mogwais_count(), 1);
			assert_eq!(BattleMogs::all_mogwais_hash(mogwai_id_1), 0);
			assert_eq!(BattleMogs::all_mogwais_hash(mogwai_id_2), 0);

			assert_eq!(BattleMogs::mogwai_of_owner_by_index((account, 0)), mogwai_id_2);
			assert_eq!(BattleMogs::owned_mogwais_count(account), 1);
			assert_eq!(BattleMogs::owned_mogwais_hash(mogwai_id_1), 0);
			assert_eq!(BattleMogs::all_mogwais_hash(mogwai_id_2), 0);
		});
	}

	#[test]
	fn sacrifice_into_not_allowed_with_mogwai_from_different_owners() {
		ExtBuilder::default().build().execute_with(|| {
			let account = CHARLIE;
			let other = BOB;
			let mogwai_id_1 = create_mogwai(account, 0);
			let mogwai_id_2 = create_mogwai(other, 0);

			Mogwais::<Test>::mutate(mogwai_id_1, |mogwai| {
				mogwai.rarity = RarityType::Epic;
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |mogwai| {
				mogwai.rarity = RarityType::Epic;
			});

			assert_noop!(
				BattleMogs::sacrifice_into(Origin::signed(account), mogwai_id_1, mogwai_id_2),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn sacrifice_into_sacrifice_mogwai_into_self_not_allowed() {
		ExtBuilder::default().build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account, 0);

			Mogwais::<Test>::mutate(mogwai_id_1, |mogwai| {
				mogwai.rarity = RarityType::Epic;
			});

			assert_noop!(
				BattleMogs::sacrifice_into(Origin::signed(account), mogwai_id_1, mogwai_id_1),
				Error::<Test>::MogwaiSame
			);
		});
	}
}

#[cfg(test)]
mod buy_mogwai {
	use super::*;

	#[test]
	fn buy_mogwai_successfully() {
		ExtBuilder::default().build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_id = create_mogwai(account, 0);
			let sell_price = 1;

			assert_ok!(BattleMogs::set_price(Origin::signed(account), mogwai_id, sell_price));

			assert_eq!(BattleMogs::owner_of(mogwai_id), Some(account));
			assert_eq!(BattleMogs::owned_mogwais_count(account), 1);
			assert_eq!(BattleMogs::owned_mogwais_count(buyer), 0);
			assert_eq!(BattleMogs::all_mogwais_count(), 1);

			assert_ok!(BattleMogs::buy_mogwai(Origin::signed(buyer), mogwai_id, sell_price));

			System::assert_last_event(Event::BattleMogs(crate::Event::MogwaiBought(
				buyer, account, mogwai_id, sell_price,
			)));

			assert_eq!(BattleMogs::owner_of(mogwai_id), Some(buyer));
			assert_eq!(BattleMogs::owned_mogwais_count(account), 0);
			assert_eq!(BattleMogs::owned_mogwais_count(buyer), 1);
			assert_eq!(BattleMogs::all_mogwais_count(), 1);
		});
	}

	#[test]
	fn buy_mogwai_cannot_buy_mogwai_not_on_sale() {
		ExtBuilder::default().build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_id = create_mogwai(account, 0);

			assert_noop!(
				BattleMogs::buy_mogwai(Origin::signed(buyer), mogwai_id, 1_000),
				Error::<Test>::MogwaiNotForSale
			);
		});
	}

	#[test]
	fn buy_mogwai_cannot_buy_mogwai_if_account_has_reached_mogwai_limit() {
		ExtBuilder::default().build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_limit = BattleMogs::config_value(buyer, 1);
			let mogwai_id = create_mogwai(account, 0);
			let sell_price = 1;

			assert_ok!(BattleMogs::set_price(Origin::signed(account), mogwai_id, sell_price));

			for _ in 0..mogwai_limit {
				let _ = create_mogwai(buyer, 0);
			}

			assert_noop!(
				BattleMogs::buy_mogwai(Origin::signed(buyer), mogwai_id, sell_price),
				Error::<Test>::MaxMogwaisInAccount
			);
		});
	}
}

#[cfg(test)]
mod morph_mogwai {
	use super::*;
}

#[cfg(test)]
mod breed_mogwai {
	use super::*;
}
