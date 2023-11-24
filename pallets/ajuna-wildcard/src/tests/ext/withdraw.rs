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

fn create_and_deposit_fungible_asset(
	who: &MockAccountId,
	origin: AssetOrigin,
	asset_id: MockAssetId,
	balance: MockBalance,
	deposit_amt: MockBalance,
) -> (AssetOf<Test>, DepositValueKindOf<Test>, AssetDeposit) {
	let asset_id = create_fungible(*who, asset_id, 1);
	mint_fungible(*who, asset_id, balance);

	let asset = AssetOf::<Test> { origin, kind: AssetKind::Fungible(asset_id) };

	let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Asset(deposit_amt));

	let primary_id = if origin == CHAIN_ID {
		generate_native_fungible_wide_id(asset_id)
	} else {
		generate_foreign_fungible_wide_id(asset_id)
	};

	let asset_deposit = AssetDeposit {
		origin,
		asset_type: AssetType::Fungible,
		primary_id,
		secondary_id: generate_wide_id_for_amount(deposit_amt),
	};

	deposit_asset(who, asset_deposit);

	(asset, value, asset_deposit)
}

fn create_and_deposit_native_non_fungible(
	who: &MockAccountId,
	item_base_id: u64,
) -> (AssetOf<Test>, DepositValueKindOf<Test>, NftAddressOf<Test>, AssetDeposit) {
	let (collection_id, item_id) =
		mint_non_fungible(who, &create_collection(who), &(item_base_id as u32));

	let addr = NftAddress(collection_id, item_id);

	let asset = AssetOf::<Test> { origin: CHAIN_ID, kind: AssetKind::NonFungible(addr.clone()) };

	let value = DepositValueKindOf::<Test>::NonFungible;

	let (primary_id, secondary_id) = generate_native_non_fungible_wide_id(collection_id, item_id);

	let asset_deposit = AssetDeposit {
		origin: CHAIN_ID,
		asset_type: AssetType::NonFungible,
		primary_id,
		secondary_id,
	};

	deposit_asset(who, asset_deposit);

	(asset, value, addr, asset_deposit)
}

fn create_and_deposit_foreign_non_fungible(
	who: &MockAccountId,
	item_base_id: u64,
) -> (AssetOf<Test>, DepositValueKindOf<Test>, NftAddressOf<Test>, AssetDeposit) {
	let (collection_id, item_id) =
		mint_non_fungible(who, &create_collection(who), &(item_base_id as u32));

	let addr = NftAddress(collection_id, item_id);

	let asset =
		AssetOf::<Test> { origin: FOREIGN_CHAIN_ID, kind: AssetKind::NonFungible(addr.clone()) };

	let value = DepositValueKindOf::<Test>::NonFungible;

	let (primary_id, secondary_id) = generate_foreign_non_fungible_wide_id(collection_id, item_id);

	let asset_deposit = AssetDeposit {
		origin: CHAIN_ID,
		asset_type: AssetType::NonFungible,
		primary_id,
		secondary_id,
	};

	deposit_asset(who, asset_deposit);

	(asset, value, addr, asset_deposit)
}

mod fungible_native {
	use super::*;
	use sp_core::H256;

