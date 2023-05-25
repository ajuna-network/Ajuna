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
use frame_support::storage::migration;

#[derive(Decode)]
pub struct OldSeasonStatus {
	pub early: bool,
	pub active: bool,
	pub early_ended: bool,
	pub max_tier_avatars: u32,
}

impl OldSeasonStatus {
	fn migrate_to_v5(self) -> SeasonStatus {
		SeasonStatus {
			season_id: SeasonId::default(),
			early: self.early,
			active: self.active,
			early_ended: self.early_ended,
			max_tier_avatars: self.max_tier_avatars,
		}
	}
}

#[frame_support::storage_alias]
pub(crate) type CurrentSeasonId<T: Config> = StorageValue<Pallet<T>, SeasonId, ValueQuery>;

pub struct MigrateToV5<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateToV5<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::current_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 4 && current_version == 5 {
			let _ = CurrentSeasonStatus::<T>::translate::<OldSeasonStatus, _>(|maybe_old_value| {
				maybe_old_value.map(|old_value| {
					log::info!(target: LOG_TARGET, "Migrated current season status");
					let mut new_value = old_value.migrate_to_v5();
					new_value.season_id = CurrentSeasonId::<T>::get();
					new_value
				})
			});

			let owners = migration::storage_iter::<BoundedAvatarIdsOf<T>>(
				<Pallet<T>>::name().as_bytes(),
				b"Owners",
			)
			.drain()
			.filter(|(_owner, avatar_ids)| !avatar_ids.is_empty())
			.map(|(owner, avatar_ids)| (T::AccountId::decode(&mut &owner[..]).unwrap(), avatar_ids))
			.collect::<Vec<_>>();

			let mut translated_account = 0_u64;
			let mut translated_avatars = 0_u64;
			owners.iter().for_each(|(owner, avatar_ids)| {
				let season_id = 1;
				Owners::<T>::insert(owner, season_id, avatar_ids);
				translated_account += 1;
				translated_avatars += avatar_ids.len() as u64;
			});
			log::info!(
				target: LOG_TARGET,
				"Updated {} accounts and {} avatars",
				translated_account,
				translated_avatars,
			);

			current_version.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version);
			T::DbWeight::get().reads_writes(2 + 2 * translated_account, 2 + 2 * translated_account)
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
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 5);
		assert!(CurrentSeasonStatus::<T>::get().season_id > Zero::zero());

		let mut avatar_ids_from_avatars = Avatars::<T>::iter_keys().collect::<Vec<_>>();
		avatar_ids_from_avatars.sort();
		avatar_ids_from_avatars.dedup();

		let mut avatar_ids_from_owners = Owners::<T>::iter_values().flatten().collect::<Vec<_>>();
		avatar_ids_from_owners.sort();
		avatar_ids_from_owners.dedup();

		// There should be 13,111 avatars in storage.
		assert_eq!(avatar_ids_from_avatars.len(), 13_111);
		assert_eq!(avatar_ids_from_owners.len(), 13_111);
		assert_eq!(avatar_ids_from_avatars, avatar_ids_from_owners);

		// There should be 892 owners of avatars in storage.
		let mut season_ids = Owners::<T>::iter_keys()
			.filter(|(owner, season_id)| !Owners::<T>::get(owner, season_id).is_empty())
			.map(|(_owner, season_id)| season_id)
			.collect::<Vec<SeasonId>>();
		assert_eq!(season_ids.len(), 892);

		// Check all migrated season IDs are 1.
		season_ids.sort();
		season_ids.dedup();
		assert_eq!(season_ids.len(), 1);
		assert_eq!(season_ids, vec![1]);

		Ok(())
	}
}
