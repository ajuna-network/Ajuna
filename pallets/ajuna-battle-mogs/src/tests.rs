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

use crate::{
	mock, mock::*, AccountAchievement, AchievementState, Error, Event, GameEventType, MogwaiPrices,
	Mogwais, PhaseType, RarityType,
};
use frame_support::{assert_noop, assert_ok};

#[cfg(test)]
mod update_config {
	use super::*;

	#[test]
	fn config_is_updated_properly() {
		ExtBuilder.build().execute_with(|| {
			assert_eq!(BattleMogs::account_config(ALICE), None);

			assert_ok!(BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 1, Some(1)));
			System::assert_last_event(mock::RuntimeEvent::BattleMogs(
				crate::Event::AccountConfigChanged(ALICE, [0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
			));
			assert_ok!(BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 1, Some(2)));
			System::assert_last_event(mock::RuntimeEvent::BattleMogs(
				crate::Event::AccountConfigChanged(ALICE, [0, 2, 0, 0, 0, 0, 0, 0, 0, 0]),
			));
			assert_ok!(BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 1, Some(3)));
			System::assert_last_event(mock::RuntimeEvent::BattleMogs(
				crate::Event::AccountConfigChanged(ALICE, [0, 3, 0, 0, 0, 0, 0, 0, 0, 0]),
			));
		});
	}

	#[test]
	fn config_update_fails_validation() {
		ExtBuilder.build().execute_with(|| {
			assert_eq!(BattleMogs::account_config(ALICE), None);

			assert_noop!(
				BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 1, Some(6)),
				Error::<Test>::ConfigUpdateInvalid
			);

			assert_noop!(
				BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 3, Some(1)),
				Error::<Test>::ConfigUpdateInvalid
			);

			assert_noop!(
				BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 6, Some(4)),
				Error::<Test>::ConfigUpdateInvalid
			);
		});
	}
}

fn create_mogwai(owner: MockAccountId) -> MockMogwaiId {
	BattleMogs::create_mogwai(RuntimeOrigin::signed(owner)).expect("Failed mogwai creation!");
	if let crate::mock::RuntimeEvent::BattleMogs(Event::<Test>::MogwaiCreated(_, mogwai_id)) =
		last_event()
	{
		return mogwai_id
	}

	panic!("Expected MogwaiCreated event");
}

fn put_mogwai_on_sale(owner: MockAccountId, mogwai_id: MockMogwaiId, price: u64) {
	BattleMogs::set_price(RuntimeOrigin::signed(owner), mogwai_id, price)
		.expect("Failed setting mogwai price!");
}

#[cfg(test)]
mod set_price {
	use super::*;

	#[test]
	fn set_price_successfully() {
		ExtBuilder.build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);
			let sell_price = 1_000;