	#[test]
	fn test_token_success() {
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

				assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature));

				System::assert_last_event(mock::RuntimeEvent::Wildcard(
					crate::Event::AssetWithdraw {
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
	fn test_token_error_invalid_proof() {
		let token_amt = 1_000;
		let initial_balance = 1_000_000;

		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let now = <Test as Config>::Time::now();
				let (_, _, deposit) = create_and_deposit_tokens(&ALICE, token_amt, 0);

				let proof_err = {
					let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
					proof.primary_id = H256::default();
					proof
				};
				let signature = generate_signature_for(&proof_err);

				run_to_block(10);

				assert_noop!(
					Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
					Error::<Test>::InvalidInput
				);

				let invalid_signature = sp_core::sr25519::Signature([34; 64]);
				let proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);

				assert_noop!(
					Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, invalid_signature),
					Error::<Test>::BadSignature
				);

				let proof_err = {
					let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
					proof.exit_flag = false;
					proof
				};
				let signature = generate_signature_for(&proof_err);

				assert_noop!(
					Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
					Error::<Test>::BadExitFlag
				);
			});
	}

	#[test]
	fn test_token_error_withdrawal_greater_than_deposit() {
		let token_amt = 1_000;
		let initial_balance = 1_000_000;

		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let now = <Test as Config>::Time::now();
				let (_, _, deposit) = create_and_deposit_tokens(&ALICE, token_amt, 0);

				let proof_err = {
					let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
					proof.secondary_id = generate_wide_id_for_amount(token_amt * 10);
					proof
				};
				let signature = generate_signature_for(&proof_err);

				run_to_block(10);

				assert_noop!(
					Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
					Error::<Test>::InsufficientReserveFunds
				);
			});
	}

	#[test]
	fn test_asset_success() {
		let token_amt = 1_000;
		let asset_id = 18;
		let initial_balance = 1_000_000;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (asset, _, deposit) = create_and_deposit_fungible_asset(
				&ALICE,
				CHAIN_ID,
				asset_id,
				initial_balance,
				token_amt,
			);
			let reserve = Wildcard::reserve_account();

			assert_eq!(Assets::balance(asset_id, ALICE), initial_balance - token_amt);
			assert_eq!(Assets::balance(asset_id, reserve), token_amt);

			let proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
				epoch: now,
				withdrawer: ALICE,
				asset_origin: proof.origin,
				asset_type: AssetType::from(proof.asset_type),
				primary_id: proof.primary_id,
				secondary_id: proof.secondary_id,
			}));

			assert_eq!(Withdrawals::<Test>::get(ALICE, asset), Some((now, 0)));

			assert_eq!(Assets::balance(asset_id, ALICE), initial_balance);
			assert_eq!(Assets::balance(asset_id, reserve), 0);
		});
	}

	#[test]
	fn test_error_invalid_input() {
		let token_amt = 1_000;
		let asset_id = 18;
		let initial_balance = 1_000_000;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (_, _, deposit) = create_and_deposit_fungible_asset(
				&ALICE,
				CHAIN_ID,
				asset_id,
				initial_balance,
				token_amt,
			);

			let proof_err = {
				let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
				proof.asset_type = AssetType::NonFungible as u8;
				proof
			};
			let signature = generate_signature_for(&proof_err);

			run_to_block(10);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn test_error_withdrawal_greater_than_deposit() {
		let token_amt = 1_000;
		let asset_id = 457;
		let initial_balance = 1_000_000;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (_, _, deposit) = create_and_deposit_fungible_asset(
				&ALICE,
				CHAIN_ID,
				asset_id,
				initial_balance,
				token_amt,
			);
			let proof_err = {
				let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
				proof.secondary_id = generate_wide_id_for_amount(token_amt * 10);
				proof
			};
			let signature = generate_signature_for(&proof_err);

			run_to_block(10);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
				Error::<Test>::InsufficientReserveFunds
			);
		});
	}
}

mod fungible_foreign {
	use super::*;

