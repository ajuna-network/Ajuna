use super::*;

#[test]
fn test_end2end_native_fungible() {
	let asset_balance = 1_000;
	let asset_id = 31;

	ExtBuilder::default().build().execute_with(|| {
		// ALICE deposits 500 units of a native fungible asset with id 31
		let asset_id = create_fungible(ALICE, asset_id, 1);
		mint_fungible(ALICE, asset_id, asset_balance);

		let deposit_amt = 500;

		let primary_id = generate_native_fungible_wide_id(asset_id);

		let asset_deposit = AssetDeposit {
			origin: CHAIN_ID,
			asset_type: AssetType::Fungible,
			primary_id,
			secondary_id: generate_wide_id_for_amount(deposit_amt),
		};

		let reserve = Wildcard::reserve_account();

		assert_eq!(Assets::balance(asset_id, ALICE), asset_balance);
		assert_eq!(Assets::balance(asset_id, reserve), 0);

		assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

		let asset = AssetOf::<Test> { origin: CHAIN_ID, kind: AssetKind::Fungible(asset_id) };

		let now = <Test as Config>::Time::now();

		let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Asset(deposit_amt));

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetDeposit {
			epoch: now,
			depositor: ALICE,
			asset_origin: asset_deposit.origin,
			asset_type: asset_deposit.asset_type,
			primary_id: asset_deposit.primary_id,
			secondary_id: asset_deposit.secondary_id,
		}));

		assert_eq!(Deposits::<Test>::get((now, ALICE, asset)), Some(value));

		assert_eq!(Assets::balance(asset_id, ALICE), asset_balance - deposit_amt);
		assert_eq!(Assets::balance(asset_id, reserve), deposit_amt);

		// BOB then withdraws 200 units with a balance proof to validate the transaction
		let withdraw_amt = 200;
		let now = <Test as Config>::Time::now();
		let asset = Asset { origin: CHAIN_ID, kind: AssetKind::Fungible(asset_id) };

		run_to_block(10);

		let proof = BalanceProof {
			epoch: now,
			origin: CHAIN_ID,
			account: H256::default(),
			exit_flag: true,
			chunk_index: 0,
			chunk_last: 0,
			asset_origin: CHAIN_ID,
			asset_type: AssetType::Fungible as u8,
			primary_id,
			secondary_id: generate_wide_id_for_amount(withdraw_amt),
		};
		let signature = generate_signature_for(&proof);

		assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(BOB), proof, signature));

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
			epoch: now,
			withdrawer: BOB,
			asset_origin: proof.origin,
			asset_type: AssetType::from(proof.asset_type),
			primary_id: proof.primary_id,
			secondary_id: proof.secondary_id,
		}));

		assert_eq!(Withdrawals::<Test>::get(BOB, asset), Some((now, 0)));

		assert_eq!(Assets::balance(asset_id, BOB), withdraw_amt);
		assert_eq!(Assets::balance(asset_id, reserve), deposit_amt - withdraw_amt);

		// CHARLIE then withdraws 300 units with a balance proof to validate the transaction
		let withdraw_amt = 300;
		let now = Pallet::<Test>::calculate_epoch_number_from(<Test as Config>::Time::now())
			.expect("Should calculate epoch num");
		let asset = Asset { origin: CHAIN_ID, kind: AssetKind::Fungible(asset_id) };

		run_to_block(25);

		let proof = BalanceProof {
			epoch: now,
			origin: CHAIN_ID,
			account: H256::default(),
			exit_flag: true,
			chunk_index: 0,
			chunk_last: 0,
			asset_origin: CHAIN_ID,
			asset_type: AssetType::Fungible as u8,
			primary_id,
			secondary_id: generate_wide_id_for_amount(withdraw_amt),
		};
		let signature = generate_signature_for(&proof);

		assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(CHARLIE), proof, signature));

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
			epoch: now,
			withdrawer: CHARLIE,
			asset_origin: proof.origin,
			asset_type: AssetType::from(proof.asset_type),
			primary_id: proof.primary_id,
			secondary_id: proof.secondary_id,
		}));

		assert_eq!(Withdrawals::<Test>::get(CHARLIE, asset), Some((now, 0)));

		assert_eq!(Assets::balance(asset_id, CHARLIE), withdraw_amt);
		assert_eq!(Assets::balance(asset_id, reserve), 0);
	});
}

