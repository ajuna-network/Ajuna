use super::*;

fn deposit_asset(who: &MockAccountId, asset_deposit: AssetDeposit) {
	assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(*who), asset_deposit));
}

fn create_and_deposit_tokens(
	who: &MockAccountId,
	balance: MockBalance,
	create_amount: MockBalance,
) -> (AssetOf<Test>, DepositValueKindOf<Test>, AssetDeposit) {
	let asset = AssetOf::<Test> { origin: CHAIN_ID, kind: AssetKind::Fungible(NATIVE_TOKEN_ID) };

	let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Token(balance));

	let asset_deposit = AssetDeposit {
		origin: CHAIN_ID,
		asset_type: AssetType::Fungible,
		primary_id: generate_native_fungible_wide_id(NATIVE_TOKEN_ID),
		secondary_id: generate_wide_id_for_amount(balance),
	};

	let _ = <Test as Config>::Currency::deposit_into_existing(who, create_amount)
		.expect("Balance should be deposited");

	deposit_asset(who, asset_deposit);

	(asset, value, asset_deposit)
}

#[test]
fn test_success() {
	let token_amt = 1_000;
	let initial_balance = 1_000_000;

	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (asset, _, deposit) = create_and_deposit_tokens(&ALICE, token_amt, 0);
			let reserve = Wildcard::reserve_account();

			assert_eq!(Balances::free_balance(ALICE), initial_balance - token_amt);
			assert_eq!(Balances::free_balance(reserve), token_amt);

			let proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			Frozen::<Test>::set(Some(now));

			assert_ok!(Wildcard::withdraw_frozen(RuntimeOrigin::signed(ALICE), proof, signature));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(
				crate::Event::FrozenAssetWithdraw {
					epoch: now,
					withdrawer: ALICE,
					asset_origin: proof.origin,
					asset_type: AssetType::from(proof.asset_type),
					primary_id: proof.primary_id,
					secondary_id: proof.secondary_id,
				},
			));

			assert_eq!(Withdrawals::<Test>::get(ALICE, asset), Some((now, 0)));

			assert_eq!(Balances::free_balance(ALICE), initial_balance);
			assert_eq!(Balances::free_balance(reserve), 0);
		});
}

#[test]
fn test_cannot_withdraw_in_non_frozen_state() {
	let token_amt = 1_000;
	let initial_balance = 1_000_000;

	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (_, _, deposit) = create_and_deposit_tokens(&ALICE, token_amt, 0);
			let reserve = Wildcard::reserve_account();

			assert_eq!(Balances::free_balance(ALICE), initial_balance - token_amt);
			assert_eq!(Balances::free_balance(reserve), token_amt);

			let proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			assert_noop!(
				Wildcard::withdraw_frozen(RuntimeOrigin::signed(ALICE), proof, signature),
				Error::<Test>::NotFrozen
			);
		});
}

#[test]
fn test_cannot_withdraw_in_wron_freeze_epoch() {
	let token_amt = 1_000;
	let initial_balance = 1_000_000;

	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance)])
		.build()
		.execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (_, _, deposit) = create_and_deposit_tokens(&ALICE, token_amt, 0);
			let reserve = Wildcard::reserve_account();

			assert_eq!(Balances::free_balance(ALICE), initial_balance - token_amt);
			assert_eq!(Balances::free_balance(reserve), token_amt);

			let proof = BalanceProof::using_deposit(&deposit, now + 2, ALICE, true, 0);
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			Frozen::<Test>::set(Some(now));

			assert_noop!(
				Wildcard::withdraw_frozen(RuntimeOrigin::signed(ALICE), proof, signature),
				Error::<Test>::InvalidEpochNumber
			);
		});
}
