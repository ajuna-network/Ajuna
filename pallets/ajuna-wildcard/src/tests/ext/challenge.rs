use super::*;

#[test]
fn challenge_success() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let epoch_number =
				Pallet::<Test>::calculate_epoch_number_from(<Test as Config>::Time::now())
					.expect("Should get epoch");

			assert_eq!(
				Challenges::<Test>::get((epoch_number - 1, ALICE)),
				Option::<ChunkIndex>::None
			);

			assert_ok!(Wildcard::challenge(RuntimeOrigin::signed(ALICE)));

			assert_eq!(Challenges::<Test>::get((epoch_number - 1, ALICE)), Some(0));
		});
}

#[test]
fn challenge_fails_without_enough_free_balance() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get() - 1)])
		.build()
		.execute_with(|| {
			run_to_block(10);

			assert_noop!(
				Wildcard::challenge(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::InsufficientChallengeBalance
			);
		});
}

#[test]
fn challenge_fails_with_too_low_epoch() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Wildcard::challenge(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::InvalidEpochNumber
			);
		});
}

#[test]
fn challenge_fails_with_repeated_challenges() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);
			let epoch_number =
				Pallet::<Test>::calculate_epoch_number_from(<Test as Config>::Time::now())
					.expect("Should get epoch");

			assert_eq!(
				Challenges::<Test>::get((epoch_number - 1, ALICE)),
				Option::<ChunkIndex>::None
			);

			assert_ok!(Wildcard::challenge(RuntimeOrigin::signed(ALICE)));

			assert_eq!(Challenges::<Test>::get((epoch_number - 1, ALICE)), Some(0));

			assert_noop!(
				Wildcard::challenge(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::AlreadyChallengedEpoch
			);
		});
}
