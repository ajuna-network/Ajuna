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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod traits;

use frame_support::{
	dispatch::GetDispatchInfo,
	pallet_prelude::*,
	traits::{GetCallIndex, IsSubType},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use sp_runtime::traits::Dispatchable;

use traits::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub type AffiliatedAccountsOf<T> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::AffiliateLimit>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum amount of affiliates for a single account,
		#[pallet::constant]
		type AffiliateLimit: Get<u32>;

		/// The maximum depth of the affiliate relation chain,
		#[pallet::constant]
		type AffiliateMaxLevel: Get<u32>;
	}

	#[pallet::storage]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	/// Stores the affiliated accounts from the perspectives of the affiliatee
	#[pallet::storage]
	#[pallet::getter(fn affiliatees)]
	pub type Affiliatees<T: Config> =
		StorageMap<_, Identity, T::AccountId, T::AccountId, OptionQuery>;

	/// Stores the affiliated accounts from the perspective of the affiliator
	#[pallet::storage]
	#[pallet::getter(fn affiliators)]
	pub type Affiliator<T: Config> =
		StorageMap<_, Identity, T::AccountId, AffiliatedAccountsOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An organizer has been set.
		OrganizerSet {
			organizer: T::AccountId,
		},
		AccountAffiliated {
			account: T::AccountId,
			by: T::AccountId,
		},
		RuleExecuted {
			extrinsic_id: ExtrinsicId,
			account: T::AccountId,
			beneficiary: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// This account has already been affiliated by another affiliator
		AlreadyAffiliated,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({10_000})]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		pub fn add_rule_to(
			origin: OriginFor<T>,
			extrinsic_id: ExtrinsicId,
			rule: u8,
		) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;
			println!("{}", Self::index());
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({10_000})]
		pub fn clear_rule_from(origin: OriginFor<T>, extrinsic_id: ExtrinsicId) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({10_000})]
		pub fn add_referral(origin: OriginFor<T>, referral: T::AccountId) -> DispatchResult {
			let referer = ensure_signed(origin)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check that the origin is an organizer account.
		pub(crate) fn ensure_organizer(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(maybe_organizer)
		}
	}

	impl<T: Config> AffiliateHandler<<T as frame_system::Config>::AccountId> for Pallet<T> {
		fn get_affiliator_for(account: T::AccountId) -> Option<T::AccountId> {
			None
		}

		fn has_rule_for(extrinsic_id: ExtrinsicId) -> bool {
			false
		}

		fn execute_rule_for(
			extrinsic_id: ExtrinsicId,
			account: T::AccountId,
		) -> Result<(), DispatchError> {
			Ok(())
		}
	}
}
