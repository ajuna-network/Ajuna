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
			let epoch_number = 0;

			run_to_block(10);

			let _ = create_and_deposit_tokens(&ALICE, token_amt, 0);
			let _ = create_and_deposit_tokens(&ALICE, token_amt, 0);
			let reserve = Wildcard::reserve_account();

			assert_eq!(Balances::free_balance(ALICE), initial_balance - token_amt * 2);
			assert_eq!(Balances::free_balance(reserve), token_amt * 2);

			Frozen::<Test>::set(Some(epoch_number));

			assert_ok!(Wildcard::refund_frozen(RuntimeOrigin::signed(ALICE)));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(
				crate::Event::DepositsRefunded { epoch: epoch_number, beneficieary: ALICE },
			));

			assert_eq!(Balances::free_balance(ALICE), initial_balance);
			assert_eq!(Balances::free_balance(reserve), 0);
		});
}
