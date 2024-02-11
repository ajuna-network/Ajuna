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

pub mod xcmp_queue {
	use cumulus_pallet_xcmp_queue::{Config, Pallet};
	use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade};
	use sp_std::vec::Vec;

	const TARGET: &'static str = "runtime::fix::xcmp_queue::migration";

	/// Use the actual migration provided by substrate. For some reason, they haven't supplied
	/// the struct itself, to we have to implement it.
	pub struct MigrateV0ToV3<T>(sp_std::marker::PhantomData<T>);

	mod v0 {
		use super::*;
		use codec::{Decode, Encode};

		#[derive(Encode, Decode, Debug)]
		pub struct QueueConfigData {
			pub suspend_threshold: u32,
			pub drop_threshold: u32,
			pub resume_threshold: u32,
			// the following two values are different from https://github.com/paritytech/polkadot-sdk/commit/b5135277f41ac33109829439060a662636d78a53#diff-7369c6248261389060a5b4809f0c2fd5a39ad928c048bd3fd4b079ac111d2321
			// We define it as u64 instead of weight. This is because back in that time the
			// `frame_support::Weight` was still just the u64 wrapper.
			pub threshold_weight: u64,
			pub weight_restrict_decay: u64,
		}

		#[storage_alias]
		pub type QueueConfig<T: Config> = StorageValue<Pallet<T>, QueueConfigData, ValueQuery>;

		impl Default for QueueConfigData {
			fn default() -> Self {
				QueueConfigData {
					suspend_threshold: 2,
					drop_threshold: 5,
					resume_threshold: 1,
					threshold_weight: 100_000,
					weight_restrict_decay: 2,
				}
			}
		}
	}

	mod v1 {
		use super::*;
		use codec::{Decode, Encode};
		use frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;

		#[derive(Encode, Decode, Debug)]
		pub struct QueueConfigData {
			pub suspend_threshold: u32,
			pub drop_threshold: u32,
			pub resume_threshold: u32,
			pub threshold_weight: u64,
			pub weight_restrict_decay: u64,
			pub xcmp_max_individual_weight: u64,
		}

		#[storage_alias]
		pub type QueueConfig<T: Config> = StorageValue<Pallet<T>, QueueConfigData, ValueQuery>;

		impl Default for QueueConfigData {
			fn default() -> Self {
				QueueConfigData {
					suspend_threshold: 2,
					drop_threshold: 5,
					resume_threshold: 1,
					threshold_weight: 100_000,
					weight_restrict_decay: 2,
					xcmp_max_individual_weight: 20u64 * WEIGHT_REF_TIME_PER_MILLIS,
				}
			}
		}
	}

	/// Migrates `QueueConfigData` from v0 (without the `xcmp_max_individual_weight` field) to
	/// v1 (with max individual weight).
	/// Uses the `Default` implementation of `QueueConfigData` to choose a value for
	/// `xcmp_max_individual_weight`.
	///
	/// NOTE: Only use this function if you know what you're doing. Default to using
	/// `migrate_to_latest`.
	pub fn migrate_to_v1<T: Config>() -> Weight {
		let translate = |pre: v0::QueueConfigData| -> v1::QueueConfigData {
			v1::QueueConfigData {
				suspend_threshold: pre.suspend_threshold,
				drop_threshold: pre.drop_threshold,
				resume_threshold: pre.resume_threshold,
				threshold_weight: pre.threshold_weight,
				weight_restrict_decay: pre.weight_restrict_decay,
				xcmp_max_individual_weight: v1::QueueConfigData::default()
					.xcmp_max_individual_weight,
			}
		};

		// We can't use `QueueConfig::translate` as they do in the xcmp_pallet
		// because `QueueConfig` is private. This is why we have to use the
		// `storage_alias` proc_mac.
		let config_v0 = v0::QueueConfig::<T>::get();
		let config_v1 = translate(config_v0);
		v1::QueueConfig::<T>::put(config_v1);

		T::DbWeight::get().reads_writes(1, 1)
	}

	impl<T: Config> OnRuntimeUpgrade for MigrateV0ToV3<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			// We are executing a parity migration here. I don't know why they didn't give us the
			// full struct, but I will omit the tests for now.
			Ok(0.encode())
		}

		fn on_runtime_upgrade() -> Weight {
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			if onchain_version >= 3 {
				log::warn!(
					target: TARGET,
					"skipping v0 to v3 migration: executed on wrong storage version.\
				Expected version < 3, found {:?}",
					onchain_version,
				);
				return T::DbWeight::get().reads(1)
			}

			let mut weight = T::DbWeight::get().reads(1);
			if StorageVersion::get::<Pallet<T>>() == 0 {
				log::info!(target: TARGET, "running our own migration from 0 to 1");
				weight += migrate_to_v1::<T>();
				StorageVersion::new(1).put::<Pallet<T>>();
			}

			let version_new = StorageVersion::get::<Pallet<T>>();

			log::info!(target: TARGET, "running the pallets migration from {:?} to 3", version_new);
			weight += cumulus_pallet_xcmp_queue::migration::migrate_to_latest::<T>();

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
			ensure!(StorageVersion::get::<Pallet<T>>() == 3, "Must upgrade");
			Ok(())
		}
	}
}
