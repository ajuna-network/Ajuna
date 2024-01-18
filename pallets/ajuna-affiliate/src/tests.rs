use super::*;
//use frame_support::traits::GetCallIndex;

use crate::mock::{Affiliates, Balances, ExtBuilder, RuntimeOrigin, Test};

#[test]
fn test_a() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::traits::PalletInfoAccess;
		let balance_index = Balances::index();
		println!("index is {:?}", balance_index);
		// trigger an error to easily see the print statement
		let call_index =
			pallet::Call::<Test>::add_rule_to { extrinsic_id: (0, 0), rule: 0 }.get_call_index();
		println!("call index is {:?}", call_index);

		Affiliates::add_rule_to(RuntimeOrigin::signed(1), (1, 1), 1);

		assert!(false);
	});
}