			assert_ok!(BattleMogs::set_price(RuntimeOrigin::signed(BOB), mogwai_id, sell_price));

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(crate::Event::ForSale(
				BOB, mogwai_id, sell_price,
			)));
		});
	}

	#[test]
	fn set_price_should_fail_for_non_owned_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);

			assert_noop!(
				BattleMogs::set_price(RuntimeOrigin::signed(ALICE), mogwai_id, 1_000),
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
		ExtBuilder.build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);
			let sell_price = 1_000;

			BattleMogs::set_price(RuntimeOrigin::signed(BOB), mogwai_id, sell_price)
				.expect("Failed to set price");

			assert_ok!(BattleMogs::remove_price(RuntimeOrigin::signed(BOB), mogwai_id));

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(
				crate::Event::RemovedFromSale(BOB, mogwai_id),
			));
		});
	}

	#[test]
	fn remove_price_should_fail_for_non_owned_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);
			let sell_price = 1_000;

			BattleMogs::set_price(RuntimeOrigin::signed(BOB), mogwai_id, sell_price)
				.expect("Failed to set price");

			assert_noop!(
				BattleMogs::remove_price(RuntimeOrigin::signed(ALICE), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn remove_price_should_fail_for_mogwai_not_on_sale() {
		ExtBuilder.build().execute_with(|| {
			let mogwai_id = create_mogwai(BOB);

			assert_noop!(
				BattleMogs::remove_price(RuntimeOrigin::signed(BOB), mogwai_id),
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
		ExtBuilder.build().execute_with(|| {
			let owner = ALICE;
			assert_ok!(BattleMogs::create_mogwai(RuntimeOrigin::signed(owner)));
			let mogwai_id =
				BattleMogs::owners(owner).into_iter().next().expect("Should get mogwai id");

			assert_eq!(BattleMogs::mogwai(mogwai_id).unwrap().id, mogwai_id);

			assert_eq!(BattleMogs::all_mogwais_count(), 1);

			assert_eq!(BattleMogs::owned_mogwais_count(owner), 1);

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(crate::Event::MogwaiCreated(
				owner, mogwai_id,
			)));
		});
	}

	#[test]
	fn create_mogwai_cannot_go_over_limit() {
		ExtBuilder.build().execute_with(|| {
			let account = ALICE;
			let mogwai_limit = BattleMogs::config_value(ALICE, 1);

			for _ in 0..mogwai_limit {
				assert_ok!(BattleMogs::create_mogwai(RuntimeOrigin::signed(account)));
			}

			assert_noop!(
				BattleMogs::create_mogwai(RuntimeOrigin::signed(account)),
				Error::<Test>::MaxMogwaisInAccount
			);

			assert_ok!(BattleMogs::update_config(RuntimeOrigin::signed(ALICE), 1, Some(1)));

			let new_mogwai_limit = BattleMogs::config_value(ALICE, 1);

			for _ in mogwai_limit..new_mogwai_limit {
				assert_ok!(BattleMogs::create_mogwai(RuntimeOrigin::signed(account)));
			}

			assert_noop!(
				BattleMogs::create_mogwai(RuntimeOrigin::signed(account)),
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
		ExtBuilder.build().execute_with(|| {
			let account = ALICE;
			let mogwai_id = create_mogwai(account);

			assert_ok!(BattleMogs::remove_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			assert_eq!(BattleMogs::mogwai(mogwai_id), None);

			assert_eq!(BattleMogs::all_mogwais_count(), 0);

			assert_eq!(BattleMogs::owned_mogwais_count(account), 0);

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(crate::Event::MogwaiRemoved(
				account, mogwai_id,
			)));
		});
	}

	#[test]
	fn remove_mogwai_only_owner_can_remove() {
		ExtBuilder.build().execute_with(|| {
			let account = ALICE;
			let other = CHARLIE;
			let mogwai_id = create_mogwai(account);

			assert_noop!(
				BattleMogs::remove_mogwai(RuntimeOrigin::signed(other), mogwai_id),
				Error::<Test>::FounderAction
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
		ExtBuilder.build().execute_with(|| {
			let target = BOB;
			let mogwai_id = create_mogwai(founder);

			assert_ok!(BattleMogs::transfer(RuntimeOrigin::signed(founder), target, mogwai_id));

			assert_eq!(BattleMogs::mogwai(mogwai_id).unwrap().owner, target);

			assert_eq!(BattleMogs::owned_mogwais_count(target), 1);
			assert_eq!(BattleMogs::owned_mogwais_count(founder), 0);

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(
				crate::Event::MogwaiTransfered(founder, target, mogwai_id),
			));
		});
	}

	#[test]
	fn transfer_only_founder_can_transfer() {
		ExtBuilder.build().execute_with(|| {
			let target = ALICE;
			let sender = CHARLIE;
			let mogwai_id = create_mogwai(sender);

			assert_noop!(
				BattleMogs::transfer(RuntimeOrigin::signed(sender), target, mogwai_id),
				Error::<Test>::FounderAction
			);
		});
	}

	#[test]
	fn transfer_cannot_transfer_above_limit() {
		let founder = ALICE;
		ExtBuilder.build().execute_with(|| {
			let target = BOB;
			let mogwai_limit = BattleMogs::config_value(target, 1);

			for _ in 0..mogwai_limit {
				let _ = create_mogwai(target);
			}

			let mogwai_id = create_mogwai(founder);

			assert_noop!(
				BattleMogs::transfer(RuntimeOrigin::signed(founder), target, mogwai_id),
				Error::<Test>::MaxMogwaisInAccount
			);
		});
	}

	#[test]
	fn transfer_removes_mogwai_sale() {
		ExtBuilder.build().execute_with(|| {
			let sender = ALICE;
			let target = BOB;
			let mogwai_id = create_mogwai(sender);

			put_mogwai_on_sale(sender, mogwai_id, 1000);

			assert_ok!(BattleMogs::transfer(RuntimeOrigin::signed(sender), target, mogwai_id));
			assert!(!MogwaiPrices::<Test>::contains_key(mogwai_id));
		});
	}
}

#[cfg(test)]
mod hatch_mogwai {
	use super::*;

	#[test]
	fn hatch_mogwai_successfully() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account);

			let mogwai = BattleMogs::mogwai(mogwai_id).expect("Should have found mogwai");
			assert_eq!(mogwai.phase, PhaseType::Bred);

			run_to_block(
				System::block_number() + GameEventType::time_till(GameEventType::Hatch) as u64,
			);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			let mogwai = BattleMogs::mogwai(mogwai_id).expect("Should have found mogwai");
			assert_eq!(mogwai.phase, PhaseType::Hatched);

			assert_eq!(
				BattleMogs::account_achievements(account, AccountAchievement::EggHatcher),
				Some(AchievementState::InProgress {
					current: 1,
					target: AccountAchievement::EggHatcher.target_for()
				})
			);
		});
	}

	#[test]
	fn hatch_mogwai_cannot_hatch_non_owned_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let other = CHARLIE;
			let mogwai_id = create_mogwai(account);

			run_to_block(
				System::block_number() + GameEventType::time_till(GameEventType::Hatch) as u64,
			);

			assert_noop!(
				BattleMogs::hatch_mogwai(RuntimeOrigin::signed(other), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn hatch_mogwai_cannot_hatch_until_enough_time_has_passed() {
		ExtBuilder.build().execute_with(|| {
			ExtBuilder.build().execute_with(|| {
				let account = BOB;
				let mogwai_id = create_mogwai(account);

				let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

				run_to_block(System::block_number() + time_till_hatch / 2);

				assert_noop!(
					BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id),
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
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			assert_ok!(BattleMogs::sacrifice(RuntimeOrigin::signed(account), mogwai_id));

			assert_eq!(BattleMogs::mogwai(mogwai_id), None);

			assert_eq!(BattleMogs::all_mogwais_count(), 0);

			assert_eq!(BattleMogs::owned_mogwais_count(account), 0);

			assert_eq!(
				BattleMogs::account_achievements(account, AccountAchievement::Sacrificer),
				Some(AchievementState::InProgress {
					current: 1,
					target: AccountAchievement::Sacrificer.target_for()
				})
			);
		});
	}

	#[test]
	fn sacrifice_not_allowed_with_non_hatched_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id = create_mogwai(account);

			assert_noop!(
				BattleMogs::sacrifice(RuntimeOrigin::signed(account), mogwai_id),
				Error::<Test>::MogwaiNoHatch
			);
		});
	}

	#[test]
	fn sacrifice_can_only_be_done_by_owner() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let other = BOB;
			let mogwai_id = create_mogwai(account);

			assert_noop!(
				BattleMogs::sacrifice(RuntimeOrigin::signed(other), mogwai_id),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn sacrifice_not_allowed_with_mogwai_on_sale() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id = create_mogwai(account);

			put_mogwai_on_sale(account, mogwai_id, 1000);

			assert_noop!(
				BattleMogs::sacrifice(RuntimeOrigin::signed(account), mogwai_id),
				Error::<Test>::MogwaiIsOnSale
			);
		});
	}

	#[test]
	fn sacrifice_with_balance_overflow() {
		let balance = MockBalance::MAX;
		let existential_deposit = <Test as pallet_balances::Config>::ExistentialDeposit::get();
		// We need to create an account that has almost the maximum amount of Balance possible
		// in order to test the resilience of the sacrifice extrinsic, in this case CHARLIE gets
		// almost all funds possible
		ExtBuilder
			.build_with_balances(
				[
					(ALICE, existential_deposit),
					(BOB, existential_deposit),
					(CHARLIE, balance - (2 * existential_deposit)),
				]
				.to_vec(),
			)
			.execute_with(|| {
				let account = CHARLIE;
				let mogwai_id = create_mogwai(account);

				// We artificially increase the mogwai intrinsic so that it then deposits to the
				// target account trying to cause an overflow when calling BattleMogs::sacrifice
				crate::Mogwais::<Test>::mutate(mogwai_id, |maybe_mogwai| {
					if let Some(ref mut mogwai) = maybe_mogwai {
						mogwai.intrinsic = balance;
					}
				});

				let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

				run_to_block(System::block_number() + time_till_hatch);

				assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id));

				assert_ok!(BattleMogs::sacrifice(RuntimeOrigin::signed(account), mogwai_id));
			});
	}
}

#[cfg(test)]
mod sacrifice_into {
	use super::*;

	#[test]
	fn sacrifice_into_successfully() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_1));
			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_2));

			// We need to up the rarity in order to be allowed to sacrifice
			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			assert_ok!(BattleMogs::sacrifice_into(
				RuntimeOrigin::signed(account),
				mogwai_id_1,
				mogwai_id_2
			));

			assert_eq!(BattleMogs::mogwai(mogwai_id_1), None);

			assert_eq!(BattleMogs::all_mogwais_count(), 1);
			assert_eq!(BattleMogs::owned_mogwais_count(account), 1);

			assert_eq!(
				BattleMogs::account_achievements(account, AccountAchievement::Sacrificer),
				Some(AchievementState::InProgress {
					current: 1,
					target: AccountAchievement::Sacrificer.target_for()
				})
			);
		});
	}

	#[test]
	fn sacrifice_into_not_allowed_with_any_common_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_1));
			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_2));

			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Common;
				}
			});

			assert_noop!(
				BattleMogs::sacrifice_into(
					RuntimeOrigin::signed(account),
					mogwai_id_1,
					mogwai_id_2
				),
				Error::<Test>::MogwaiBadRarity
			);
		});
	}

	#[test]
	fn sacrifice_into_not_allowed_with_non_hatched_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			// We need to up the rarity in order to be allowed to sacrifice
			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			assert_noop!(
				BattleMogs::sacrifice_into(
					RuntimeOrigin::signed(account),
					mogwai_id_1,
					mogwai_id_2
				),
				Error::<Test>::MogwaiNoHatch
			);
		});
	}

	#[test]
	fn sacrifice_into_not_allowed_with_mogwai_from_different_owners() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let other = BOB;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(other);

			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			assert_noop!(
				BattleMogs::sacrifice_into(
					RuntimeOrigin::signed(account),
					mogwai_id_1,
					mogwai_id_2
				),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn sacrifice_into_sacrifice_mogwai_into_self_not_allowed() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);

			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			assert_noop!(
				BattleMogs::sacrifice_into(
					RuntimeOrigin::signed(account),
					mogwai_id_1,
					mogwai_id_1
				),
				Error::<Test>::MogwaiSame
			);
		});
	}

	#[test]
	fn sacrifice_into_not_allowed_if_any_of_the_target_mogwais_on_sale() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_1));
			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_2));

			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
				}
			});

			put_mogwai_on_sale(account, mogwai_id_1, 1000);

			assert_noop!(
				BattleMogs::sacrifice_into(
					RuntimeOrigin::signed(account),
					mogwai_id_1,
					mogwai_id_2
				),
				Error::<Test>::MogwaiIsOnSale
			);

			assert_ok!(BattleMogs::remove_price(RuntimeOrigin::signed(account), mogwai_id_1));

			put_mogwai_on_sale(account, mogwai_id_2, 1000);

			assert_noop!(
				BattleMogs::sacrifice_into(
					RuntimeOrigin::signed(account),
					mogwai_id_1,
					mogwai_id_2
				),
				Error::<Test>::MogwaiIsOnSale
			);
		});
	}

	#[test]
	fn sacrifice_into_with_overflow() {
		ExtBuilder.build().execute_with(|| {
			let account = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_1));
			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_2));

			// We need to up the rarity in order to be allowed to sacrifice
			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
					mogwai.intrinsic = MockBalance::MAX;
				}
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.rarity = RarityType::Epic;
					mogwai.intrinsic = MockBalance::MAX;
				}
			});

			assert_ok!(BattleMogs::sacrifice_into(
				RuntimeOrigin::signed(account),
				mogwai_id_1,
				mogwai_id_2
			));
		});
	}
}

