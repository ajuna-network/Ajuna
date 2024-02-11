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

//! The following are temporary local migration fixes to solve inconsistencies caused by not
//! migrating Storage at the time of migrating runtime code

pub mod scheduler {
	// this is necessary because migrations from v0 to v3 are no longer available in the scheduler
	// pallet code and migrating is only possible from v3. The strategy here is to empty the agenda,
	// which it already is.
	use frame_support::traits::OnRuntimeUpgrade;
	use frame_system::pallet_prelude::BlockNumberFor;
	use pallet_scheduler::*;
	use sp_std::vec::Vec;

	/// The log target.
	const TARGET: &'static str = "runtime::fix::scheduler::migration";

	pub mod v1 {
		use super::*;
		use frame_support::{pallet_prelude::*, traits::schedule};

		#[cfg_attr(any(feature = "std", test), derive(PartialEq, Eq))]
		#[derive(Clone, RuntimeDebug, Encode, Decode)]
		pub(crate) struct ScheduledV1<Call, BlockNumber> {
			maybe_id: Option<Vec<u8>>,
			priority: schedule::Priority,
			call: Call,
			maybe_periodic: Option<schedule::Period<BlockNumber>>,
		}

		#[frame_support::storage_alias]
		pub(crate) type Agenda<T: Config> = StorageMap<
			Pallet<T>,
			Twox64Concat,
			BlockNumberFor<T>,
			Vec<Option<ScheduledV1<<T as Config>::RuntimeCall, BlockNumberFor<T>>>>,
			ValueQuery,
		>;

		#[frame_support::storage_alias]
		pub(crate) type Lookup<T: Config> =
			StorageMap<Pallet<T>, Twox64Concat, Vec<u8>, TaskAddress<BlockNumberFor<T>>>;
	}

	pub mod v4 {
		use super::*;
		use frame_support::pallet_prelude::*;

		#[frame_support::storage_alias]
		pub type Agenda<T: Config> = StorageMap<
			Pallet<T>,
			Twox64Concat,
			BlockNumberFor<T>,
			BoundedVec<
				Option<ScheduledOf<T>>,
				<T as pallet_scheduler::Config>::MaxScheduledPerBlock,
			>,
			ValueQuery,
		>;

		pub(crate) type TaskName = [u8; 32];

		#[frame_support::storage_alias]
		pub(crate) type Lookup<T: Config> =
			StorageMap<Pallet<T>, Twox64Concat, TaskName, TaskAddress<BlockNumberFor<T>>>;

		/// Migrate the scheduler pallet from V0 to V4 without changing storage, as there is no
		/// active schedule anyhow.
		pub struct MigrateToV4<T>(sp_std::marker::PhantomData<T>);

		impl<T: Config> OnRuntimeUpgrade for MigrateToV4<T> {
			#[cfg(feature = "try-runtime")]
			fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
				let agendas = v1::Agenda::<T>::iter_keys().count() as u32;
				let lookups = v1::Lookup::<T>::iter_keys().count() as u32;
				log::info!(target: TARGET, "agendas present which will be left untouched: {}/{}...", agendas, lookups);
				Ok((agendas, lookups).encode())
			}

			fn on_runtime_upgrade() -> Weight {
				let onchain_version = Pallet::<T>::on_chain_storage_version();
				if onchain_version >= 3 {
					log::warn!(
						target: TARGET,
						"skipping v0 to v4 migration: executed on wrong storage version.\
					Expected version < 3, found {:?}",
						onchain_version,
					);
					return T::DbWeight::get().reads(1)
				}
				log::info!(target: TARGET, "migrating from {:?} to 4", onchain_version);
				StorageVersion::new(4).put::<Pallet<T>>();

				T::DbWeight::get().reads_writes(1, 1)
			}

			#[cfg(feature = "try-runtime")]
			fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
				ensure!(StorageVersion::get::<Pallet<T>>() == 4, "Must upgrade");

				let agendas = Agenda::<T>::iter_keys().count() as u32;
				let lookups = Lookup::<T>::iter_keys().count() as u32;
				log::info!(target: TARGET, "agendas present a posteriori: {}/{}...", agendas, lookups);
				Ok(())
			}
		}
	}
}

pub mod parachain_systems {
	use cumulus_pallet_parachain_system::{Config, Pallet};
	use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
	use frame_system::pallet_prelude::BlockNumberFor;
	use sp_std::vec::Vec;

	const TARGET: &'static str = "runtime::fix::parachain_systems::migration";

	/// This is a mock migration, because the pallet has been declaring internal migrations,
	/// which does migrate the data to prevent chain brickages in case they are forgotten.
	/// However, these pallet hooks don't update the storage version. Hence, we only have to
	/// set the storage version here.
	pub struct MigrateV0ToV2<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for MigrateV0ToV2<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			// We are executing a parity migration here. I don't know why they didn't give us the
			// full struct, but I will omit the tests for now.
			Ok(0.encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			let mut weight = T::DbWeight::get().reads(1);
			if onchain_version >= 2 {
				log::warn!(
					target: TARGET,
					"skipping v0 to v2 migration: executed on wrong storage version.\
				Expected version < 2, found {:?}",
					onchain_version,
				);
				return weight
			}

			log::info!(target: TARGET, "migrating from {:?} to 2", onchain_version);
			StorageVersion::new(2).put::<Pallet<T>>();
			weight.saturating_accrue(T::DbWeight::get().writes(1));

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
			ensure!(StorageVersion::get::<Pallet<T>>() == 2, "Must upgrade");
			Ok(())
		}
	}
}

pub mod xcmp_queue {
	use cumulus_pallet_xcmp_queue::{Config, Pallet};
	use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};
	use sp_std::vec::Vec;

	const TARGET: &'static str = "runtime::fix::xcmp_queue::migration";

	/// This is a mock migration, because the pallet has been declaring internal migrations,
	/// which does migrate the data to prevent chain brickages in case they are forgotten.
	/// However, these pallet hooks don't update the storage version. Hence, we only have to
	/// set the storage version here.
	///
	/// This can be confirmed by inspecting the current on chain data for `QueueConfig`.
	pub struct MigrateV0ToV3<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for MigrateV0ToV3<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			// We are executing a parity migration here. I don't know why they didn't give us the
			// full struct, but I will omit the tests for now.
			Ok(0.encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			let mut weight = T::DbWeight::get().reads(1);
			if onchain_version >= 3 {
				log::warn!(
					target: TARGET,
					"skipping v0 to v3 migration: executed on wrong storage version.\
				Expected version < 3, found {:?}",
					onchain_version,
				);
				return weight
			}

			StorageVersion::new(3).put::<Pallet<T>>();
			weight.saturating_accrue(T::DbWeight::get().writes(1));

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
			ensure!(StorageVersion::get::<Pallet<T>>() == 3, "Must upgrade");
			Ok(())
		}
	}
}