	#[test]
	fn test_success() {
		let token_amt = 1_000;
		let asset_id = 18;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let reserve = Wildcard::reserve_account();

			assert_eq!(Assets::balance(asset_id, ALICE), 0);
			assert_eq!(Assets::balance(asset_id, reserve), 0);

			let asset = Asset { origin: FOREIGN_CHAIN_ID, kind: AssetKind::Fungible(asset_id) };

			let proof = BalanceProof {
				epoch: now,
				origin: FOREIGN_CHAIN_ID,
				account: H256::default(),
				exit_flag: true,
				chunk_index: 0,
				chunk_last: 0,
				asset_origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::Fungible as u8,
				primary_id: generate_foreign_fungible_wide_id(asset_id),
				secondary_id: generate_wide_id_for_amount(token_amt),
			};
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
				epoch: now,
				withdrawer: ALICE,
				asset_origin: proof.origin,
				asset_type: AssetType::from(proof.asset_type),
				primary_id: proof.primary_id,
				secondary_id: proof.secondary_id,
			}));

			assert_eq!(Withdrawals::<Test>::get(ALICE, asset), Some((now, 0)));

			assert_eq!(Assets::balance(asset_id, ALICE), token_amt);
			assert_eq!(Assets::balance(asset_id, reserve), 0);
		});
	}

	#[test]
	fn test_error_invalid_proof() {
		let token_amt = 1_000;
		let asset_id = 18;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let _ =
				CurrencyOf::<Test>::deposit_creating(&Pallet::<Test>::reserve_account(), 1_000_000);

			let proof_err = BalanceProof {
				epoch: now,
				origin: CHAIN_ID,
				account: H256::default(),
				exit_flag: true,
				chunk_index: 0,
				chunk_last: 0,
				asset_origin: CHAIN_ID,
				asset_type: AssetType::Fungible as u8,
				primary_id: generate_foreign_fungible_wide_id(asset_id),
				secondary_id: generate_wide_id_for_amount(token_amt),
			};
			let signature = generate_signature_for(&proof_err);

			run_to_block(10);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
				Error::<Test>::InvalidInput
			);

			let proof = BalanceProof {
				epoch: now,
				origin: FOREIGN_CHAIN_ID,
				account: H256::default(),
				exit_flag: true,
				chunk_index: 0,
				chunk_last: 0,
				asset_origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible as u8,
				primary_id: generate_foreign_fungible_wide_id(asset_id),
				secondary_id: generate_wide_id_for_amount(token_amt),
			};
			let invalid_signature = sp_core::sr25519::Signature([34; 64]);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, invalid_signature),
				Error::<Test>::BadSignature
			);

			let proof_err = BalanceProof {
				epoch: now,
				origin: FOREIGN_CHAIN_ID,
				account: H256::default(),
				exit_flag: false,
				chunk_index: 0,
				chunk_last: 0,
				asset_origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible as u8,
				primary_id: generate_foreign_fungible_wide_id(asset_id),
				secondary_id: generate_wide_id_for_amount(token_amt),
			};
			let signature = generate_signature_for(&proof_err);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
				Error::<Test>::BadExitFlag
			);
		});
	}
}

mod non_fungible {
	use super::*;

