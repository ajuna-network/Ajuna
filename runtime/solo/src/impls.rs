// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{Assets, Balances, Treasury};
use ajuna_primitives::{AccountId, Balance};
use frame_support::traits::{
	fungibles::{Credit, Inspect},
	Currency, OnUnbalanced,
};
use pallet_asset_tx_payment::HandleCredit;
use sp_runtime::{traits::Convert, FixedPointNumber, FixedU128};

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct OneToOneConversion;
impl<Balance, AssetBalance: From<Balance>> Convert<Balance, AssetBalance> for OneToOneConversion {
	fn convert(balance: Balance) -> AssetBalance {
		balance.into()
	}
}

pub struct CreditToTreasury;
impl HandleCredit<AccountId, Assets> for CreditToTreasury {
	// Converts asset balance to balance by reversing BalanceToAssetBalance::to_asset_balance()
	//   (to balance) asset_balance = balance * asset_min_balance / min_balance
	//   (to asset balance) balance = asset_balance * min_balance / asset_min_balance
	fn handle_credit(credit: Credit<AccountId, Assets>) {
		let asset_id = credit.asset();
		let minimum_asset_balance: Balance =
			OneToOneConversion::convert(Assets::minimum_balance(asset_id));
		let minimum_balance: Balance = OneToOneConversion::convert(Balances::minimum_balance());
		let amount = FixedU128::saturating_from_rational(minimum_balance, minimum_asset_balance)
			.saturating_mul_int(credit.peek());
		let amount = NegativeImbalance::new(amount);
		Treasury::on_unbalanced(amount);
	}
}
