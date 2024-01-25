use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::bounded_vec;

mod add_rule {
	use super::*;

	#[test]
	fn add_rule_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = PayoutRuleOf::<Test>::default();

			assert_ok!(<Affiliates as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule.clone()));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(crate::Event::RuleAdded {
				rule_id,
			}));

			assert_eq!(AffiliateRules::<Test>::get(rule_id), Some(rule));
		})
	}

	#[test]
	fn cannot_add_rule_to_already_marked_extrinsic() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = PayoutRuleOf::<Test>::default();
			assert_ok!(<Affiliates as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule));

			let rule_2 = PayoutRuleOf::<Test>::default();
			assert_noop!(
				<Affiliates as RuleMutator<
					AccountIdOf<Test>,
					<Test as Config>::AffiliateMaxLevel,
				>>::try_add_rule_for(rule_id, rule_2),
				Error::<Test>::ExtrinsicAlreadyHasRule
			);
		})
	}
}

mod clear_rule {
	use super::*;

	#[test]
	fn clear_rule_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = PayoutRuleOf::<Test>::default();

			assert_ok!(<Affiliates as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule.clone()));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(crate::Event::RuleAdded {
				rule_id,
			}));

			assert_eq!(AffiliateRules::<Test>::get(rule_id), Some(rule));

			<Affiliates as RuleMutator<AccountIdOf<Test>, <Test as Config>::AffiliateMaxLevel>>::clear_rule_for(rule_id);

			System::assert_last_event(mock::RuntimeEvent::Affiliates(crate::Event::RuleCleared {
				rule_id,
			}));

			assert_eq!(AffiliateRules::<Test>::get(rule_id), None);
		})
	}

	#[test]
	fn clear_rule_for_non_existent_rule() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;

			<Affiliates as RuleMutator<AccountIdOf<Test>, <Test as Config>::AffiliateMaxLevel>>::clear_rule_for(rule_id);

			System::assert_last_event(mock::RuntimeEvent::Affiliates(crate::Event::RuleCleared {
				rule_id,
			}));

			assert_eq!(AffiliateRules::<Test>::get(rule_id), None);
		})
	}
}

mod affiliate_to {
	use super::*;

	#[test]
	fn affiliate_to_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test>::insert(BOB, state);

			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&BOB, &ALICE
			));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test>::get(ALICE), Some(bounded_vec![BOB]));
		});
	}

	#[test]
	fn affiliate_to_should_work_with_chain() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test>::insert(BOB, state);
			Affiliators::<Test>::insert(ALICE, state);
			Affiliators::<Test>::insert(CHARLIE, state);
			Affiliators::<Test>::insert(DAVE, state);

			// First step on the chain BOB <- ALICE
			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&BOB, &ALICE
			));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test>::get(ALICE), Some(bounded_vec![BOB]));

			// Second step on the chain BOB <- ALICE <- CHARLIE
			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&ALICE, &CHARLIE
			));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(
				crate::Event::AccountAffiliated { account: CHARLIE, to: ALICE },
			));

			assert_eq!(Affiliators::<Test>::get(ALICE).affiliates, 1);
			assert_eq!(Affiliatees::<Test>::get(CHARLIE), Some(bounded_vec![ALICE, BOB]));

			// Third step on the chain BOB <- ALICE <- CHARLIE <- DAVE
			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&CHARLIE, &DAVE
			));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(
				crate::Event::AccountAffiliated { account: DAVE, to: CHARLIE },
			));

			assert_eq!(Affiliators::<Test>::get(CHARLIE).affiliates, 1);
			assert_eq!(Affiliatees::<Test>::get(DAVE), Some(bounded_vec![CHARLIE, ALICE]));

			// Fourth step on the chain BOB <- ALICE <- CHARLIE <- DAVE <- EDWARD
			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&DAVE, &EDWARD
			));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(
				crate::Event::AccountAffiliated { account: EDWARD, to: DAVE },
			));

			assert_eq!(Affiliators::<Test>::get(DAVE).affiliates, 1);
			assert_eq!(Affiliatees::<Test>::get(EDWARD), Some(bounded_vec![DAVE, CHARLIE]));
		});
	}

	#[test]
	fn affiliate_to_rejects_with_self_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &ALICE
				),
				Error::<Test>::CannotAffiliateSelf
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_if_account_is_affiliator() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test>::insert(BOB, state);
			Affiliators::<Test>::insert(ALICE, state);

			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&BOB, &ALICE
			));

			assert_noop!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &BOB
				),
				Error::<Test>::CannotAffiliateExistingAffiliator
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_affiliating_to_more_than_one_account() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test>::insert(BOB, state);
			Affiliators::<Test>::insert(CHARLIE, state);

			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&BOB, &ALICE
			));

			assert_noop!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&CHARLIE, &ALICE
				),
				Error::<Test>::CannotAffiliateAlreadyAffiliatedAccount
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_unaffiliable_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				),
				Error::<Test>::CannotAffiliateToAccount
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_blocked_account() {
		ExtBuilder::default().build().execute_with(|| {
			Affiliators::<Test>::mutate(BOB, |state| {
				state.status = AffiliatableStatus::Blocked;
			});

			assert_noop!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				),
				Error::<Test>::CannotAffiliateToAccount
			);
		});
	}
}

mod clear_affiliation {
	use super::*;

	#[test]
	fn clear_affiliation_should_work() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test>::insert(BOB, state);

			assert_ok!(<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
				&BOB, &ALICE
			));

			System::assert_last_event(mock::RuntimeEvent::Affiliates(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test>::get(ALICE), Some(bounded_vec![BOB]));

			assert_ok!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliators::<Test>::get(BOB).affiliates, 0);
			assert_eq!(Affiliatees::<Test>::get(ALICE), None);
		});
	}

	#[test]
	fn clear_affiliation_if_no_affiliation_exists() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			assert_eq!(Affiliatees::<Test>::get(ALICE), None);

			assert_ok!(
				<Affiliates as AffiliateMutator<AccountIdOf<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliatees::<Test>::get(ALICE), None);
		});
	}
}
