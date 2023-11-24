use super::*;

#[test]
fn respond_challenge_success() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let (account, challenge_epoch, chunk_index) = make_challenge(ALICE);

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index));

			let proof = ZeroBalanceProof::using_challenge(challenge_epoch, account);
			let signature = generate_signature_for(&proof);

			assert_ok!(Wildcard::respond_zero_challenge(
				RuntimeOrigin::signed(ALICE),
				proof,
				signature
			));

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), None);

			System::assert_last_event(mock::RuntimeEvent::Wildcard(
				crate::Event::ChallengeZeroResponded {
					challenger: account,
					challenged_epoch: challenge_epoch,
					balance_proof: proof,
				},
			));
		});
}

#[test]
fn respond_challenge_with_challenge_keys_fails() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let (account, challenge_epoch, chunk_index) = make_challenge(ALICE);

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index));

			let proof = ZeroBalanceProof::using_challenge(challenge_epoch, BOB);
			let signature = generate_signature_for(&proof);

			assert_noop!(
				Wildcard::respond_zero_challenge(RuntimeOrigin::signed(BOB), proof, signature),
				Error::<Test>::WronglyRespondedChallenge
			);
		});
}
