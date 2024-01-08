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
pub struct MintConfigV5<T: Config> {
	pub open: bool,
	pub cooldown: BlockNumberFor<T>,
	pub free_mint_fee_multiplier: MintCount,
}

impl<T> MintConfigV5<T>
where
	T: Config,
{
	fn migrate_to_v6(self) -> MintConfig<BlockNumberFor<T>> {
		MintConfig {
			open: self.open,
			cooldown: self.cooldown,
			free_mint_fee_multiplier: self.free_mint_fee_multiplier,
		}
	}
}

#[derive(Decode)]
pub struct ForgeConfigV5 {
	pub open: bool,
}

impl ForgeConfigV5 {
	fn migrate_to_v6(self) -> ForgeConfig {
		ForgeConfig { open: self.open }
	}
}

#[derive(Decode)]
pub struct TransferConfigV5 {
	pub open: bool,
	pub free_mint_transfer_fee: MintCount,
	pub min_free_mint_transfer: MintCount,
}

impl TransferConfigV5 {
	fn migrate_to_v6(self) -> TransferConfig {
		TransferConfig {
			open: self.open,
			free_mint_transfer_fee: self.free_mint_transfer_fee,
			min_free_mint_transfer: self.min_free_mint_transfer,
		}
	}
}

#[derive(Decode)]
pub struct TradeConfigV5 {
	pub open: bool,
}

impl TradeConfigV5 {
	fn migrate_to_v6(self) -> TradeConfig {
		TradeConfig { open: self.open }
	}
}

#[derive(Decode)]
pub struct NftTransferConfigV5 {
	pub open: bool,
}

impl NftTransferConfigV5 {
	fn migrate_to_v6(self) -> NftTransferConfig {
		NftTransferConfig { open: self.open }
	}
}

#[derive(Decode)]
pub struct GlobalConfigV5<T: Config> {
	pub mint: MintConfigV5<T>,
	pub forge: ForgeConfigV5,
	pub transfer: TransferConfigV5,
	pub trade: TradeConfigV5,
	pub nft_transfer: NftTransferConfigV5,
}

impl<T> GlobalConfigV5<T>
where
	T: Config,
{
	fn migrate_to_v6(self) -> GlobalConfig<BlockNumberFor<T>> {
		GlobalConfig {
			mint: self.mint.migrate_to_v6(),
			forge: self.forge.migrate_to_v6(),
			transfer: self.transfer.migrate_to_v6(),
			freemint_transfer: FreemintTransferConfig { mode: FreeMintTransferMode::Open },
			trade: self.trade.migrate_to_v6(),
			nft_transfer: self.nft_transfer.migrate_to_v6(),
		}
	}
}

pub struct MigrateToV6<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV6<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 5 && current_version == 6 {
			let _ = GlobalConfigs::<T>::translate::<GlobalConfigV5<T>, _>(|old_config| {
				log::info!(target: LOG_TARGET, "Updated GlobalConfig from v5 to v6");
				old_config.map(|old| old.migrate_to_v6())
			});

			current_version.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version);
			T::DbWeight::get().reads_writes(1, 1)
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration for v6 did not execute."
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();

		if onchain_version == 5 && current_version == 6 {
			log::info!(
				target: LOG_TARGET,
				"Nothing to check in v5 -> v6 migration."
			);
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration post upgrade v6 did not execute."
			);
		}

		Ok(())
	}
}