#[test]
fn test_end2end_foreign_fungible() {
	let asset_id = 31;

	ExtBuilder::default().build().execute_with(|| {
		let withdraw_amt = 3000;
		let now = <Test as Config>::Time::now();
		let asset = Asset { origin: FOREIGN_CHAIN_ID, kind: AssetKind::Fungible(asset_id) };

		let primary_id = generate_foreign_fungible_wide_id(asset_id);

		assert_eq!(Assets::balance(asset_id, ALICE), 0);

		run_to_block(10);

		// ALICE withdraws 3000 units of a foreign fungible asset with id 31 using a BalanceProof
		let proof = BalanceProof {
			epoch: now,
			origin: FOREIGN_CHAIN_ID,
			account: H256::default(),
			exit_flag: true,
			chunk_index: 0,
			chunk_last: 0,
			asset_origin: FOREIGN_CHAIN_ID,
			asset_type: AssetType::Fungible as u8,
			primary_id,
			secondary_id: generate_wide_id_for_amount(withdraw_amt),
		};
		let signature = generate_signature_for(&proof);

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

		assert_eq!(Assets::balance(asset_id, ALICE), withdraw_amt);

		// ALICE then deposits 2000 units back of the foreign asset with id 31
		let deposit_amt = 2000;

		let asset_deposit = AssetDeposit {
			origin: FOREIGN_CHAIN_ID,
			asset_type: AssetType::Fungible,
			primary_id,
			secondary_id: generate_wide_id_for_amount(deposit_amt),
		};

		assert_eq!(Assets::balance(asset_id, ALICE), withdraw_amt);

		assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

		let asset =
			AssetOf::<Test> { origin: FOREIGN_CHAIN_ID, kind: AssetKind::Fungible(asset_id) };
		let now = Pallet::<Test>::calculate_epoch_number_from(<Test as Config>::Time::now())
			.expect("Should calculate epoch num");
		let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Asset(deposit_amt));

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetDeposit {
			epoch: now,
			depositor: ALICE,
			asset_origin: asset_deposit.origin,
			asset_type: asset_deposit.asset_type,
			primary_id: asset_deposit.primary_id,
			secondary_id: asset_deposit.secondary_id,
		}));

		assert_eq!(Deposits::<Test>::get((now, ALICE, &asset)), Some(value));

		assert_eq!(Assets::balance(asset_id, ALICE), withdraw_amt - deposit_amt);

		// BOB then withdraws 1200 units with a balance proof to validate the transaction
		let charlie_withdrawal_amt = 1200;

		run_to_block(25);

		let proof = BalanceProof {
			epoch: now,
			origin: FOREIGN_CHAIN_ID,
			account: H256::default(),
			exit_flag: true,
			chunk_index: 0,
			chunk_last: 0,
			asset_origin: FOREIGN_CHAIN_ID,
			asset_type: AssetType::Fungible as u8,
			primary_id,
			secondary_id: generate_wide_id_for_amount(charlie_withdrawal_amt),
		};
		let signature = generate_signature_for(&proof);

		assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(CHARLIE), proof, signature));

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
			epoch: now,
			withdrawer: CHARLIE,
			asset_origin: proof.origin,
			asset_type: AssetType::from(proof.asset_type),
			primary_id: proof.primary_id,
			secondary_id: proof.secondary_id,
		}));

		assert_eq!(Withdrawals::<Test>::get(CHARLIE, &asset), Some((now, 0)));

		assert_eq!(Assets::balance(asset_id, CHARLIE), charlie_withdrawal_amt);
		assert_eq!(Assets::balance(asset_id, ALICE), withdraw_amt - deposit_amt);
	});
}

