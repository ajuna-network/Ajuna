use super::*;

#[test]
fn freeze_success() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);
			let (_, epoch_number, _) = make_challenge(ALICE);
			run_to_block(12);

			assert_ok!(Wildcard::freeze(RuntimeOrigin::signed(BOB)));

			assert_eq!(Frozen::<Test>::get(), Some::<EpochNumber>(epoch_number - 1));
		});
}

#[test]
fn freeze_fails_if_already_frozen() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			Frozen::<Test>::set(Some(1));

			assert_noop!(Wildcard::freeze(RuntimeOrigin::signed(BOB)), Error::<Test>::PalletFrozen);
		});
}

#[test]
fn freeze_fails_if_epoch_lower_than_min() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(1);

			assert_noop!(
				Wildcard::freeze(RuntimeOrigin::signed(BOB)),
				Error::<Test>::InvalidEpochNumber
			);
		});
}

#[test]
fn freeze_fails_if_out_of_sequence() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(1);

			assert_noop!(
				Wildcard::freeze(RuntimeOrigin::signed(BOB)),
				Error::<Test>::InvalidEpochNumber
			);
		});
}

#[test]
fn freeze_fials_if_no_challenge_for_epoch_present() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			assert_noop!(
				Wildcard::freeze(RuntimeOrigin::signed(BOB)),
				Error::<Test>::NothingToFreeze
			);
		});
}
