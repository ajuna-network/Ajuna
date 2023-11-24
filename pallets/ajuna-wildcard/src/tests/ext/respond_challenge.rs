use super::*;

#[test]
fn respond_challenge_success() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let (account, challenge_epoch, chunk_index) = make_challenge(ALICE);
			let chunk_last = chunk_index + 3;

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index));

			let proof =
				BalanceProof::using_challenge(challenge_epoch, account, chunk_index, chunk_last);
			let signature = generate_signature_for(&proof);

			assert_ok!(Wildcard::respond_challenge(RuntimeOrigin::signed(ALICE), proof, signature));

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index + 1));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(
				crate::Event::ChallengeResponded {
					challenger: account,
					challenged_epoch: challenge_epoch,
					balance_proof: proof,
				},
			));
		});
}

#[test]
fn respond_challenge_with_last_chunk_index_removes_challenge_entry() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let (account, challenge_epoch, chunk_index) = make_challenge(ALICE);

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index));

			let proof =
				BalanceProof::using_challenge(challenge_epoch, account, chunk_index, chunk_index);
			let signature = generate_signature_for(&proof);

			assert_ok!(Wildcard::respond_challenge(RuntimeOrigin::signed(ALICE), proof, signature));

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), None);

			System::assert_last_event(mock::RuntimeEvent::Wildcard(
				crate::Event::ChallengeResponded {
					challenger: account,
					challenged_epoch: challenge_epoch,
					balance_proof: proof,
				},
			));
		});
}

#[test]
fn respond_challenge_with_incorrect_index_fails() {
	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let (account, challenge_epoch, chunk_index) = make_challenge(ALICE);
			let modified_index = chunk_index + 1;

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index));

			let proof = BalanceProof::using_challenge(
				challenge_epoch,
				account,
				modified_index,
				modified_index,
			);
			let signature = generate_signature_for(&proof);

			assert_noop!(
				Wildcard::respond_challenge(RuntimeOrigin::signed(BOB), proof, signature),
				Error::<Test>::WrongChunkRespondedChallenge
			);
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
			let modified_index = chunk_index + 1;

			assert_eq!(Challenges::<Test>::get((challenge_epoch, account)), Some(chunk_index));

			let proof =
				BalanceProof::using_challenge(challenge_epoch, BOB, modified_index, modified_index);
			let signature = generate_signature_for(&proof);

			assert_noop!(
				Wildcard::respond_challenge(RuntimeOrigin::signed(BOB), proof, signature),
				Error::<Test>::WronglyRespondedChallenge
			);
		});
}