#[cfg(test)]
mod buy_mogwai {
	use super::*;

	#[test]
	fn buy_mogwai_successfully() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_id = create_mogwai(account);
			let sell_price = 1;

			assert_ok!(BattleMogs::set_price(
				RuntimeOrigin::signed(account),
				mogwai_id,
				sell_price
			));

			assert_eq!(BattleMogs::mogwai(mogwai_id).unwrap().owner, account);
			assert_eq!(BattleMogs::owned_mogwais_count(account), 1);
			assert_eq!(BattleMogs::owned_mogwais_count(buyer), 0);
			assert_eq!(BattleMogs::all_mogwais_count(), 1);

			assert_ok!(BattleMogs::buy_mogwai(RuntimeOrigin::signed(buyer), mogwai_id, sell_price));

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(crate::Event::MogwaiBought(
				buyer, account, mogwai_id, sell_price,
			)));

			assert_eq!(BattleMogs::mogwai(mogwai_id).unwrap().owner, buyer);
			assert_eq!(BattleMogs::owned_mogwais_count(account), 0);
			assert_eq!(BattleMogs::owned_mogwais_count(buyer), 1);
			assert_eq!(BattleMogs::all_mogwais_count(), 1);

			assert_eq!(
				BattleMogs::account_achievements(buyer, AccountAchievement::Buyer),
				Some(AchievementState::InProgress {
					current: 1,
					target: AccountAchievement::Buyer.target_for()
				})
			);

			assert_eq!(
				BattleMogs::account_achievements(account, AccountAchievement::Seller),
				Some(AchievementState::InProgress {
					current: 1,
					target: AccountAchievement::Seller.target_for()
				})
			);
		});
	}

	#[test]
	fn buy_mogwai_removes_sale_entry_in_storage() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_id = create_mogwai(account);
			let sell_price = 1;

			assert_ok!(BattleMogs::set_price(
				RuntimeOrigin::signed(account),
				mogwai_id,
				sell_price
			));
			assert_ok!(BattleMogs::buy_mogwai(RuntimeOrigin::signed(buyer), mogwai_id, sell_price));

			assert!(!MogwaiPrices::<Test>::contains_key(mogwai_id));
		});
	}

	#[test]
	fn buy_mogwai_cannot_buy_mogwai_not_on_sale() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_id = create_mogwai(account);

			assert_noop!(
				BattleMogs::buy_mogwai(RuntimeOrigin::signed(buyer), mogwai_id, 1_000),
				Error::<Test>::MogwaiNotForSale
			);
		});
	}

	#[test]
	fn buy_mogwai_cannot_buy_mogwai_if_account_has_reached_mogwai_limit() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let buyer = ALICE;
			let mogwai_limit = BattleMogs::config_value(buyer, 1);
			let mogwai_id = create_mogwai(account);
			let sell_price = 1;

			assert_ok!(BattleMogs::set_price(
				RuntimeOrigin::signed(account),
				mogwai_id,
				sell_price
			));

			for _ in 0..mogwai_limit {
				let _ = create_mogwai(buyer);
			}

			assert_noop!(
				BattleMogs::buy_mogwai(RuntimeOrigin::signed(buyer), mogwai_id, sell_price),
				Error::<Test>::MaxMogwaisInAccount
			);
		});
	}
}

