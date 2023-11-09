use super::*;

mod fungible_native {
	use super::*;

	#[test]
	fn test_token_success() {
		let initial_balance = 1_000_000;

		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let token_amt = 100_000;

				let asset_deposit = AssetDeposit {
					origin: CHAIN_ID,
					asset_type: AssetType::Fungible,
					primary_id: generate_native_fungible_wide_id(NATIVE_TOKEN_ID),
					secondary_id: generate_wide_id_for_amount(token_amt),
				};

				let reserve = Wildcard::reserve_account();

				assert_eq!(Balances::free_balance(ALICE), initial_balance);
				assert_eq!(Balances::free_balance(reserve), 0);

				assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

				let asset = AssetOf::<Test> {
					origin: CHAIN_ID,
					kind: AssetKind::Fungible(NATIVE_TOKEN_ID),
				};

				let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Token(token_amt));
				let now = <Test as Config>::Time::now();

				System::assert_last_event(mock::RuntimeEvent::Wildcard(
					crate::Event::AssetDeposit {
						epoch: now,
						depositor: ALICE,
						asset_origin: asset_deposit.origin,
						asset_type: asset_deposit.asset_type,
						primary_id: asset_deposit.primary_id,
						secondary_id: asset_deposit.secondary_id,
					},
				));

				assert_eq!(Deposits::<Test>::get((now, ALICE, asset)), Some(value));

				assert_eq!(Balances::free_balance(ALICE), initial_balance - token_amt);
				assert_eq!(Balances::free_balance(reserve), token_amt);
			});
	}

	#[test]
	fn test_token_error_invalid_input() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id: generate_foreign_fungible_wide_id(NATIVE_TOKEN_ID),
				secondary_id: generate_wide_id_for_amount(1),
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn test_token_error_not_enough_balance() {
		let initial_balance = 1_000_000;

		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let asset_deposit = AssetDeposit {
					origin: CHAIN_ID,
					asset_type: AssetType::Fungible,
					primary_id: generate_native_fungible_wide_id(NATIVE_TOKEN_ID),
					secondary_id: generate_wide_id_for_amount(initial_balance * 10),
				};

				assert_noop!(
					Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
					sp_runtime::ArithmeticError::Underflow,
				);
			});
	}

	#[test]
	fn test_asset_success() {
		let foreign_asset_id = 643;
		let foreign_asset_balance = 1_000;

		ExtBuilder::default().build().execute_with(|| {
			let asset_amt = 100;

			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id: generate_native_fungible_wide_id(foreign_asset_id),
				secondary_id: generate_wide_id_for_amount(asset_amt),
			};

			let asset_id = create_fungible(ALICE, foreign_asset_id, 1);
			mint_fungible(ALICE, asset_id, foreign_asset_balance);

			let reserve = Wildcard::reserve_account();

			assert_eq!(Assets::balance(asset_id, ALICE), foreign_asset_balance);
			assert_eq!(Assets::balance(asset_id, reserve), 0);

			assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

			let asset =
				AssetOf::<Test> { origin: CHAIN_ID, kind: AssetKind::Fungible(foreign_asset_id) };

			let now = <Test as Config>::Time::now();

			let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Asset(asset_amt));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetDeposit {
				epoch: now,
				depositor: ALICE,
				asset_origin: asset_deposit.origin,
				asset_type: asset_deposit.asset_type,
				primary_id: asset_deposit.primary_id,
				secondary_id: asset_deposit.secondary_id,
			}));

			assert_eq!(Deposits::<Test>::get((now, ALICE, asset)), Some(value));

			assert_eq!(Assets::balance(asset_id, ALICE), foreign_asset_balance - asset_amt);
			assert_eq!(Assets::balance(asset_id, reserve), asset_amt);
		});
	}

	#[test]
	fn test_asset_error_invalid_input() {
		let asset_id = 99;
		ExtBuilder::default().build().execute_with(|| {
			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id: generate_native_fungible_wide_id(asset_id),
				secondary_id: generate_wide_id_for_amount(1),
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				sp_runtime::TokenError::UnknownAsset
			);
		});
	}

	#[test]
	fn test_asset_error_not_enough_balance() {
		let foreign_asset_id = 4574;
		let foreign_asset_balance = 1_000;

		ExtBuilder::default().build().execute_with(|| {
			let asset_id = create_fungible(ALICE, foreign_asset_id, 1);
			mint_fungible(ALICE, asset_id, foreign_asset_balance);

			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id: generate_native_fungible_wide_id(asset_id),
				secondary_id: generate_wide_id_for_amount(foreign_asset_balance * 10),
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				sp_runtime::ArithmeticError::Underflow,
			);
		});
	}
}

