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
pub enum OldStorageTier {
	One = 25,
	Two = 50,
	Three = 75,
	Four = 100,
}

impl OldStorageTier {
	fn migrate_to_v3(self) -> StorageTier {
		match self {
			OldStorageTier::One => StorageTier::One,
			OldStorageTier::Two => StorageTier::Two,
			OldStorageTier::Three => StorageTier::Three,
			OldStorageTier::Four => StorageTier::Four,
		}
	}
}

#[derive(Decode)]
pub struct OldAccountInfo<BlockNumber> {
	pub free_mints: MintCount,
	pub storage_tier: OldStorageTier,
	pub stats: Stats<BlockNumber>,
}

impl<BlockNumber> OldAccountInfo<BlockNumber> {
	fn migrate_to_v3(self) -> AccountInfo<BlockNumber> {
		AccountInfo {
			free_mints: self.free_mints,
			storage_tier: self.storage_tier.migrate_to_v3(),
			stats: self.stats,
		}
	}
}

pub struct MigrateToV3<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV3<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 2 && current_version == 3 {
			let mut translated = 0_u64;
			Accounts::<T>::translate::<OldAccountInfo<T::BlockNumber>, _>(|_key, old_value| {
				translated.saturating_inc();
				Some(old_value.migrate_to_v3())
			});
			log::info!(target: LOG_TARGET, "Updated {} accounts", translated);

			translated = 0;
			v2::Owners::<T>::iter().for_each(|(account_id, avatar_ids)| {
				avatar_ids.iter().for_each(|avatar_id| {
					let maybe_avatar = Avatars::<T>::get(avatar_id);
					if let Some((_owner, avatar)) = maybe_avatar {
						translated.saturating_inc();
						Owners::<T>::try_append(account_id.clone(), avatar.season_id, avatar_id)
							.unwrap();
					}
				});
			});
			log::info!(target: LOG_TARGET, "Updated {} avatars", translated);

			current_version.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version);
			T::DbWeight::get().reads_writes(translated + 1, translated + 1)
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
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 3);
		Ok(())
	}
}