	#[test]
	fn test_native_success() {
		let asset_id = 18;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (asset, _, NftAddress(collection_id, item_id), deposit) =
				create_and_deposit_native_non_fungible(&ALICE, asset_id);
			let reserve = Wildcard::reserve_account();

			assert_eq!(Nft::owner(collection_id, item_id), Some(reserve));

			let proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
				epoch: now,
				withdrawer: ALICE,
				asset_origin: proof.origin,
				asset_type: AssetType::from(proof.asset_type),
				primary_id: proof.primary_id,
				secondary_id: proof.secondary_id,
			}));

			assert_eq!(Withdrawals::<Test>::get(ALICE, asset), Some((now, 0)));
			assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
		});
	}

	#[test]
	fn test_native_error_invalid_proof() {
		let asset_id = 18;

		ExtBuilder::default().build().execute_with(|| {
			let now = <Test as Config>::Time::now();
			let (_, _, NftAddress(collection_id, item_id), deposit) =
				create_and_deposit_native_non_fungible(&ALICE, asset_id);

			let reserve = Wildcard::reserve_account();

			assert_eq!(Nft::owner(collection_id, item_id), Some(reserve));

			let proof_err = {
				let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
				proof.asset_type = AssetType::Fungible as u8;
				proof
			};
			let signature = generate_signature_for(&proof_err);

			run_to_block(10);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
				Error::<Test>::InvalidInput
			);

			let proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
			let invalid_signature = sp_core::sr25519::Signature([34; 64]);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, invalid_signature),
				Error::<Test>::BadSignature
			);

			let proof_err = {
				let mut proof = BalanceProof::using_deposit(&deposit, now, ALICE, true, 0);
				proof.exit_flag = false;
				proof
			};
			let signature = generate_signature_for(&proof_err);

			assert_noop!(
				Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof_err, signature),
				Error::<Test>::BadExitFlag
			);
		});
	}

	#[test]
	fn test_foreign_success() {
		ExtBuilder::default().build().execute_with(|| {
			let _ =
				CurrencyOf::<Test>::deposit_creating(&Pallet::<Test>::reserve_account(), 1_000_000);

			let now = <Test as Config>::Time::now();
			let collection_id = 33;
			let item_id = 44;

			assert_eq!(Nft::owner(collection_id, item_id), None);

			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);

			let proof = BalanceProof {
				epoch: now,
				origin: FOREIGN_CHAIN_ID,
				account: H256::default(),
				exit_flag: true,
				chunk_index: 0,
				chunk_last: 0,
				asset_origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible as u8,
				primary_id,
				secondary_id,
			};
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature));

			let asset_type = AssetType::from(proof.asset_type);

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
				epoch: now,
				withdrawer: ALICE,
				asset_origin: proof.origin,
				asset_type,
				primary_id: proof.primary_id,
				secondary_id: proof.secondary_id,
			}));

			let mapping_key = (proof.origin, asset_type, proof.primary_id);

			let collection_id =
				CollectionIdMapping::<Test>::get(mapping_key).expect("Should get id");
			let asset = Asset {
				origin: FOREIGN_CHAIN_ID,
				kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
			};

			assert_eq!(Withdrawals::<Test>::get(ALICE, asset), Some((now, 0)));
			assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
		});
	}

	#[test]
	fn test_foreign_withdraw_generates_new_mapping() {
		ExtBuilder::default().build().execute_with(|| {
			let _ =
				CurrencyOf::<Test>::deposit_creating(&Pallet::<Test>::reserve_account(), 1_000_000);
			let now = <Test as Config>::Time::now();
			let collection_id = 33;
			let item_id = 44;

			assert_eq!(Nft::owner(collection_id, item_id), None);

			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);

			let proof = BalanceProof {
				epoch: now,
				origin: FOREIGN_CHAIN_ID,
				account: H256::default(),
				exit_flag: true,
				chunk_index: 0,
				chunk_last: 0,
				asset_origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible as u8,
				primary_id,
				secondary_id,
			};
			let signature = generate_signature_for(&proof);

			run_to_block(10);

			let mapping_key = (proof.origin, AssetType::NonFungible, proof.primary_id);
			assert!(!CollectionIdMapping::<Test>::contains_key(mapping_key));

			assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature));

			let mapping_key = (proof.origin, AssetType::NonFungible, proof.primary_id);
			let collection_id = CollectionIdMapping::<Test>::get(mapping_key);
			assert!(collection_id.is_some());
			let collection_id = collection_id.unwrap();
			let asset = Asset {
				origin: FOREIGN_CHAIN_ID,
				kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
			};

			assert_eq!(Withdrawals::<Test>::get(ALICE, asset), Some((now, 0)));
			assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));
		});
	}

	#[test]
	fn test_foreign_error_missing_mapping_with_no_balance() {
		let existential_deposit = MockExistentialDeposit::get();

		ExtBuilder::default()
			.balances(&[(ALICE, existential_deposit)])
			.build()
			.execute_with(|| {
				let now = <Test as Config>::Time::now();
				let collection_id = 12;
				let item_id = 34;

				assert_eq!(Nft::owner(collection_id, item_id), None);

				let (primary_id, secondary_id) =
					generate_foreign_non_fungible_wide_id(collection_id, item_id);

				let proof = BalanceProof {
					epoch: now,
					origin: FOREIGN_CHAIN_ID,
					account: H256::default(),
					exit_flag: true,
					chunk_index: 0,
					chunk_last: 0,
					asset_origin: FOREIGN_CHAIN_ID,
					asset_type: AssetType::NonFungible as u8,
					primary_id,
					secondary_id,
				};
				let signature = generate_signature_for(&proof);

				run_to_block(10);

				assert_noop!(
					Wildcard::withdraw(RuntimeOrigin::signed(ALICE), proof, signature),
					pallet_balances::Error::<Test>::InsufficientBalance
				);
			});
	}
}
