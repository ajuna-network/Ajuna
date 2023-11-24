use super::*;

pub(crate) fn generate_signature_for_with(
	proof: &FreezeProof,
	signature: &sp_core::sr25519::Pair,
) -> sp_core::sr25519::Signature {
	signature.sign(proof.extract_msg().as_slice())
}

pub(crate) fn sign_public_key(key: &sp_core::sr25519::Public) -> sp_core::sr25519::Signature {
	let client_key_msg = {
		let mut bytes = sp_std::vec::Vec::with_capacity(0);

		bytes.extend(LIGHT_CLIENT_PROOF_PREFIX.to_vec());
		bytes.extend(key.to_vec());

		bytes
	};

	MockKeyPair::get().sign(&client_key_msg)
}

#[test]
fn propagate_freeze_success() {
	let key_pair = sp_core::sr25519::Pair::from_seed(&[2; 32]);

	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let freeze_epoch = 5;

			let freeze_proof = FreezeProof {
				identifier: Default::default(),
				epoch: freeze_epoch,
				origin: CHAIN_ID,
			};

			let freeze_signature = generate_signature_for_with(&freeze_proof, &key_pair);
			let key_signature = sign_public_key(&key_pair.public());

			assert_ok!(Wildcard::propagate_freeze(
				RuntimeOrigin::signed(BOB),
				freeze_proof,
				key_pair.public(),
				key_signature,
				freeze_signature
			));

			assert_eq!(Frozen::<Test>::get(), Some::<EpochNumber>(freeze_epoch));
		});
}

#[test]
fn propagate_freeze_fails_with_wrong_client_key_signature() {
	let key_pair = sp_core::sr25519::Pair::from_seed(&[2; 32]);
	let other_pair = sp_core::sr25519::Pair::from_seed(&[36; 32]);

	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let freeze_epoch = 9;

			let freeze_proof = FreezeProof {
				identifier: Default::default(),
				epoch: freeze_epoch,
				origin: CHAIN_ID,
			};

			let freeze_signature = generate_signature_for_with(&freeze_proof, &key_pair);
			let key_signature = sign_public_key(&key_pair.public());

			assert_noop!(
				Wildcard::propagate_freeze(
					RuntimeOrigin::signed(BOB),
					freeze_proof,
					other_pair.public(),
					key_signature,
					freeze_signature
				),
				Error::<Test>::BadSignature
			);
		});
}

#[test]
fn propagate_freeze_fails_with_wrong_proof_signature() {
	let key_pair = sp_core::sr25519::Pair::from_seed(&[2; 32]);

	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let freeze_epoch = 9;

			let freeze_proof = FreezeProof {
				identifier: Default::default(),
				epoch: freeze_epoch,
				origin: CHAIN_ID,
			};

			let freeze_signature = generate_signature_for_with(&freeze_proof, &key_pair);
			let key_signature = sign_public_key(&key_pair.public());

			let other_proof =
				FreezeProof { identifier: H256([82; 32]), epoch: freeze_epoch, origin: CHAIN_ID };

			assert_noop!(
				Wildcard::propagate_freeze(
					RuntimeOrigin::signed(BOB),
					other_proof,
					key_pair.public(),
					key_signature,
					freeze_signature
				),
				Error::<Test>::BadSignature
			);
		});
}

#[test]
fn propagate_freeze_fails_with_current_epoch_lower_than_4() {
	let key_pair = sp_core::sr25519::Pair::from_seed(&[2; 32]);

	ExtBuilder::default()
		.balances(&[(ALICE, ChallengeBalance::get())])
		.build()
		.execute_with(|| {
			run_to_block(1);

			let freeze_epoch = 4;

			let freeze_proof = FreezeProof {
				identifier: Default::default(),
				epoch: freeze_epoch,
				origin: CHAIN_ID,
			};

			let freeze_signature = generate_signature_for_with(&freeze_proof, &key_pair);
			let key_signature = sign_public_key(&key_pair.public());

			assert_noop!(
				Wildcard::propagate_freeze(
					RuntimeOrigin::signed(BOB),
					freeze_proof,
					key_pair.public(),
					key_signature,
					freeze_signature
				),
				Error::<Test>::InvalidEpochNumber
			);
		});
}