mod fungible_foreign {
	use super::*;

	#[test]
	fn test_success() {
		let foreign_asset_balance = 1_000;
		let foreign_asset_id = 31;

		ExtBuilder::default().build().execute_with(|| {
			let asset_id = create_fungible(ALICE, foreign_asset_id, 1);
			mint_fungible(ALICE, asset_id, foreign_asset_balance);

			let asset_amt = 100;

			let primary_id = generate_foreign_fungible_wide_id(asset_id);
			AssetIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::Fungible, primary_id),
				asset_id,
			);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id,
				secondary_id: generate_wide_id_for_amount(asset_amt),
			};

			let reserve = Wildcard::reserve_account();

			assert_eq!(Assets::balance(asset_id, ALICE), foreign_asset_balance);
			assert_eq!(Assets::balance(asset_id, reserve), 0);

			assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

			let asset = AssetOf::<Test> {
				origin: FOREIGN_CHAIN_ID,
				kind: AssetKind::Fungible(foreign_asset_id),
			};

			let now = <Test as Config>::Time::now();

			let value = DepositValueKindOf::<Test>::Fungible(DepositValue::Asset(asset_amt));

			System::assert_last_event(mock::RuntimeEvent::Wildcard(crate::Event::AssetDeposit {
				epoch: now,
				depositor: ALICE,
				asset_origin: asset_deposit.origin,
				asset_type: asset_deposit.asset_type,
				primary_id: asset_deposit.primary_id,
				secondary_id: asset_deposit.secondary_id,
			}));

			assert_eq!(Deposits::<Test>::get((now, ALICE, asset)), Some(value));

			assert_eq!(Assets::balance(asset_id, ALICE), foreign_asset_balance - asset_amt);
			assert_eq!(Assets::balance(asset_id, reserve), 0);
		});
	}

	#[test]
	fn test_error_invalid_input() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id: generate_native_fungible_wide_id(2),
				secondary_id: generate_wide_id_for_amount(1),
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn test_error_not_enough_balance() {
		let foreign_asset_balance = 1_000;
		let asset_id = 124;

		ExtBuilder::default().build().execute_with(|| {
			let asset_id = create_fungible(ALICE, asset_id, 1);
			mint_fungible(ALICE, asset_id, foreign_asset_balance);

			let primary_id = generate_foreign_fungible_wide_id(1);
			AssetIdMapping::<Test>::insert((FOREIGN_CHAIN_ID, AssetType::Fungible, primary_id), 1);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id,
				secondary_id: generate_wide_id_for_amount(foreign_asset_balance * 10),
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				sp_runtime::TokenError::FundsUnavailable,
			);
		});
	}
}

mod non_fungible {
	use super::*;

	#[test]
	fn test_native_success() {
		ExtBuilder::default().build().execute_with(|| {
			let (collection_id, item_id) =
				mint_non_fungible(&ALICE, &create_collection(&ALICE), &1);
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

			assert_eq!(Deposits::<Test>::get((now, ALICE, asset)), Some(value));

			assert_eq!(Nft::owner(collection_id, item_id), Some(reserve_account));
		});
	}

