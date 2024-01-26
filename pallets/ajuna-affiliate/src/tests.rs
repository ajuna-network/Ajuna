use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::bounded_vec;

mod add_rule {
	use super::*;

	#[test]
	fn add_rule_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = PayoutRuleOf::<Test, Instance1>::default();

			assert_ok!(<AffiliatesAlpha as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance1>>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule.clone()));

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule));
		})
	}

	#[test]
	fn cannot_add_rule_to_already_marked_extrinsic() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = PayoutRuleOf::<Test, Instance1>::default();
			assert_ok!(<AffiliatesAlpha as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance1>>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule));

			let rule_2 = PayoutRuleOf::<Test, Instance1>::default();
			assert_noop!(
				<AffiliatesAlpha as RuleMutator<
					AccountIdOf<Test>,
					<Test as Config<Instance1>>::AffiliateMaxLevel,
				>>::try_add_rule_for(rule_id, rule_2),
				Error::<Test, Instance1>::ExtrinsicAlreadyHasRule
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
			let rule = PayoutRuleOf::<Test, Instance1>::default();

			assert_ok!(<AffiliatesAlpha as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance1>>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule.clone()));

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule));

			<AffiliatesAlpha as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance1>>::AffiliateMaxLevel,
			>>::clear_rule_for(rule_id);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleCleared { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), None);
		})
	}

	#[test]
	fn clear_rule_for_non_existent_rule() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;

			<AffiliatesAlpha as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance1>>::AffiliateMaxLevel,
			>>::clear_rule_for(rule_id);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleCleared { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), None);
		})
	}
}

mod affiliate_to {
	use super::*;

	#[test]
	fn affiliate_to_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
		});
	}

	#[test]
	fn affiliate_to_should_work_with_chain() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(ALICE, state);
			Affiliators::<Test, Instance1>::insert(CHARLIE, state);
			Affiliators::<Test, Instance1>::insert(DAVE, state);

			// First step on the chain BOB <- ALICE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));

			// Second step on the chain BOB <- ALICE <- CHARLIE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: CHARLIE, to: ALICE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 1);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(CHARLIE),
				Some(bounded_vec![ALICE, BOB])
			);

			// Third step on the chain BOB <- ALICE <- CHARLIE <- DAVE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&CHARLIE, &DAVE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: DAVE, to: CHARLIE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(CHARLIE).affiliates, 1);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(DAVE),
				Some(bounded_vec![CHARLIE, ALICE])
			);

			// Fourth step on the chain BOB <- ALICE <- CHARLIE <- DAVE <- EDWARD
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&DAVE, &EDWARD
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: EDWARD, to: DAVE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 1);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(EDWARD),
				Some(bounded_vec![DAVE, CHARLIE])
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_self_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &ALICE
				),
				Error::<Test, Instance1>::CannotAffiliateSelf
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_if_account_is_affiliator() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(ALICE, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &BOB
				),
				Error::<Test, Instance1>::CannotAffiliateExistingAffiliator
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_affiliating_to_more_than_one_account() {
		ExtBuilder::default().build().execute_with(|| {
			let state = AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(CHARLIE, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&CHARLIE, &ALICE
				),
				Error::<Test, Instance1>::CannotAffiliateAlreadyAffiliatedAccount
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_unaffiliable_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				),
				Error::<Test, Instance1>::CannotAffiliateToAccount
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_blocked_account() {
		ExtBuilder::default().build().execute_with(|| {
			Affiliators::<Test, Instance1>::mutate(BOB, |state| {
				state.status = AffiliatableStatus::Blocked;
			});

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				),
				Error::<Test, Instance1>::CannotAffiliateToAccount
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
			Affiliators::<Test, Instance1>::insert(BOB, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);
		});
	}

	#[test]
	fn clear_affiliation_returns_ok_if_no_affiliation_exists() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);
		});
	}
}

mod multi_instance_tests {
	use super::*;

	#[test]
	fn affiliates_in_one_instance_dont_affect_other_instance() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let state_alpha =
				AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state_alpha);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			// Instance1 state contains the affiliated state
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			// Instance2 state contains no information as expected
			assert_eq!(Affiliators::<Test, Instance2>::get(BOB).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance2>::get(ALICE), None);

			let state_beta =
				AffiliatorState { status: AffiliatableStatus::Affiliatable, affiliates: 0 };
			Affiliators::<Test, Instance2>::insert(ALICE, state_beta);

			// Trying to affiliate to ALICE on AffiliatesAlpha will fail since
			// she is not Affiliatable there
			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				),
				Error::<Test, Instance1>::CannotAffiliateToAccount
			);

			// In AffiliatesBeta it works as expected
			assert_ok!(
				<AffiliatesBeta as AffiliateMutator<AccountIdOf<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				)
			);

			// Instance1 state doesn't contain the affiliation changes
			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), None);
			// While Instance2 does
			assert_eq!(Affiliators::<Test, Instance2>::get(ALICE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance2>::get(CHARLIE), Some(bounded_vec![ALICE]));
		});
	}

	#[test]
	fn rule_in_one_instance_doesnt_affect_other_instance() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let rule_id = 0;
			let rule = PayoutRuleOf::<Test, Instance1>::default();

			assert_ok!(<AffiliatesAlpha as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance1>>::AffiliateMaxLevel,
			>>::try_add_rule_for(rule_id, rule.clone()));

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id },
			));

			// The rule is added in Instance1
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule.clone()));
			// But not on Instance2
			assert_eq!(AffiliateRules::<Test, Instance2>::get(rule_id), None);

			<AffiliatesBeta as RuleMutator<
				AccountIdOf<Test>,
				<Test as Config<Instance2>>::AffiliateMaxLevel,
			>>::clear_rule_for(rule_id);

			// The rule remains in Instance1
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule));
			// No changes also in Instance2
			assert_eq!(AffiliateRules::<Test, Instance2>::get(rule_id), None);
		});
	}
}