#[cfg(test)]
mod morph_mogwai {

	use super::*;

	#[test]
	fn morph_mogwai_successfully() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			assert_ok!(BattleMogs::morph_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			System::assert_last_event(mock::RuntimeEvent::BattleMogs(crate::Event::MogwaiMorphed(
				mogwai_id,
			)));

			assert_eq!(
				BattleMogs::account_achievements(account, AccountAchievement::Morpheus),
				Some(AchievementState::InProgress {
					current: 1,
					target: AccountAchievement::Morpheus.target_for()
				})
			);
		});
	}

	#[test]
	fn morph_mogwai_fails_morphing_non_hatched_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account);

			assert_noop!(
				BattleMogs::morph_mogwai(RuntimeOrigin::signed(account), mogwai_id),
				Error::<Test>::MogwaiNoHatch
			);
		});
	}

	#[test]
	fn morph_mogwai_fails_morphing_non_owned_mogwai() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let other = ALICE;
			let mogwai_id = create_mogwai(account);

			assert!(BattleMogs::morph_mogwai(RuntimeOrigin::signed(other), mogwai_id).is_err());
		});
	}

	#[test]
	fn morph_mogwai_fails_morphing_mogwai_on_sale() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			put_mogwai_on_sale(account, mogwai_id, 1000);

			assert_noop!(
				BattleMogs::morph_mogwai(RuntimeOrigin::signed(account), mogwai_id),
				Error::<Test>::MogwaiIsOnSale
			);
		});
	}

	#[test]
	fn morph_mogwai_with_overflow() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account);

			Mogwais::<Test>::mutate(mogwai_id, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.intrinsic = MockBalance::MAX;
				}
			});

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id));

			assert_ok!(BattleMogs::morph_mogwai(RuntimeOrigin::signed(account), mogwai_id));
		});
	}
}

