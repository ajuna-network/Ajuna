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

use super::*;

#[derive(Decode, Encode, Default)]
pub struct OldGlobalConfig<T: Config> {
	pub mint: MintConfig<BalanceOf<T>, T::BlockNumber>,
	pub forge: ForgeConfig,
	pub transfer: TransferConfig<BalanceOf<T>>,
	pub trade: TradeConfig<BalanceOf<T>>,
	pub account: AccountConfig<BalanceOf<T>>,
}

impl<T: Config> OldGlobalConfig<T> {
	fn migrate_to_v4(self) -> GlobalConfig<BalanceOf<T>, T::BlockNumber> {
		GlobalConfig {
			mint: self.mint,
			forge: self.forge,
			transfer: self.transfer,
			trade: self.trade,
			account: self.account,
			nft_transfer: NftTransferConfig { open: true },
		}
	}
}

pub struct MigrateToV4<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV4<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 3 && current_version == 4 {
			let _ = GlobalConfigs::<T>::translate::<OldGlobalConfig<T>, _>(|maybe_old_value| {
				maybe_old_value.map(|old_value| {
					log::info!(target: LOG_TARGET, "Migrated global config");
					old_value.migrate_to_v4()
				})
			});

			let treasury_balance = Pallet::<T>::treasury(1);
			T::Currency::make_free_balance_be(
				&Pallet::<T>::treasury_account_id(),
				treasury_balance.saturating_add(T::Currency::minimum_balance()),
			);

			current_version.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version);
			T::DbWeight::get().reads_writes(2, 2)
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 4);
		let GlobalConfig { nft_transfer, .. } = GlobalConfigs::<T>::get();
		assert!(nft_transfer.open);

		let treasury_balance = Pallet::<T>::treasury(1);
		assert!(treasury_balance > Zero::zero());
		assert!(
			T::Currency::free_balance(&Pallet::<T>::treasury_account_id()) ==
				treasury_balance.saturating_add(T::Currency::minimum_balance())
		);
		Ok(())
	}
}