#[test]
fn test_end2end_native_non_fungible() {
	ExtBuilder::default().build().execute_with(|| {
		// ALICE deposits a non-fungible asset
		let (collection_id, item_id) = mint_non_fungible(&ALICE, &create_collection(&ALICE), &1);
		let (primary_id, secondary_id) =
			generate_native_non_fungible_wide_id(collection_id, item_id);

		let asset_deposit = AssetDeposit {
			origin: CHAIN_ID,
			asset_type: AssetType::NonFungible,
			primary_id,
			secondary_id,
		};

		assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));

		assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

		let asset = AssetOf::<Test> {
			origin: CHAIN_ID,
			kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
		};
		let now = <Test as Config>::Time::now();
		let value = DepositValueKindOf::<Test>::NonFungible;

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetDeposit {
			epoch: now,
			depositor: ALICE,
			asset_origin: asset_deposit.origin,
			asset_type: asset_deposit.asset_type,
			primary_id: asset_deposit.primary_id,
			secondary_id: asset_deposit.secondary_id,
		}));

		let reserve_account = Wildcard::reserve_account();

		assert_eq!(Deposits::<Test>::get((now, ALICE, &asset)), Some(value));

		assert_eq!(Nft::owner(collection_id, item_id), Some(reserve_account));

		// BOB then withdraws the NFT
		let proof = BalanceProof {
			epoch: now,
			origin: CHAIN_ID,
			account: H256::default(),
			exit_flag: true,
			chunk_index: 0,
			chunk_last: 0,
			asset_origin: CHAIN_ID,
			asset_type: AssetType::NonFungible as u8,
			primary_id,
			secondary_id,
		};
		let signature = generate_signature_for(&proof);

		run_to_block(10);

		assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(BOB), proof, signature));

		System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
			epoch: now,
			withdrawer: BOB,
			asset_origin: proof.origin,
			asset_type: AssetType::from(proof.asset_type),
			primary_id: proof.primary_id,
			secondary_id: proof.secondary_id,
		}));

		assert_eq!(Withdrawals::<Test>::get(BOB, &asset), Some((now, 0)));
		assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
	});
}

#[test]
fn test_end2end_foreign_non_fungible() {
	let initial_balance = 1_000_000;

	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance), (CHARLIE, initial_balance)])
		.build()
		.execute_with(|| {
			let _ = CurrencyOf::<Test>::deposit_creating(
				&Pallet::<Test>::reserve_account(),
				initial_balance,
			);

			let (primary_id, secondary_id) = generate_foreign_non_fungible_wide_id(34, 23);
			let now = <Test as Config>::Time::now();

			// ALICE withdraws a non-fungible asset
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

			let collection_id = CollectionIdMapping::<Test>::get((
				proof.origin,
				AssetType::NonFungible,
				proof.primary_id,
			))
			.expect("Should get id");
			let item_id = ItemIdMapping::<Test>::get((
				proof.origin,
				AssetType::NonFungible,
				proof.secondary_id,
			))
			.expect("Should get id");

			let asset = AssetOf::<Test> {
				origin: FOREIGN_CHAIN_ID,
				kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
			};

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
			// ALICE then deposits it back
			run_to_block(20);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id: proof.primary_id,
				secondary_id: proof.secondary_id,
			};

			assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

			assert_eq!(Nft::owner(collection_id, item_id), None);
			// CHARLIE withdraws it again
			run_to_block(25);

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

			run_to_block(35);

			assert_ok!(Wildcard::withdraw(RuntimeOrigin::signed(CHARLIE), proof, signature));

			let asset = AssetOf::<Test> {
				origin: FOREIGN_CHAIN_ID,
				kind: AssetKind::NonFungible(NftAddress(collection_id, item_id)),
			};

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetWithdraw {
				epoch: now,
				withdrawer: CHARLIE,
				asset_origin: proof.origin,
				asset_type: AssetType::from(proof.asset_type),
				primary_id: proof.primary_id,
				secondary_id: proof.secondary_id,
			}));

			assert_eq!(Withdrawals::<Test>::get(CHARLIE, asset), Some((now, 0)));
			assert_eq!(Nft::owner(collection_id, item_id), Some(CHARLIE));
		});
}
