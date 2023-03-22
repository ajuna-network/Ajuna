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

#[derive(Decode)]
pub struct OldMintConfig<T: Config> {
	pub open: bool,
	pub fees: MintFees<BalanceOf<T>>,
	pub cooldown: T::BlockNumber,
	pub free_mint_fee_multiplier: MintCount,
	pub free_mint_transfer_fee: MintCount,
	pub min_free_mint_transfer: MintCount,
}

impl<T: Config> OldMintConfig<T> {
	fn migrate_to_v2(
		self,
	) -> (MintConfig<BalanceOf<T>, T::BlockNumber>, TransferConfig<BalanceOf<T>>) {
		(
			MintConfig {
				open: self.open,
				fees: self.fees,
				cooldown: self.cooldown,
				free_mint_fee_multiplier: self.free_mint_fee_multiplier,
			},
			TransferConfig {
				open: true,
				free_mint_transfer_fee: self.free_mint_transfer_fee,
				min_free_mint_transfer: self.min_free_mint_transfer,
				avatar_transfer_fee: 1_000_000_000_000_u64.unique_saturated_into(),
			},
		)
	}
}

#[derive(Decode)]
pub struct OldGlobalConfig<T: Config> {
	pub mint: OldMintConfig<T>,
	pub forge: ForgeConfig,
	pub trade: TradeConfig<BalanceOf<T>>,
	pub account: AccountConfig<BalanceOf<T>>,
}

impl<T: Config> OldGlobalConfig<T> {
	fn migrate_to_v2(self) -> GlobalConfig<BalanceOf<T>, T::BlockNumber> {
		let (mint, transfer) = self.mint.migrate_to_v2();
		GlobalConfig { mint, forge: self.forge, transfer, trade: self.trade, account: self.account }
	}
}

#[frame_support::storage_alias]
pub type Owners<T: Config> =
	StorageMap<Pallet<T>, Identity, AccountIdOf<T>, BoundedAvatarIdsOf<T>, ValueQuery>;

pub struct MigrateToV2<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 1 && current_version == 2 {
			let _ = GlobalConfigs::<T>::translate::<OldGlobalConfig<T>, _>(|maybe_old_value| {
				maybe_old_value.map(|old_value| {
					log::info!(target: LOG_TARGET, "Migrated global config");
					old_value.migrate_to_v2()
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
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 2);
		let GlobalConfig { transfer, .. } = GlobalConfigs::<T>::get();
		assert!(transfer.open);
		assert_eq!(transfer.avatar_transfer_fee, 1_000_000_000_000_u64.unique_saturated_into());
		Ok(())
	}
}