	#[test]
	fn test_native_error_non_existent_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let (primary_id, secondary_id) = generate_native_non_fungible_wide_id(1, 1);

			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);

			let (primary_id, secondary_id) = generate_native_non_fungible_wide_id(3, 1);

			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id,
				secondary_id,
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn test_native_error_non_owned_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let (collection_id, item_id) = mint_non_fungible(&BOB, &create_collection(&BOB), &1);
			let (primary_id, secondary_id) =
				generate_native_non_fungible_wide_id(collection_id, item_id);

			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::ItemNotOwned
			);
		});
	}

	#[test]
	fn test_native_error_deposit_asset_twice() {
		ExtBuilder::default().build().execute_with(|| {
			let (collection_id, item_id) =
				mint_non_fungible(&ALICE, &create_collection(&ALICE), &1);
			let (primary_id, secondary_id) =
				generate_native_non_fungible_wide_id(collection_id, item_id);

			let asset_deposit = AssetDeposit {
				origin: CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::ItemNotOwned
			);
		});
	}

	#[test]
	fn test_foreign_success() {
		ExtBuilder::default().build().execute_with(|| {
			let (collection_id, item_id) =
				mint_non_fungible(&ALICE, &create_collection(&ALICE), &13);
			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);

			CollectionIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, primary_id),
				collection_id,
			);
			ItemIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, secondary_id),
				item_id,
			);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));

			assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

			let asset = AssetOf::<Test> {
				origin: FOREIGN_CHAIN_ID,
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

			assert_eq!(Deposits::<Test>::get((now, ALICE, asset)), Some(value));

			assert_eq!(Nft::owner(collection_id, item_id), None);
		});
	}

	#[test]
	fn test_foreign_error_non_existent_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let collection_id = 1;
			let item_id = 1;
			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);
			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			CollectionIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, primary_id),
				collection_id,
			);
			ItemIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, secondary_id),
				item_id,
			);

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);

			let collection_id = create_collection(&ALICE);
			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);

			CollectionIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, primary_id),
				collection_id,
			);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);
		});
	}

	#[test]
	fn test_foreign_error_non_owned_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let (collection_id, item_id) = mint_non_fungible(&BOB, &create_collection(&BOB), &1);
			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);

			CollectionIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, primary_id),
				collection_id,
			);
			ItemIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, secondary_id),
				item_id,
			);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::ItemNotOwned
			);
		});
	}

	#[test]
	fn test_foreign_error_deposit_asset_twice() {
		ExtBuilder::default().build().execute_with(|| {
			let (collection_id, item_id) =
				mint_non_fungible(&ALICE, &create_collection(&ALICE), &13);
			let (primary_id, secondary_id) =
				generate_foreign_non_fungible_wide_id(collection_id, item_id);

			CollectionIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, primary_id),
				collection_id,
			);
			ItemIdMapping::<Test>::insert(
				(FOREIGN_CHAIN_ID, AssetType::NonFungible, secondary_id),
				item_id,
			);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::NonFungible,
				primary_id,
				secondary_id,
			};

			assert_ok!(Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit));

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::InvalidInput
			);
		});
	}
}

mod frozen {
	use super::*;

	#[test]
	fn cannot_deposit_if_frozen() {
		ExtBuilder::default().build().execute_with(|| {
			Frozen::<Test>::set(Some(0));

			let primary_id = generate_foreign_fungible_wide_id(1);
			AssetIdMapping::<Test>::insert((FOREIGN_CHAIN_ID, AssetType::Fungible, primary_id), 1);

			let asset_deposit = AssetDeposit {
				origin: FOREIGN_CHAIN_ID,
				asset_type: AssetType::Fungible,
				primary_id,
				secondary_id: generate_wide_id_for_amount(0),
			};

			assert_noop!(
				Wildcard::deposit(RuntimeOrigin::signed(ALICE), asset_deposit),
				Error::<Test>::PalletFrozen,
			);
		});
	}
}
