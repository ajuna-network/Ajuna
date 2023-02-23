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
use frame_support::traits::OnRuntimeUpgrade;

// The current storage version.
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

const LOG_TARGET: &str = "runtime::ajuna-awesome-avatars";

pub mod v1 {
	use super::*;

	#[derive(Decode)]
	pub struct OldTradeConfig<Balance> {
		pub open: bool,
		pub buy_fee: Balance,
	}

	#[derive(Decode, Encode, Default)]
	pub struct TradeConfigV1<Balance> {
		pub open: bool,
		pub min_fee: Balance,
		pub percent_fee: u8,
	}

	impl<Balance> OldTradeConfig<Balance> {
		fn migrate_to_v1(self) -> TradeConfigV1<Balance> {
			TradeConfigV1 { open: self.open, min_fee: self.buy_fee, percent_fee: 1 }
		}
	}

	#[derive(Decode)]
	pub struct OldGlobalConfig<Balance, BlockNumber> {
		pub mint: MintConfig<Balance, BlockNumber>,
		pub forge: ForgeConfig,
		pub trade: OldTradeConfig<Balance>,
		pub account: AccountConfig<Balance>,
	}

	#[derive(Decode, Encode, Default)]
	pub struct GlobalConfigV1<Balance, BlockNumber> {
		pub mint: MintConfig<Balance, BlockNumber>,
		pub forge: ForgeConfig,
		pub trade: TradeConfigV1<Balance>,
		pub account: AccountConfig<Balance>,
	}

	impl<Balance, BlockNumber> OldGlobalConfig<Balance, BlockNumber> {
		fn migrate_to_v1(self) -> GlobalConfigV1<Balance, BlockNumber> {
			GlobalConfigV1 {
				mint: self.mint,
				forge: self.forge,
				trade: self.trade.migrate_to_v1(),
				account: self.account,
			}
		}
	}

	#[frame_support::storage_alias]
	pub(crate) type GlobalConfigs<T: Config> =
		StorageValue<Pallet<T>, GlobalConfigV1<BalanceOf<T>, BlockNumberFor<T>>, ValueQuery>;

	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			if onchain_version == 0 && current_version == 1 {
				let _ = GlobalConfigs::<T>::translate::<
					OldGlobalConfig<BalanceOf<T>, T::BlockNumber>,
					_,
				>(|maybe_old_value| {
					maybe_old_value.map(|old_value| {
						log::info!(target: LOG_TARGET, "migrated global config");
						old_value.migrate_to_v1()
					})
				});
				current_version.put::<Pallet<T>>();
				log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version,);
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
			assert_eq!(Pallet::<T>::on_chain_storage_version(), 1);
			let global_configs = GlobalConfigs::<T>::get();
			assert_eq!(global_configs.trade.min_fee, 1_000_000_000_u64.unique_saturated_into());
			assert_eq!(global_configs.trade.percent_fee, 1);
			Ok(())
		}
	}
}