#[cfg(test)]
mod breed_mogwai {
	use super::*;

	#[test]
	fn breed_mogwai_successfully() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_1));
			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_2));

			assert_ok!(BattleMogs::breed_mogwai(
				RuntimeOrigin::signed(account),
				mogwai_id_1,
				mogwai_id_2
			));
		});
	}

	#[test]
	fn breed_mogwai_not_allowed_if_mogwai_not_hatched() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			assert_noop!(
				BattleMogs::breed_mogwai(RuntimeOrigin::signed(account), mogwai_id_1, mogwai_id_2),
				Error::<Test>::MogwaiNoHatch
			);
		});
	}

	#[test]
	fn breed_mogwai_not_allowed_if_mogwai_is_not_owned() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let other = ALICE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(other);

			assert_noop!(
				BattleMogs::breed_mogwai(RuntimeOrigin::signed(account), mogwai_id_2, mogwai_id_1),
				Error::<Test>::MogwaiNotOwned
			);
		});
	}

	#[test]
	fn breed_mogwai_not_allowed_if_mogwais_are_the_same() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id = create_mogwai(account);

			assert_noop!(
				BattleMogs::breed_mogwai(RuntimeOrigin::signed(account), mogwai_id, mogwai_id),
				Error::<Test>::MogwaiSame
			);
		});
	}

	#[test]
	fn breed_mogwai_not_allowed_if_account_reached_mogwai_limit() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_limit = BattleMogs::config_value(account, 1);
			let other = CHARLIE;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(other);

			for _ in 0..(mogwai_limit - 1) {
				let _ = create_mogwai(account);
			}

			assert_noop!(
				BattleMogs::breed_mogwai(RuntimeOrigin::signed(account), mogwai_id_1, mogwai_id_2),
				Error::<Test>::MaxMogwaisInAccount
			);
		});
	}

	#[test]
	fn breed_mogwai_with_overflow() {
		ExtBuilder.build().execute_with(|| {
			let account = BOB;
			let mogwai_id_1 = create_mogwai(account);
			let mogwai_id_2 = create_mogwai(account);

			let time_till_hatch = GameEventType::time_till(GameEventType::Hatch) as u64;

			run_to_block(System::block_number() + time_till_hatch);

			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_1));
			assert_ok!(BattleMogs::hatch_mogwai(RuntimeOrigin::signed(account), mogwai_id_2));

			Mogwais::<Test>::mutate(mogwai_id_1, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.intrinsic = MockBalance::MAX;
				}
			});

			Mogwais::<Test>::mutate(mogwai_id_2, |maybe_mogwai| {
				if let Some(ref mut mogwai) = maybe_mogwai {
					mogwai.intrinsic = MockBalance::MAX;
				}
			});

			assert_ok!(BattleMogs::breed_mogwai(
				RuntimeOrigin::signed(account),
				mogwai_id_1,
				mogwai_id_2
			));
		});
	}
}
