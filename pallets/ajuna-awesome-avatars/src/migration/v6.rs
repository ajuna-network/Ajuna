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
pub struct OldSeasonStats {
	pub minted: Stat,
	pub forged: Stat,
}

impl OldSeasonStats {
	fn migrate_to_v6(self) -> SeasonInfo {
		SeasonInfo { minted: self.minted, forged: self.forged, disposed: 0, lost_soul_points: 0 }
	}
}

#[frame_support::storage_alias]
pub(crate) type CurrentSeasonId<T: Config> = StorageValue<Pallet<T>, SeasonId, ValueQuery>;

pub struct MigrateToV5<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV5<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();

		let mut translated_season_stats = 0_u64;
		if onchain_version == 5 && current_version == 6 {
			SeasonStats::<T>::translate::<OldSeasonStats, _>(|_, _, old_stats| {
				log::info!(target: LOG_TARGET, "Updated SeasonStats");
				Some(old_stats.migrate_to_v6())
			});
			translated_season_stats += 1;

			T::DbWeight::get().reads_writes(translated_season_stats, translated_season_stats)
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
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 6);
		assert!(CurrentSeasonStatus::<T>::get().season_id > Zero::zero());

		// There are 2,024 season stats entries as of as of 08/08/2023. But the exact number could
		// change as avatars are traded between accounts.
		// We estimate there should be between 2,000 and 2,500 accounts.
		let season_stats = SeasonStats::<T>::iter_keys().collect::<Vec<_>>();
		assert!(season_stats.len() >= 2000 && season_stats.len() <= 2500);

		Ok(())
	}
}
